//! Array node parsing and navigation.
//!
//! Realm stores data in arrays with an 8-byte header followed by payload.
//! Arrays can contain integers, refs, or other primitives in bit-packed format.

use crate::error::{Error, Result};
use crate::realm::format::types::{RealmRef, TaggedValue, RefOrTagged};
use byteorder::{ByteOrder, LittleEndian};

/// Array node header (8 bytes).
///
/// Structure:
/// - Bytes 0-3: checksum (always 0x41414141)
/// - Byte 4: flags
/// - Bytes 5-7: size (24-bit little-endian)
#[derive(Debug, Clone, Copy)]
pub struct ArrayHeader {
    /// Checksum value (should always be 0x41414141)
    pub checksum: u32,
    /// Flags byte encoding width, type, and properties
    pub flags: u8,
    /// Number of elements in the array
    pub size: u32,
}

// Flag bit positions
const FLAG_IS_INNER: u8 = 0x80;      // Bit 7: inner B+tree node
const FLAG_HAS_REFS: u8 = 0x40;      // Bit 6: contains refs
const FLAG_CONTEXT: u8 = 0x20;       // Bit 5: context flag (for strings)
const WIDTH_MASK: u8 = 0x07;         // Bits 0-2: width encoding

// Width types (bits 3-4)
const WTYPE_BITS: u8 = 0;      // Payload = (size * width + 7) / 8 bytes
const WTYPE_MULTIPLY: u8 = 1;  // Payload = size * width bytes
const WTYPE_IGNORE: u8 = 2;    // Payload = size bytes

impl ArrayHeader {
    /// Parse header from 8 bytes.
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(Error::database_corrupted(format!(
                "Array header too small: {} bytes (expected 8) / 数组头太小：{} 字节（需要 8）",
                data.len(), data.len()
            )));
        }

        let checksum = LittleEndian::read_u32(&data[0..4]);
        let flags = data[4];

        // Size is 3 bytes big-endian (MSB first) at positions 5-7
        // byte[5] = MSB, byte[6] = middle, byte[7] = LSB
        let size = ((data[5] as u32) << 16) | ((data[6] as u32) << 8) | (data[7] as u32);

        Ok(ArrayHeader {
            checksum,
            flags,
            size,
        })
    }

    /// Check if this is an inner B+tree node.
    pub fn is_inner_node(&self) -> bool {
        (self.flags & FLAG_IS_INNER) != 0
    }

    /// Check if this array contains refs (vs inline values).
    pub fn has_refs(&self) -> bool {
        (self.flags & FLAG_HAS_REFS) != 0
    }

    /// Get the context flag (used for string blob detection).
    pub fn context_flag(&self) -> bool {
        (self.flags & FLAG_CONTEXT) != 0
    }

    /// Get the width type (0=bits, 1=multiply, 2=ignore).
    pub fn width_type(&self) -> u8 {
        (self.flags >> 3) & 0x03
    }

    /// Get the width encoding (bits 0-2).
    fn width_encoding(&self) -> u8 {
        self.flags & WIDTH_MASK
    }

    /// Calculate element width in bits.
    ///
    /// Width encoding maps to: 0→0, 1→1, 2→2, 3→4, 4→8, 5→16, 6→32, 7→64
    pub fn width(&self) -> u8 {
        let enc = self.width_encoding();
        if enc == 0 {
            0
        } else {
            1 << (enc - 1)
        }
    }

    /// Calculate payload length in bytes.
    pub fn payload_len(&self) -> usize {
        let width = self.width() as usize;
        let size = self.size as usize;

        match self.width_type() {
            WTYPE_BITS => {
                // Round up: (size * width + 7) / 8
                (size * width + 7) / 8
            }
            WTYPE_MULTIPLY => {
                // Exact: size * width
                size * width
            }
            WTYPE_IGNORE => {
                // Ignore width: size bytes
                size
            }
            _ => 0, // Unknown type
        }
    }

    /// Total node size (header + payload).
    pub fn total_size(&self) -> usize {
        8 + self.payload_len()
    }
}

/// Borrowed view over an array node with zero-copy access.
pub struct ArrayView<'a> {
    header: ArrayHeader,
    payload: &'a [u8],
}

