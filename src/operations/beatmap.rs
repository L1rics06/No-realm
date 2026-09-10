//! Beatmap 操作模块
//!
//! 提供 Beatmap 的查询和删除功能。
//!
//! # 修改限制
//!
//! - ✅ 查询 Beatmap
//! - ✅ 删除 Beatmap Set
//! - ❌ **禁止** 修改 Beatmap 内容
//! - ❌ **禁止** 创建 Beatmap

use crate::error::{Error, Result};
use crate::models::{BeatmapSetInfo, BeatmapMetadata};
use crate::realm::RealmDatabase;
use crate::safety::safe_operation;
use crate::backup::BackupStrategy;
use uuid::Uuid;
use std::fs;
use std::path::Path;

/// 列出所有 BeatmapSet
///
/// # 参数
///
/// - `db` - 数据库连接
///
/// # 返回
///
/// 所有 BeatmapSet 的列表
///
/// # 错误
///
/// - 数据库访问失败
///
/// # 示例
///
/// ```no_run
/// use no_realm::{RealmDatabase, operations};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let db = RealmDatabase::open("/path/to/client.realm")?;
/// let beatmaps = operations::beatmap::list_all(&db)?;
/// println!("Found {} beatmap sets", beatmaps.len());
/// # Ok(())
/// # }
/// ```
pub fn list_all(db: &RealmDatabase) -> Result<Vec<BeatmapSetInfo>> {
    use crate::realm::format::deserializer::{Deserializer, Value};

    // 检查文件大小
    let size = db.size()?;
    if size < 32 {
        return Ok(Vec::new());
    }

    // 读取 Realm 文件
    let data = fs::read(db.path()).map_err(|e| {
        Error::other(format!("Failed to read realm file: {}", e))
    })?;

    // 解析文件头
    let header = crate::realm::format::RealmHeader::parse(&data)?;

    // 创建反序列化器
    let deser = Deserializer::new(&data, header.active_top_ref() as usize)?;

    // 读取所有 BeatmapSetInfo (table 0)
    let objects = deser.iter_table_objects(0)?;

    // 转换为 BeatmapSetInfo
    let mut beatmap_sets = Vec::new();
    for (_row_index, obj) in objects {
        if let Some(beatmap_set) = object_to_beatmap_set(&obj) {
            beatmap_sets.push(beatmap_set);
        }
    }

    Ok(beatmap_sets)
}

/// 将 RealmObject 转换为 BeatmapSetInfo
fn object_to_beatmap_set(obj: &crate::realm::format::deserializer::RealmObject) -> Option<BeatmapSetInfo> {
    use crate::realm::format::deserializer::Value;

    // 提取 UUID (必需)
    let id = match obj.fields.get("ID") {
        Some(Value::Uuid(uuid)) => {
            // 转换为 Uuid 类型
            Uuid::from_bytes(*uuid)
        }
        _ => return None,
    };

    // 提取 OnlineID
    let online_id = match obj.fields.get("OnlineID") {
        Some(Value::Int(n)) => Some(*n),
        _ => None,
    };

    // 提取 DateAdded
    let date_added = match obj.fields.get("DateAdded") {
        Some(Value::Timestamp(ts)) => {
            // Realm timestamp 是相对于某个 epoch 的纳秒数
            // 转换为 DateTime (简化处理)
            chrono::DateTime::from_timestamp(*ts / 1_000_000_000, (*ts % 1_000_000_000) as u32)
                .unwrap_or_else(|| chrono::Utc::now())
        }
        _ => chrono::Utc::now(),
    };

    Some(BeatmapSetInfo {
        id,
        online_id,
        date_added,
        beatmaps: Vec::new(),  // TODO: 解析 linked beatmaps
        files: Vec::new(),     // TODO: 解析 linked files
        metadata: BeatmapMetadata {
            title: String::new(),
            title_unicode: None,
            artist: String::new(),
            artist_unicode: None,
            author: String::new(),
            author_id: None,
            source: None,
            tags: None,
            audio_file: String::new(),
            background_file: None,
        },
        protected: false,      // TODO: 从数据库读取
        deleted_at: None,      // TODO: 从数据库读取
    })
}

