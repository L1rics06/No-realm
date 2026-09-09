//! 安全操作模块
//!
//! 提供 RAII 安全守卫和操作验证，确保所有写操作在备份保护下进行。
//!
//! # 核心设计
//!
//! - [`SafetyGuard`] - RAII 守卫，自动管理备份和恢复
//! - [`safe_operation`] - 便捷函数，包装任意操作
//!
//! # 安全保证
//!
//! 1. **操作前备份** - 所有写操作必须通过守卫
//! 2. **失败自动恢复** - Drop 时检测未提交，自动回滚
//! 3. **panic 安全** - 即使 panic 也能恢复备份
//!
//! # 使用示例
//!
//! ```no_run
//! use no_realm::safety::SafetyGuard;
//! use no_realm::backup::BackupStrategy;
//! use no_realm::error::Result;
//! use std::path::Path;
//!
//! # fn main() -> Result<()> {
//! let db_path = Path::new("/path/to/client.realm");
//!
//! // 方式 1: 显式使用守卫
//! {
//!     let mut guard = SafetyGuard::new(db_path, &BackupStrategy::default())?;
//!     // 执行危险操作
//!     perform_write_operation()?;
//!     guard.commit(); // 成功才提交
//! } // Drop 时自动检查，未提交则恢复
//!
//! // 方式 2: 使用便捷函数
//! no_realm::safety::safe_operation(db_path, &BackupStrategy::default(), |_path| {
//!     perform_write_operation()?;
//!     Ok(())
//! })?;
//! # Ok(())
//! # }
//! # fn perform_write_operation() -> Result<()> { Ok(()) }
//! ```

use crate::backup::{Backup, BackupManager, BackupStrategy};
use crate::error::{Error, Result};
use log::{debug, error, info, warn};
use std::path::{Path, PathBuf};

/// RAII 安全守卫
///
/// 自动管理备份和恢复的生命周期：
/// - 创建时：自动创建备份
/// - 提交时：标记操作成功
/// - Drop 时：未提交则自动恢复备份
///
/// # 字段
///
/// - `backup` - 创建的备份实例
/// - `target_path` - 原始数据库路径
/// - `committed` - 是否已提交（成功完成操作）
pub struct SafetyGuard {
    backup: Backup,
    target_path: PathBuf,
    committed: bool,
}

impl SafetyGuard {
    /// 创建新的安全守卫
    ///
    /// # 参数
    ///
    /// - `db_path` - 数据库文件路径
    /// - `strategy` - 备份策略
    ///
    /// # 错误
    ///
    /// - 如果备份创建失败
    /// - 如果路径不存在或不可读
    pub fn new(db_path: &Path, strategy: &BackupStrategy) -> Result<Self> {
        info!("Creating safety guard for: {}", db_path.display());

        let manager = BackupManager::new(db_path)?;
        let mut backup = manager.create_backup(strategy)?;

        // 验证备份
        backup.verify()?;

        debug!("Backup verified, guard created");

        Ok(Self {
            backup,
            target_path: db_path.to_path_buf(),
            committed: false,
        })
    }

    /// 提交操作
    ///
    /// 标记操作成功完成，Drop 时不会恢复备份。
    pub fn commit(&mut self) {
        info!("Operation committed successfully");
        self.committed = true;
    }

    /// 获取备份实例的引用
    pub fn backup(&self) -> &Backup {
        &self.backup
    }

    /// 检查是否已提交
    pub fn is_committed(&self) -> bool {
        self.committed
    }
}

impl Drop for SafetyGuard {
    fn drop(&mut self) {
        if self.committed {
            debug!("Guard dropped, operation was committed");
            return;
        }

        // 未提交 = 操作失败，需要恢复
        warn!("Guard dropped without commit, restoring backup");

        match self.backup.restore(&self.target_path) {
            Ok(()) => {
                info!("Backup restored successfully after failed operation");
            }
            Err(e) => {
                error!("CRITICAL: Failed to restore backup: {}", e);
                error!("Database may be in inconsistent state!");
                error!("Backup location: {}", self.backup.path().display());
            }
        }
    }
}