impl<'a> ArrayView<'a> {
    /// Create an ArrayView from file data at offset.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Not enough data for header
    /// - Not enough data for payload
    /// - Checksum is invalid
    pub fn parse(data: &'a [u8], offset: usize) -> Result<Self> {
        if offset + 8 > data.len() {
            return Err(Error::database_corrupted(format!(
                "Array at offset {} exceeds file bounds / 数组偏移 {} 超出文件边界",
                offset, offset
            )));
        }

        let header = ArrayHeader::parse(&data[offset..offset + 8])?;

        // Validate checksum
        if header.checksum != 0x41414141 {
            return Err(Error::database_corrupted(format!(
                "Invalid array checksum: 0x{:08X} (expected 0x41414141) / 无效的数组校验和：0x{:08X}",
                header.checksum, header.checksum
            )));
        }

        let payload_start = offset + 8;
        let payload_end = payload_start + header.payload_len();

        if payload_end > data.len() {
            return Err(Error::database_corrupted(format!(
                "Array payload at offset {} exceeds file bounds / 数组数据在偏移 {} 超出文件边界",
                offset, offset
            )));
        }

        let payload = &data[payload_start..payload_end];

        Ok(ArrayView { header, payload })
    }

    /// Get the array header.
    pub fn header(&self) -> &ArrayHeader {
        &self.header
    }

    /// Get number of elements.
    pub fn len(&self) -> usize {
        self.header.size as usize
    }

    /// Check if array is empty.
    pub fn is_empty(&self) -> bool {
        self.header.size == 0
    }

    /// Get element width in bits.
    pub fn width(&self) -> u8 {
        self.header.width()
    }

    /// Read an integer element at index.
    ///
    /// # Errors
    ///
    /// Returns error if index is out of bounds.
    pub fn get(&self, index: usize) -> Result<i64> {
        if index >= self.len() {
            return Err(Error::other(format!(
                "Array index {} out of bounds (len={}) / 数组索引 {} 越界（长度={}）",
                index, self.len(), index, self.len()
            )));
        }

        let width = self.width();
        let value = self.read_bits_elem(index, width);

        // Sign extend for widths >= 8
        let signed = if width >= 8 && width < 64 {
            // Sign extend: cast to appropriate signed type then to i64
            match width {
                8 => (value as u8) as i8 as i64,
                16 => (value as u16) as i16 as i64,
                32 => (value as u32) as i32 as i64,
                _ => value as i64,
            }
        } else {
            value as i64
        };

        Ok(signed)
    }

    /// Read element as RefOrTagged.
    pub fn get_ref_or_tagged(&self, index: usize) -> Result<RefOrTagged> {
        let value = self.get(index)? as u64;
        Ok(RefOrTagged::parse(value))
    }