/// 根据 ID 查询 BeatmapSet
///
/// # 参数
///
/// - `db` - 数据库连接
/// - `id` - BeatmapSet 的 UUID
///
/// # 返回
///
/// 如果找到返回 `Some(BeatmapSetInfo)`，否则返回 `None`
pub fn get_by_id(_db: &RealmDatabase, _id: &Uuid) -> Result<Option<BeatmapSetInfo>> {
    // TODO: 实现 Realm 查询
    Ok(None)
}

/// 根据在线 ID 查询 BeatmapSet
///
/// # 参数
///
/// - `db` - 数据库连接
/// - `online_id` - 在线 BeatmapSet ID
pub fn get_by_online_id(_db: &RealmDatabase, _online_id: i64) -> Result<Option<BeatmapSetInfo>> {
    // TODO: 实现 Realm 查询
    Ok(None)
}

/// 搜索 Beatmap
///
/// 根据标题、艺术家、创作者等关键词搜索。
///
/// # 参数
///
/// - `db` - 数据库连接
/// - `query` - 搜索关键词
pub fn search(_db: &RealmDatabase, _query: &str) -> Result<Vec<BeatmapSetInfo>> {
    // TODO: 实现搜索功能
    Ok(Vec::new())
}

/// 删除 BeatmapSet
///
/// **安全操作**: 在备份保护下执行删除。
///
/// # 删除顺序
///
/// 1. 创建数据库备份
/// 2. 从 Realm 删除 BeatmapSet 记录
/// 3. 删除关联的文件
/// 4. 验证操作成功
///
/// 如果任何步骤失败，自动恢复备份。
///
/// # 参数
///
/// - `db` - 数据库连接
/// - `id` - BeatmapSet 的 UUID
///
/// # 错误
///
/// - BeatmapSet 不存在
/// - BeatmapSet 被保护（protected = true）
/// - 数据库操作失败
/// - 文件删除失败
///
/// # 示例
///
/// ```no_run
/// use no_realm::{RealmDatabase, operations};
/// use uuid::Uuid;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let db = RealmDatabase::open("/path/to/client.realm")?;
/// let beatmap_id = Uuid::parse_str("...")?;
///
/// // 安全删除，失败自动恢复
/// operations::beatmap::delete(&db, &beatmap_id)?;
/// # Ok(())
/// # }
/// ```
pub fn delete(db: &RealmDatabase, id: &Uuid) -> Result<()> {
    safe_operation(db.path(), &BackupStrategy::default(), |_path| {
        // TODO: 实现删除逻辑
        // 1. 查询 BeatmapSet
        // 2. 检查是否 protected
        // 3. 删除 Realm 记录
        // 4. 删除文件
        Err(Error::other(format!(
            "Delete beatmap not yet implemented for ID: {}",
            id
        )))
    })
}

/// 软删除 BeatmapSet
///
/// 标记为已删除但不实际删除文件。
///
/// # 参数
///
/// - `db` - 数据库连接
/// - `id` - BeatmapSet 的 UUID
pub fn soft_delete(db: &RealmDatabase, id: &Uuid) -> Result<()> {
    safe_operation(db.path(), &BackupStrategy::default(), |_path| {
        // TODO: 实现软删除
        Err(Error::other(format!(
            "Soft delete not yet implemented for ID: {}",
            id
        )))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_db() -> NamedTempFile {
        NamedTempFile::new().unwrap()
    }

    #[test]
    fn test_list_all_empty() {
        let temp_db = create_test_db();
        let db = RealmDatabase::open(temp_db.path()).unwrap();

        let result = list_all(&db);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_get_by_id_not_found() {
        let temp_db = create_test_db();
        let db = RealmDatabase::open(temp_db.path()).unwrap();

        let id = Uuid::new_v4();
        let result = get_by_id(&db, &id);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_delete_not_implemented() {
        let temp_db = create_test_db();
        let db = RealmDatabase::open(temp_db.path()).unwrap();

        let id = Uuid::new_v4();
        let result = delete(&db, &id);
        assert!(result.is_err());
    }
}
