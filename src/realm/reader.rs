//! Realm 二进制格式解析
//!
//! 简化的 Realm 文件读取器，专门用于 osu!lazer 数据库

use crate::error::{Error, Result};
use crate::models::BeatmapSetInfo;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Realm 文件头
#[derive(Debug)]
pub struct RealmHeader {
    pub file_format_version: u8,
    pub db_version: u64,
}

/// 简化的 Realm 文件读取器
pub struct RealmReader {
    file: File,
    header: RealmHeader,
}

impl RealmReader {
    /// 打开 Realm 文件
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;

        // 读取文件头
        let header = Self::read_header(&mut file)?;

        Ok(Self { file, header })
    }

    /// 读取 Realm 文件头
    fn read_header(file: &mut File) -> Result<RealmHeader> {
        file.seek(SeekFrom::Start(0))?;

        let mut buffer = [0u8; 32];
        file.read_exact(&mut buffer)?;

        // 检查 "T-DB" 标记
        if &buffer[16..20] != b"T-DB" {
            return Err(Error::other("Invalid Realm file: missing T-DB marker"));
        }

        // 解析版本信息
        let file_format_version = buffer[0];
        let db_version = u64::from_le_bytes(buffer[8..16].try_into().unwrap());

        Ok(RealmHeader {
            file_format_version,
            db_version,
        })
    }

    /// 获取文件格式版本
    pub fn file_format_version(&self) -> u8 {
        self.header.file_format_version
    }

    /// 查询 BeatmapSet (未实现)
    pub fn query_beatmap_sets(&mut self) -> Result<Vec<BeatmapSetInfo>> {
        // Realm 二进制格式较复杂，需要更深入的解析
        // 建议使用 SQLite 导出或等待官方工具
        log::warn!(
            "Realm file format version: {}, parsing not fully implemented",
            self.header.file_format_version
        );
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_invalid_realm_file() {
        let mut temp = NamedTempFile::new().unwrap();
        temp.write_all(b"not a realm file").unwrap();
        temp.flush().unwrap();

        let result = RealmReader::open(temp.path());
        assert!(result.is_err());
    }
}
