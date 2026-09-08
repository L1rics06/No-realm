//! 皮肤操作模块
//!
//! 提供皮肤的完整 CRUD 操作。

use crate::backup::BackupStrategy;
use crate::error::{Error, Result};
use crate::models::SkinInfo;
use crate::realm::RealmDatabase;
use crate::safety::safe_operation;
use uuid::Uuid;

/// 列出所有皮肤
pub fn list_all(_db: &RealmDatabase) -> Result<Vec<SkinInfo>> {
    // TODO: 实现 Realm 查询
    Ok(Vec::new())
}

/// 根据 ID 获取皮肤
pub fn get_by_id(_db: &RealmDatabase, _id: &Uuid) -> Result<Option<SkinInfo>> {
    // TODO: 实现 Realm 查询
    Ok(None)
}

/// 根据名称搜索皮肤
pub fn search(_db: &RealmDatabase, _query: &str) -> Result<Vec<SkinInfo>> {
    // TODO: 实现搜索
    Ok(Vec::new())
}

/// 创建新皮肤
///
/// **安全操作**: 在备份保护下执行。
///
/// # 参数
///
/// - `db` - 数据库连接
/// - `name` - 皮肤名称
/// - `creator` - 创建者名称（可选）
///
/// # 返回
///
/// 创建的皮肤信息
pub fn create(db: &RealmDatabase, name: &str, _creator: Option<&str>) -> Result<SkinInfo> {
    safe_operation(db.path(), &BackupStrategy::default(), |_path| {
        // TODO: 实现创建逻辑
        Err(Error::other(format!(
            "Create skin not yet implemented: {}",
            name
        )))
    })
}

/// 更新皮肤信息
///
/// **安全操作**: 在备份保护下执行。
pub fn update(db: &RealmDatabase, skin: &SkinInfo) -> Result<()> {
    safe_operation(db.path(), &BackupStrategy::default(), |_path| {
        // TODO: 实现更新逻辑
        Err(Error::other(format!(
            "Update skin not yet implemented: {}",
            skin.id
        )))
    })
}

/// 删除皮肤
///
/// **安全操作**: 在备份保护下执行删除。
pub fn delete(db: &RealmDatabase, id: &Uuid) -> Result<()> {
    safe_operation(db.path(), &BackupStrategy::default(), |_path| {
        // TODO: 实现删除逻辑
        Err(Error::other(format!(
            "Delete skin not yet implemented: {}",
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
    fn test_create_not_implemented() {
        let temp_db = create_test_db();
        let db = RealmDatabase::open(temp_db.path()).unwrap();

        let result = create(&db, "Test Skin", Some("Creator"));
        assert!(result.is_err());
    }
}
