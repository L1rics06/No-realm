//! 备份模块
//!
//! 提供数据库备份、验证和恢复功能。
//!
//! # 核心原则
//!
//! - 每次写操作前都应该创建备份
//! - 备份必须经过验证才能使用
//! - 失败时自动恢复备份
//!
//! # 示例
//!
//! ```no_run
//! use no_realm::RealmDatabase;
//! use no_realm::backup::{BackupManager, BackupStrategy};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let db = RealmDatabase::open("/path/to/client.realm")?;
//! let manager = BackupManager::new(db.path())?;
//!
//! // 创建备份
//! let backup = manager.create_backup(&BackupStrategy::default())?;
//!
//! // 验证备份
//! backup.verify()?;
//!
//! // 如果需要，恢复备份
//! // backup.restore(db.path())?;
//! # Ok(())
//! # }
//! ```

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

mod integrity;
mod restore;
mod strategy;

pub use integrity::BackupVerification;
pub use restore::restore_backup;
pub use strategy::BackupStrategy;

/// 备份元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// 备份 ID
    pub id: String,

    /// 创建时间
    pub timestamp: DateTime<Utc>,

    /// 操作类型（可选）
    pub operation: Option<String>,

    /// 原始文件大小（字节）
    pub original_size: u64,

    /// 备份文件路径
    pub backup_path: PathBuf,

    /// 原始文件 SHA256
    pub original_sha256: Option<String>,

    /// 是否已验证
    pub verified: bool,

    /// 是否可以恢复
    pub can_restore: bool,
}

/// 备份对象
#[derive(Debug)]
pub struct Backup {
    metadata: BackupMetadata,
}

impl Backup {
    /// 创建新的备份
    fn new(source: &Path, backup_path: PathBuf, operation: Option<String>) -> Result<Self> {
        let metadata = std::fs::metadata(source)?;

        let backup_metadata = BackupMetadata {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            operation,
            original_size: metadata.len(),
            backup_path,
            original_sha256: None, // 可选：计算哈希
            verified: false,
            can_restore: false,
        };

        Ok(Self {
            metadata: backup_metadata,
        })
    }

    /// 获取备份 ID
    pub fn id(&self) -> &str {
        &self.metadata.id
    }

    /// 获取备份路径
    pub fn path(&self) -> &Path {
        &self.metadata.backup_path
    }

    /// 获取元数据
    pub fn metadata(&self) -> &BackupMetadata {
        &self.metadata
    }

    /// 验证备份完整性
    ///
    /// 检查：
    /// - 文件是否存在
    /// - 文件大小是否匹配
    /// - 文件是否可读
    pub fn verify(&mut self) -> Result<BackupVerification> {
        debug!("Verifying backup: {}", self.id());

        let verification = integrity::verify_backup(&self.metadata)?;

        // 更新元数据
        self.metadata.verified = verification.is_valid();
        self.metadata.can_restore = verification.realm_openable;

        if verification.is_valid() {
            info!("Backup verification passed: {}", self.id());
        } else {
            warn!("Backup verification failed: {}", self.id());
        }

        Ok(verification)
    }

    /// 恢复备份到指定路径
    ///
    /// # 安全性
    ///
    /// - 会覆盖目标文件
    /// - 建议先验证备份
    pub fn restore(&self, target: &Path) -> Result<()> {
        if !self.metadata.can_restore {
            return Err(Error::other(
                "Backup cannot be restored (not verified or invalid)",
            ));
        }

        restore::restore_backup(&self.metadata.backup_path, target)
    }
}

/// 备份管理器
#[derive(Debug)]
pub struct BackupManager {
    /// 源数据库路径
    source_path: PathBuf,

    /// 备份目录
    backup_dir: PathBuf,

    /// 备份清单文件路径
    manifest_path: PathBuf,
}

impl BackupManager {
    /// 创建新的备份管理器
    ///
    /// # 参数
    ///
    /// * `source_path` - 要备份的数据库文件路径
    pub fn new<P: AsRef<Path>>(source_path: P) -> Result<Self> {
        let source_path = source_path.as_ref().to_path_buf();

        if !source_path.exists() {
            return Err(Error::FileNotFound { path: source_path });
        }

        // 备份目录在源文件同级目录下
        let backup_dir = source_path
            .parent()
            .ok_or_else(|| Error::other("Cannot determine parent directory"))?
            .join("backups");

        let manifest_path = backup_dir.join("manifest.json");

        Ok(Self {
            source_path,
            backup_dir,
            manifest_path,
        })
    }

    /// 创建备份
    pub fn create_backup(&self, strategy: &BackupStrategy) -> Result<Backup> {
        // 确保备份目录存在
        if !self.backup_dir.exists() {
            fs::create_dir_all(&self.backup_dir)?;
            info!("Created backup directory: {:?}", self.backup_dir);
        }

        let timestamp = Utc::now();
        let id = Uuid::new_v4();

        // 生成备份文件名
        let backup_filename = format!(
            "client.realm.backup.{}.{}",
            timestamp.format("%Y%m%dT%H%M%SZ"),
            &id.to_string()[..8]
        );

        let backup_path = self.backup_dir.join(backup_filename);

        info!("Creating backup: {:?}", backup_path);

        // 复制文件
        fs::copy(&self.source_path, &backup_path)?;

        // 创建备份对象
        let mut backup = Backup::new(&self.source_path, backup_path, None)?;

        // 如果策略要求，验证备份
        if strategy.verify_after_backup {
            backup.verify()?;
        }

        // 清理旧备份
        self.cleanup_old_backups(strategy.retain_count)?;

        // 保存清单（TODO）

        Ok(backup)
    }

    /// 清理旧备份
    fn cleanup_old_backups(&self, retain_count: usize) -> Result<()> {
        if !self.backup_dir.exists() {
            return Ok(());
        }

        // 获取所有备份文件
        let mut backups: Vec<_> = fs::read_dir(&self.backup_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("client.realm.backup.")
            })
            .collect();

        // 按修改时间排序（最新的在前）
        backups.sort_by_key(|entry| {
            entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });
        backups.reverse();

        // 删除超过保留数量的备份
        if backups.len() > retain_count {
            for entry in backups.iter().skip(retain_count) {
                let path = entry.path();
                info!("Removing old backup: {:?}", path);
                fs::remove_file(path)?;
            }
        }

        Ok(())
    }

    /// 列出所有备份
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }

        let backups: Vec<PathBuf> = fs::read_dir(&self.backup_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("client.realm.backup.")
            })
            .map(|entry| entry.path())
            .collect();

        Ok(backups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_backup_manager_creation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("client.realm");
        fs::write(&db_path, b"test data")?;

        let manager = BackupManager::new(&db_path)?;
        assert_eq!(manager.source_path, db_path);

        Ok(())
    }

    #[test]
    fn test_create_backup() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("client.realm");
        fs::write(&db_path, b"test data")?;

        let manager = BackupManager::new(&db_path)?;
        let strategy = BackupStrategy::default();

        let backup = manager.create_backup(&strategy)?;

        assert!(backup.path().exists());
        assert_eq!(fs::read(backup.path())?, b"test data");

        Ok(())
    }

    #[test]
    fn test_backup_cleanup() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("client.realm");
        fs::write(&db_path, b"test data")?;

        let manager = BackupManager::new(&db_path)?;
        let mut strategy = BackupStrategy::default();
        strategy.retain_count = 2;

        // 创建 3 个备份
        for _ in 0..3 {
            manager.create_backup(&strategy)?;
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let backups = manager.list_backups()?;
        assert_eq!(backups.len(), 2, "Should only keep 2 backups");

        Ok(())
    }
}
