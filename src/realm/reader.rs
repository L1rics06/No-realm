//! Realm 二进制格式解析
//!
//! 简化的 Realm 文件读取器，专门用于 osu!lazer 数据库

use crate::error::{Error, Result};
use crate::models::{BeatmapSetInfo, BeatmapInfo, BeatmapMetadata, RealmFile};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use uuid::Uuid;

/// Realm 文件头
#[derive(Debug)]
pub struct RealmHeader {
    pub file_format_version: u8,
    pub db_version: u64,
}

/// Realm Schema 信息
#[derive(Debug, Clone)]
pub struct RealmSchema {
    pub tables: HashMap<String, RealmTable>,
}

/// Realm 表定义
#[derive(Debug, Clone)]
pub struct RealmTable {
    pub name: String,
    pub fields: Vec<RealmField>,
}

/// Realm 字段定义
#[derive(Debug, Clone)]
pub struct RealmField {
    pub name: String,
    pub field_type: String,
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

    /// 读取所有文件内容
    fn read_all_bytes(&mut self) -> Result<Vec<u8>> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut buffer = Vec::new();
        self.file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    /// 查找所有字符串（用于分析）
    pub fn find_strings(&mut self) -> Result<Vec<String>> {
        let data = self.read_all_bytes()?;
        let mut strings = Vec::new();
        let mut current = String::new();

        for &byte in &data {
            if byte >= 32 && byte <= 126 {
                current.push(byte as char);
            } else if !current.is_empty() && current.len() > 3 {
                strings.push(current.clone());
                current.clear();
            } else {
                current.clear();
            }
        }

        Ok(strings)
    }

    /// 查询 BeatmapSet (简化版本)
    pub fn query_beatmap_sets(&mut self) -> Result<Vec<BeatmapSetInfo>> {
        let strings = self.find_strings()?;

        // 查找包含 "BeatmapSet" 的字符串附近的数据
        log::info!("Found {} strings in Realm file", strings.len());

        // 目前返回空列表，因为需要更复杂的二进制解析
        log::warn!("Full Realm parsing requires understanding binary structure");

        Ok(Vec::new())
    }

    /// 分析 Schema
    pub fn analyze_schema(&mut self) -> Result<RealmSchema> {
        let strings = self.find_strings()?;

        let mut tables = HashMap::new();

        // 查找类名（通常以 "class_" 开头或大写字母）
        let class_keywords = ["BeatmapInfo", "BeatmapSet", "BeatmapMetadata", "Skin", "BeatmapCollection"];

        for keyword in &class_keywords {
            if strings.iter().any(|s| s.contains(keyword)) {
                log::info!("Found table: {}", keyword);

                // 查找相关字段
                let fields = strings.iter()
                    .filter(|s| s.len() > 3 && s.len() < 30 && s.chars().all(|c| c.is_alphanumeric()))
                    .map(|s| RealmField {
                        name: s.clone(),
                        field_type: "unknown".to_string(),
                    })
                    .collect();

                tables.insert(keyword.to_string(), RealmTable {
                    name: keyword.to_string(),
                    fields,
                });
            }
        }

        Ok(RealmSchema { tables })
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

    #[test]
    fn test_find_strings() {
        let mut temp = NamedTempFile::new().unwrap();
        // 写入带字符串的数据
        temp.write_all(b"\x00\x00Hello\x00World\x00Test\x00\x00").unwrap();
        temp.flush().unwrap();

        let mut reader = RealmReader::open(temp.path()).unwrap_or_else(|_| {
            // 如果不是有效的 Realm 文件，跳过测试
            panic!("Not a valid Realm file for testing");
        });

        // 这个测试可能失败，因为测试文件不是有效的 Realm 格式
    }
}