/// 在备份保护下执行操作
///
/// 这是一个便捷函数，自动管理 SafetyGuard 的生命周期。
///
/// # 参数
///
/// - `db_path` - 数据库文件路径
/// - `strategy` - 备份策略
/// - `operation` - 要执行的操作闭包，接收数据库路径作为参数
///
/// # 返回
///
/// 操作的返回值
///
/// # 错误
///
/// - 备份创建失败
/// - 操作执行失败（会自动恢复备份）
///
/// # 示例
///
/// ```no_run
/// use no_realm::safety::safe_operation;
/// use no_realm::backup::BackupStrategy;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// safe_operation(
///     Path::new("/path/to/client.realm"),
///     &BackupStrategy::default(),
///     |_path| {
///         // 执行写操作
///         println!("Performing safe operation");
///         Ok(())
///     }
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn safe_operation<F, T>(db_path: &Path, strategy: &BackupStrategy, operation: F) -> Result<T>
where
    F: FnOnce(&Path) -> Result<T>,
{
    let mut guard = SafetyGuard::new(db_path, strategy)?;

    // 执行操作
    let result = operation(db_path)?;

    // 成功则提交
    guard.commit();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_db(dir: &Path, name: &str) -> PathBuf {
        let db_path = dir.join(name);
        fs::write(&db_path, b"test database content").unwrap();
        db_path
    }

    #[test]
    fn test_safety_guard_commit() {
        let dir = tempdir().unwrap();
        let db_path = create_test_db(dir.path(), "test.realm");

        let original_content = fs::read(&db_path).unwrap();

        {
            let mut guard = SafetyGuard::new(&db_path, &BackupStrategy::default()).unwrap();
            assert!(!guard.is_committed());

            // 修改数据库
            fs::write(&db_path, b"modified content").unwrap();

            // 提交
            guard.commit();
            assert!(guard.is_committed());
        }

        // 提交后，修改应该保留
        let final_content = fs::read(&db_path).unwrap();
        assert_eq!(final_content, b"modified content");
        assert_ne!(final_content, original_content);
    }

    #[test]
    fn test_safety_guard_auto_restore() {
        let dir = tempdir().unwrap();
        let db_path = create_test_db(dir.path(), "test.realm");

        let original_content = fs::read(&db_path).unwrap();

        {
            let _guard = SafetyGuard::new(&db_path, &BackupStrategy::default()).unwrap();

            // 修改数据库
            fs::write(&db_path, b"modified content").unwrap();

            // 不提交，直接 drop
        }

        // 未提交，应该自动恢复
        let final_content = fs::read(&db_path).unwrap();
        assert_eq!(final_content, original_content);
    }

    #[test]
    fn test_safe_operation_success() {
        let dir = tempdir().unwrap();
        let db_path = create_test_db(dir.path(), "test.realm");

        let result = safe_operation(&db_path, &BackupStrategy::default(), |path| {
            fs::write(path, b"safe operation")?;
            Ok(42)
        });

        assert_eq!(result.unwrap(), 42);

        let content = fs::read(&db_path).unwrap();
        assert_eq!(content, b"safe operation");
    }

    #[test]
    fn test_safe_operation_failure() {
        let dir = tempdir().unwrap();
        let db_path = create_test_db(dir.path(), "test.realm");

        let original_content = fs::read(&db_path).unwrap();

        let result: Result<()> = safe_operation(&db_path, &BackupStrategy::default(), |path| {
            fs::write(path, b"will be reverted")?;
            Err(Error::operation_aborted("intentional failure"))
        });

        assert!(result.is_err());

        // 失败后应该恢复
        let final_content = fs::read(&db_path).unwrap();
        assert_eq!(final_content, original_content);
    }

    #[test]
    #[should_panic(expected = "test panic")]
    fn test_safety_guard_panic_recovery() {
        let dir = tempdir().unwrap();
        let db_path = create_test_db(dir.path(), "test.realm");

        let original_content = fs::read(&db_path).unwrap();

        let result = std::panic::catch_unwind(|| {
            let _guard = SafetyGuard::new(&db_path, &BackupStrategy::default()).unwrap();
            fs::write(&db_path, b"panic content").unwrap();
            panic!("test panic");
        });

        assert!(result.is_err());

        // panic 后应该恢复
        let final_content = fs::read(&db_path).unwrap();
        assert_eq!(final_content, original_content);

        panic!("test panic"); // 重新抛出以满足 should_panic
    }
}
