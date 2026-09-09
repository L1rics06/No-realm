//! Realm file header parsing with binrw.
//!
//! The header occupies the first 24 bytes of a Realm file:
//! - Bytes 0-7: top_ref[0] (u64 LE) - first root pointer
//! - Bytes 8-15: top_ref[1] (u64 LE) - second root pointer (MVCC)
//! - Bytes 16-19: magic bytes (0x54, 0x2D, 0x44, 0x42) = "T-DB"
//! - Bytes 20-21: file_format_version (u16 LE)
//! - Byte 22: reserved (always 0)
//! - Byte 23: flags
//!   - Bit 0: determines which top_ref is active (0=top_ref[0], 1=top_ref[1])
//!   - Bit 7 (0x80): encryption flag (1=encrypted, must reject)

use binrw::binrw;
use crate::error::{Error, Result};

/// Realm file format version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatVersion {
    /// Legacy format (version 9) - supported by realm-codec
    Legacy9,
    /// Modern cluster-based format (version 24) - requires custom parsing
    Modern24,
    /// Unknown version
    Unknown(u16),
}

impl FormatVersion {
    /// Create from raw version number.
    ///
    /// Note: Realm stores versions as (major << 8) | minor
    /// - 9 = version 9
    /// - 0x1818 (6168) = version 24.24
    pub fn from_u16(v: u16) -> Self {
        match v {
            9 => FormatVersion::Legacy9,
            24 | 0x1818 => FormatVersion::Modern24,  // 24 or 24.24
            other => FormatVersion::Unknown(other),
        }
    }

    /// Check if this version is supported.
    pub fn is_supported(&self) -> bool {
        matches!(self, FormatVersion::Legacy9 | FormatVersion::Modern24)
    }
}

impl std::fmt::Display for FormatVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatVersion::Legacy9 => write!(f, "9 (legacy)"),
            FormatVersion::Modern24 => write!(f, "24 (modern)"),
            FormatVersion::Unknown(v) => write!(f, "{} (unknown)", v),
        }
    }
}

/// Parsed Realm file header (24 bytes).
///
/// Validated during parsing:
/// - Magic bytes must be "T-DB"
/// - File format version must be supported (9 or 24)
/// - Encrypted files are rejected
#[binrw]
#[derive(Debug, Clone)]
#[brw(little, magic = b"")]  // Custom validation below
pub struct RealmHeader {
    /// First root node reference (MVCC copy-on-write)
    pub top_ref_0: u64,

    /// Second root node reference (MVCC copy-on-write)
    pub top_ref_1: u64,

    /// Magic bytes: should be [0x54, 0x2D, 0x44, 0x42] = "T-DB"
    #[br(count = 4)]
    pub magic: Vec<u8>,

    /// File format version (9 = legacy, 24 = modern)
    pub file_format_version: u16,

    /// Reserved byte (always 0)
    pub reserved: u8,

    /// Flags byte:
    /// - Bit 0: active top_ref selector
    /// - Bit 7 (0x80): encryption flag
    pub flags: u8,
}

impl RealmHeader {
    /// Parse header from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Data is less than 24 bytes
    /// - Magic bytes are not "T-DB"
    /// - File is encrypted
    /// - Format version is unsupported
    pub fn parse(data: &[u8]) -> Result<Self> {
        use binrw::BinRead;
        use std::io::Cursor;

        if data.len() < 24 {
            return Err(Error::realm(format!(
                "File too small for header: {} bytes (expected 24) / 文件太小：{} 字节（需要 24）",
                data.len(), data.len()
            )));
        }

        let mut cursor = Cursor::new(data);
        let header = RealmHeader::read(&mut cursor).map_err(|e| {
            Error::realm(format!(
                "Failed to parse header: {} / 解析文件头失败：{}",
                e, e
            ))
        })?;

        // Validate magic bytes
        if header.magic != b"T-DB" {
            return Err(Error::database_corrupted(format!(
                "Invalid magic bytes: expected T-DB, got {:?} / 无效的魔数：期望 T-DB，实际 {:?}",
                header.magic, header.magic
            )));
        }

        // Check encryption
        if header.is_encrypted() {
            return Err(Error::realm(
                "Encrypted Realm files are not supported / 不支持加密的 Realm 文件".to_string()
            ));
        }

        // Validate version
        let version = header.version();
        if !version.is_supported() {
            return Err(Error::realm(format!(
                "Unsupported Realm format version: {} (supported: 9, 24) / 不支持的 Realm 版本：{}（支持：9, 24）",
                version, version
            )));
        }

        Ok(header)
    }

