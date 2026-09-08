//! SQLite 数据库访问层
//!
//! 该模块提供对 Realm 数据库导出的 SQLite 格式的访问。
//!
//! # 使用前提
//!
//! 用户需要先将 `client.realm` 导出为 SQLite 格式：
//! - 使用 Realm Studio 或其他 Realm 工具
//! - 导出为 `client.sqlite`
//!
//! # 限制
//!
//! - 只读访问
//! - 数据可能不同步
//! - 需要手动导出

use crate::error::{Error, Result};
use crate::models::{BeatmapCollection, BeatmapSetInfo, SkinInfo};
use rusqlite::Connection;
use std::path::Path;
use uuid::Uuid;

/// SQLite 数据库连接
pub struct SqliteDatabase {
    conn: Connection,
}

impl SqliteDatabase {
    /// 打开 SQLite 数据库
    ///
    /// # 参数
    ///
    /// - `path` - SQLite 数据库文件路径
    ///
    /// # 错误
    ///
    /// - 文件不存在
    /// - 无法打开数据库
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path.as_ref()).map_err(|e| {
            Error::other(format!("Failed to open SQLite database: {}", e))
        })?;

        Ok(Self { conn })
    }

    /// 获取数据库连接的引用
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

/// 查询所有 BeatmapSet
pub fn query_beatmap_sets(db: &SqliteDatabase) -> Result<Vec<BeatmapSetInfo>> {
    // TODO: 实现查询
    // 需要先分析 SQLite 导出的表结构
    Ok(Vec::new())
}

/// 查询所有皮肤
pub fn query_skins(db: &SqliteDatabase) -> Result<Vec<SkinInfo>> {
    // TODO: 实现查询
    Ok(Vec::new())
}

/// 查询所有收藏夹
pub fn query_collections(db: &SqliteDatabase) -> Result<Vec<BeatmapCollection>> {
    // TODO: 实现查询
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_open_nonexistent() {
        let result = SqliteDatabase::open("/nonexistent/path.sqlite");
        assert!(result.is_err());
    }

    #[test]
    fn test_open_empty_db() {
        let temp_db = NamedTempFile::new().unwrap();
        // SQLite 可以打开空文件并初始化
        let result = SqliteDatabase::open(temp_db.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_open_valid_db() {
        let temp_db = NamedTempFile::new().unwrap();
        // 创建一个有效的空 SQLite 数据库
        {
            let conn = Connection::open(temp_db.path()).unwrap();
            conn.execute("CREATE TABLE test (id INTEGER)", [])
                .unwrap();
        }

        let result = SqliteDatabase::open(temp_db.path());
        assert!(result.is_ok());
    }
}
