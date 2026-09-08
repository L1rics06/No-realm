//! 备份恢复功能

use crate::error::{Error, Result};
use log::{info, warn};
use std::fs;
use std::path::Path;

/// 恢复备份到目标位置
///
/// # 安全性
///
/// - 会覆盖目标文件
/// - 建议先验证备份的完整性
/// - 会创建一个临时文件，然后原子性地替换
///
/// # 参数
///
/// * `backup_path` - 备份文件路径
/// * `target_path` - 目标文件路径
pub fn restore_backup(backup_path: &Path, target_path: &Path) -> Result<()> {
    info!(
        "Restoring backup from {:?} to {:?}",
        backup_path, target_path
    );

    // 1. 验证备份文件存在
    if !backup_path.exists() {
        return Err(Error::FileNotFound {
            path: backup_path.to_path_buf(),
        });
    }

    // 2. 如果目标文件存在，先备份一份（以防万一）
    if target_path.exists() {
        let emergency_backup = target_path.with_extension("realm.emergency");
        warn!(
            "Target file exists, creating emergency backup: {:?}",
            emergency_backup
        );
        fs::copy(target_path, &emergency_backup)?;
    }

    // 3. 复制备份到目标位置
    // 使用临时文件然后重命名，确保原子性操作
    let temp_target = target_path.with_extension("realm.restoring");

    fs::copy(backup_path, &temp_target)?;

    // 4. 原子性地替换目标文件
    #[cfg(unix)]
    {
        // Unix 上 rename 是原子操作
        fs::rename(&temp_target, target_path)?;
    }

    #[cfg(not(unix))]
    {
        // Windows 上需要先删除目标文件
        if target_path.exists() {
            fs::remove_file(target_path)?;
        }
        fs::rename(&temp_target, target_path)?;
    }

    info!("Backup restored successfully");

    Ok(())
}

/// 恢复备份并验证
///
/// 恢复后会验证目标文件是否与备份一致。
pub fn restore_and_verify(backup_path: &Path, target_path: &Path) -> Result<()> {
    // 恢复
    restore_backup(backup_path, target_path)?;

    // 验证：比较文件大小
    let backup_meta = fs::metadata(backup_path)?;
    let target_meta = fs::metadata(target_path)?;

    if backup_meta.len() != target_meta.len() {
        return Err(Error::restore_failed(format!(
            "Size mismatch after restore: backup={}, target={}",
            backup_meta.len(),
            target_meta.len()
        )));
    }

    // TODO: 可以添加更严格的验证（如哈希比较）

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_restore_backup() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // 创建备份文件
        let backup_path = temp_dir.path().join("backup.realm");
        fs::write(&backup_path, b"backup data")?;

        // 创建目标文件
        let target_path = temp_dir.path().join("target.realm");
        fs::write(&target_path, b"original data")?;

        // 恢复备份
        restore_backup(&backup_path, &target_path)?;

        // 验证
        let content = fs::read(&target_path)?;
        assert_eq!(content, b"backup data");

        Ok(())
    }

    #[test]
    fn test_restore_nonexistent_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_path = temp_dir.path().join("nonexistent.realm");
        let target_path = temp_dir.path().join("target.realm");

        let result = restore_backup(&backup_path, &target_path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::FileNotFound { .. }));
    }

    #[test]
    fn test_restore_to_new_file() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // 创建备份文件
        let backup_path = temp_dir.path().join("backup.realm");
        fs::write(&backup_path, b"backup data")?;

        // 目标文件不存在
        let target_path = temp_dir.path().join("new_target.realm");

        // 恢复备份
        restore_backup(&backup_path, &target_path)?;

        // 验证
        assert!(target_path.exists());
        let content = fs::read(&target_path)?;
        assert_eq!(content, b"backup data");

        Ok(())
    }

    #[test]
    fn test_restore_and_verify() -> Result<()> {
        let temp_dir = TempDir::new()?;

        let backup_path = temp_dir.path().join("backup.realm");
        fs::write(&backup_path, b"test data")?;

        let target_path = temp_dir.path().join("target.realm");

        restore_and_verify(&backup_path, &target_path)?;

        let content = fs::read(&target_path)?;
        assert_eq!(content, b"test data");

        Ok(())
    }
}