    /// Get the format version as enum.
    pub fn version(&self) -> FormatVersion {
        FormatVersion::from_u16(self.file_format_version)
    }

    /// Check if the file is encrypted (bit 7 of flags).
    pub fn is_encrypted(&self) -> bool {
        (self.flags & 0x80) != 0
    }

    /// Get the active top_ref based on flags bit 0.
    ///
    /// Realm uses MVCC copy-on-write with two root pointers.
    /// The flags bit 0 determines which one is current.
    pub fn active_top_ref(&self) -> u64 {
        if (self.flags & 0x01) == 0 {
            self.top_ref_0
        } else {
            self.top_ref_1
        }
    }

    /// Get both top_refs as an array for MVCC operations.
    pub fn top_refs(&self) -> [u64; 2] {
        [self.top_ref_0, self.top_ref_1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_valid_header(version: u16, flags: u8) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&0x1000u64.to_le_bytes()); // top_ref_0
        data.extend_from_slice(&0x2000u64.to_le_bytes()); // top_ref_1
        data.extend_from_slice(b"T-DB");                   // magic
        data.extend_from_slice(&version.to_le_bytes());    // version
        data.push(0);                                       // reserved
        data.push(flags);                                   // flags
        data
    }

    #[test]
    fn parse_valid_v9_header() {
        let data = make_valid_header(9, 0);
        let header = RealmHeader::parse(&data).unwrap();

        assert_eq!(header.top_ref_0, 0x1000);
        assert_eq!(header.top_ref_1, 0x2000);
        assert_eq!(header.magic, b"T-DB");
        assert_eq!(header.file_format_version, 9);
        assert_eq!(header.version(), FormatVersion::Legacy9);
        assert!(!header.is_encrypted());
    }

    #[test]
    fn parse_valid_v24_header() {
        let data = make_valid_header(24, 0);
        let header = RealmHeader::parse(&data).unwrap();

        assert_eq!(header.file_format_version, 24);
        assert_eq!(header.version(), FormatVersion::Modern24);
    }

    #[test]
    fn parse_too_small() {
        let data = vec![0u8; 10];
        assert!(RealmHeader::parse(&data).is_err());
    }

    #[test]
    fn parse_invalid_magic() {
        let mut data = make_valid_header(9, 0);
        data[16..20].copy_from_slice(b"XXXX");
        assert!(RealmHeader::parse(&data).is_err());
    }

    #[test]
    fn parse_encrypted_rejected() {
        let data = make_valid_header(9, 0x80);
        let result = RealmHeader::parse(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Encrypted"));
    }

    #[test]
    fn parse_unsupported_version() {
        let data = make_valid_header(99, 0);
        let result = RealmHeader::parse(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported"));
    }

    #[test]
    fn active_top_ref_flag0() {
        let data = make_valid_header(9, 0x00); // flags bit 0 = 0
        let header = RealmHeader::parse(&data).unwrap();
        assert_eq!(header.active_top_ref(), 0x1000); // top_ref_0
    }

    #[test]
    fn active_top_ref_flag1() {
        let data = make_valid_header(9, 0x01); // flags bit 0 = 1
        let header = RealmHeader::parse(&data).unwrap();
        assert_eq!(header.active_top_ref(), 0x2000); // top_ref_1
    }

    #[test]
    fn format_version_display() {
        assert_eq!(FormatVersion::Legacy9.to_string(), "9 (legacy)");
        assert_eq!(FormatVersion::Modern24.to_string(), "24 (modern)");
        assert_eq!(FormatVersion::Unknown(99).to_string(), "99 (unknown)");
    }

    #[test]
    fn format_version_supported() {
        assert!(FormatVersion::Legacy9.is_supported());
        assert!(FormatVersion::Modern24.is_supported());
        assert!(!FormatVersion::Unknown(99).is_supported());
    }
}
