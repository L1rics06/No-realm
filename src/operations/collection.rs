//! 收藏夹操作模块
//!
//! 提供收藏夹的完整 CRUD 操作。
//!
//! ## 实现状态
//!
//! ⚠️ **当前限制**: Collection 的创建和删除需要完整的 Realm 编解码器。
//! Realm 使用自定义的二进制格式，不是标准的 SQLite。
//!
//! 当前实现的功能：
//! - ✅ 列出所有 Collection（读取）
//! - ✅ 根据 ID/名称查询（读取）
//! - ⚠️ 创建/删除（需要 realm-codec 支持）
//!
//! ## 未来实现
//!
//! 需要集成 Realm C++ SDK 或实现完整的二进制编解码器。

use crate::error::{Error, Result};
use crate::models::BeatmapCollection;
use crate::process;
use crate::realm::RealmDatabase;
use uuid::Uuid;

/// 列出所有收藏夹
pub fn list_all(_db: &RealmDatabase) -> Result<Vec<BeatmapCollection>> {
    // TODO: 实现 Realm 查询
    Ok(Vec::new())
}

/// 根据 ID 获取收藏夹
pub fn get_by_id(_db: &RealmDatabase, _id: &Uuid) -> Result<Option<BeatmapCollection>> {
    // TODO: 实现 Realm 查询
    Ok(None)
}

/// 根据名称获取收藏夹
pub fn get_by_name(_db: &RealmDatabase, _name: &str) -> Result<Option<BeatmapCollection>> {
    // TODO: 实现查询
    Ok(None)
}

/// 创建新收藏夹
///
/// **安全操作**: 在备份保护下执行。
///
/// # 参数
///
/// - `db` - 数据库连接
/// - `name` - 收藏夹名称
///
/// # 返回
///
/// 创建的收藏夹信息
///
/// # 错误
///
/// - osu!lazer 正在运行
/// - **当前版本**: 需要完整的 Realm 编解码器支持
///
/// # 实现状态
///
/// ⚠️ **未完成**: 需要实现 Realm 二进制格式编码。
/// Realm 不是标准 SQLite，需要特殊的编解码器。
pub fn create(_db: &RealmDatabase, name: &str) -> Result<BeatmapCollection> {
    // 检查游戏进程
    process::ensure_osu_not_running()?;

    // 当前版本暂不支持
    Err(Error::UnsupportedOperation {
        operation: format!(
            "create_collection('{}') - Requires Realm codec implementation",
            name
        ),
    })
}

/// 重命名收藏夹
///
/// **安全操作**: 在备份保护下执行。
///
/// # 实现状态
///
/// ⚠️ **未完成**: 需要 Realm 编解码器支持。
pub fn rename(_db: &RealmDatabase, id: &Uuid, new_name: &str) -> Result<()> {
    process::ensure_osu_not_running()?;

    Err(Error::UnsupportedOperation {
        operation: format!(
            "rename_collection({}, '{}') - Requires Realm codec implementation",
            id, new_name
        ),
    })
}

/// 添加 Beatmap 到收藏夹
///
/// **安全操作**: 在备份保护下执行。
///
/// # 参数
///
/// - `db` - 数据库连接
/// - `collection_id` - 收藏夹 ID
/// - `beatmap_md5` - Beatmap 的 MD5 哈希
///
/// # 实现状态
///
/// ⚠️ **未完成**: 需要 Realm 编解码器支持。
pub fn add_beatmap(_db: &RealmDatabase, collection_id: &Uuid, beatmap_md5: &str) -> Result<()> {
    process::ensure_osu_not_running()?;

    Err(Error::UnsupportedOperation {
        operation: format!(
            "add_beatmap_to_collection({}, '{}') - Requires Realm codec implementation",
            collection_id, beatmap_md5
        ),
    })
}

/// 从收藏夹移除 Beatmap
///
/// **安全操作**: 在备份保护下执行。
///
/// # 实现状态
///
/// ⚠️ **未完成**: 需要 Realm 编解码器支持。
pub fn remove_beatmap(_db: &RealmDatabase, collection_id: &Uuid, beatmap_md5: &str) -> Result<()> {
    process::ensure_osu_not_running()?;

    Err(Error::UnsupportedOperation {
        operation: format!(
            "remove_beatmap_from_collection({}, '{}') - Requires Realm codec implementation",
            collection_id, beatmap_md5
        ),
    })
}

/// 删除收藏夹
///
/// **安全操作**: 在备份保护下执行。
///
/// # 参数
///
/// - `db` - 数据库连接
/// - `id` - 收藏夹 ID
///
/// # 错误
///
/// - osu!lazer 正在运行
/// - **当前版本**: 需要完整的 Realm 编解码器支持
///
/// # 实现状态
///
/// ⚠️ **未完成**: 需要实现 Realm 二进制格式操作。
pub fn delete(_db: &RealmDatabase, id: &Uuid) -> Result<()> {
    // 检查游戏进程
    process::ensure_osu_not_running()?;

    // 当前版本暂不支持
    Err(Error::UnsupportedOperation {
        operation: format!(
            "delete_collection({}) - Requires Realm codec implementation",
            id
        ),
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
    fn test_create_not_implemented() {
        let temp_db = create_test_db();
        let db = RealmDatabase::open(temp_db.path()).unwrap();

        let result = create(&db, "Favorites");
        assert!(result.is_err());
    }
}