    /// Read a bit-packed element.
    fn read_bits_elem(&self, index: usize, width: u8) -> u64 {
        if width == 0 {
            return 0;
        }

        match width {
            1 => {
                // 1-bit values
                let byte_idx = index / 8;
                let bit_idx = index % 8;
                if byte_idx < self.payload.len() {
                    ((self.payload[byte_idx] >> bit_idx) & 1) as u64
                } else {
                    0
                }
            }
            2 | 4 => {
                // 2-bit or 4-bit values
                let bits_per_byte = 8 / width as usize;
                let byte_idx = index / bits_per_byte;
                let sub_idx = index % bits_per_byte;
                let shift = sub_idx * width as usize;
                let mask = (1u64 << width) - 1;

                if byte_idx < self.payload.len() {
                    ((self.payload[byte_idx] >> shift) & mask as u8) as u64
                } else {
                    0
                }
            }
            8 => {
                // 8-bit values
                if index < self.payload.len() {
                    self.payload[index] as u64
                } else {
                    0
                }
            }
            16 => {
                // 16-bit little-endian
                let offset = index * 2;
                if offset + 2 <= self.payload.len() {
                    LittleEndian::read_u16(&self.payload[offset..offset + 2]) as u64
                } else {
                    0
                }
            }
            32 => {
                // 32-bit little-endian
                let offset = index * 4;
                if offset + 4 <= self.payload.len() {
                    LittleEndian::read_u32(&self.payload[offset..offset + 4]) as u64
                } else {
                    0
                }
            }
            64 => {
                // 64-bit little-endian
                let offset = index * 8;
                if offset + 8 <= self.payload.len() {
                    LittleEndian::read_u64(&self.payload[offset..offset + 8])
                } else {
                    0
                }
            }
            _ => 0, // Unknown width
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_header_parse() {
        let mut data = [0u8; 8];
        LittleEndian::write_u32(&mut data[0..4], 0x41414141); // checksum
        data[4] = 0x04; // flags: width_enc=4 → width=8
        data[5] = 0x00;
        data[6] = 0x00;
        data[7] = 0x05; // size = 5

        let header = ArrayHeader::parse(&data).unwrap();
        assert_eq!(header.checksum, 0x41414141);
        assert_eq!(header.flags, 0x04);
        assert_eq!(header.size, 5);
        assert_eq!(header.width(), 8);
    }

    #[test]
    fn array_header_width_calculation() {
        let test_cases = [
            (0, 0),   // enc=0 → width=0
            (1, 1),   // enc=1 → width=1
            (2, 2),   // enc=2 → width=2
            (3, 4),   // enc=3 → width=4
            (4, 8),   // enc=4 → width=8
            (5, 16),  // enc=5 → width=16
            (6, 32),  // enc=6 → width=32
            (7, 64),  // enc=7 → width=64
        ];

        for (enc, expected_width) in test_cases {
            let mut data = [0u8; 8];
            LittleEndian::write_u32(&mut data[0..4], 0x41414141);
            data[4] = enc;
            let header = ArrayHeader::parse(&data).unwrap();
            assert_eq!(header.width(), expected_width, "enc={}", enc);
        }
    }

    #[test]
    fn array_header_flags() {
        let mut data = [0u8; 8];
        LittleEndian::write_u32(&mut data[0..4], 0x41414141);
        data[4] = 0x80 | 0x40 | 0x20; // inner + has_refs + context

        let header = ArrayHeader::parse(&data).unwrap();
        assert!(header.is_inner_node());
        assert!(header.has_refs());
        assert!(header.context_flag());
    }

    #[test]
    fn array_view_parse() {
        // Create array: [10, 20, 30] as 8-bit values
        let mut data = vec![0u8; 8 + 3];
        LittleEndian::write_u32(&mut data[0..4], 0x41414141);
        data[4] = 0x04; // width=8
        data[7] = 0x03; // size=3
        data[8] = 10;
        data[9] = 20;
        data[10] = 30;

        let view = ArrayView::parse(&data, 0).unwrap();
        assert_eq!(view.len(), 3);
        assert_eq!(view.get(0).unwrap(), 10);
        assert_eq!(view.get(1).unwrap(), 20);
        assert_eq!(view.get(2).unwrap(), 30);
    }

    #[test]
    fn array_view_64bit() {
        let mut data = vec![0u8; 8 + 16];
        LittleEndian::write_u32(&mut data[0..4], 0x41414141);
        data[4] = 0x07; // width=64
        data[7] = 0x02; // size=2
        LittleEndian::write_u64(&mut data[8..16], 0x1234567890ABCDEF);
        LittleEndian::write_u64(&mut data[16..24], 0xFEDCBA0987654321);

        let view = ArrayView::parse(&data, 0).unwrap();
        assert_eq!(view.get(0).unwrap() as u64, 0x1234567890ABCDEF);
        assert_eq!(view.get(1).unwrap() as u64, 0xFEDCBA0987654321);
    }

    #[test]
    fn array_view_ref_or_tagged() {
        let mut data = vec![0u8; 8 + 16];
        LittleEndian::write_u32(&mut data[0..4], 0x41414141);
        data[4] = 0x07; // width=64
        data[7] = 0x02; // size=2
        LittleEndian::write_u64(&mut data[8..16], 0x1000); // RealmRef (LSB=0)
        LittleEndian::write_u64(&mut data[16..24], 0x15);  // TaggedValue (LSB=1)

        let view = ArrayView::parse(&data, 0).unwrap();

        let rot0 = view.get_ref_or_tagged(0).unwrap();
        assert!(rot0.is_ref());
        assert_eq!(rot0.as_ref().unwrap().offset(), 0x1000);

        let rot1 = view.get_ref_or_tagged(1).unwrap();
        assert!(rot1.is_tagged());
        assert_eq!(rot1.as_tagged().unwrap().value(), 0x15 >> 1);
    }

    #[test]
    fn array_view_sign_extension() {
        // Test negative 8-bit value
        let mut data = vec![0u8; 8 + 1];
        LittleEndian::write_u32(&mut data[0..4], 0x41414141);
        data[4] = 0x04; // width=8
        data[7] = 0x01; // size=1
        data[8] = 0xFF; // -1 as u8

        let view = ArrayView::parse(&data, 0).unwrap();
        assert_eq!(view.get(0).unwrap(), -1);
    }

    #[test]
    fn array_view_out_of_bounds() {
        let mut data = vec![0u8; 8];
        LittleEndian::write_u32(&mut data[0..4], 0x41414141);
        data[4] = 0x04;
        data[7] = 0x02; // size=2, but no payload

        let view = ArrayView::parse(&data, 0);
        assert!(view.is_err());
    }
}
