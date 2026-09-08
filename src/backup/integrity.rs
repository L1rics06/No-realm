//! 备份完整性验证

use crate::backup::BackupMetadata;
use crate::error::{Error, Result};
use log::{debug, warn};
use std::fs;
use std::path::Path;

/// 备份验证结果
#[derive(Debug, Clone)]
pub struct BackupVerification {
    /// 文件是否存在
    pub exists: bool,

    /// 文件大小是否匹配
    pub size_matches: bool,

    /// SHA256 是否匹配（如果有）
    pub hash_matches: Option<bool>,

    /// Realm 是否可以打开
    ///
    /// 注意：目前这只检查文件是否可读，
    /// 真正的 Realm 验证需要等待 realm-codec 集成
    pub realm_openable: bool,

    /// 关键表是否存在
    ///
    /// 注意：目前这个字段总是 None，
    /// 真正的实现需要等待 realm-codec 集成
    pub tables_exist: Option<bool>,
}

impl BackupVerification {
    /// 备份是否有效
    pub fn is_valid(&self) -> bool {
        self.exists && self.size_matches && self.realm_openable
    }

    /// 获取验证失败的原因
    pub fn failure_reason(&self) -> Option<String> {
        if !self.exists {
            return Some("Backup file does not exist".to_string());
        }

        if !self.size_matches {
            return Some("Backup file size mismatch".to_string());
        }

        if let Some(false) = self.hash_matches {
            return Some("Backup file hash mismatch".to_string());
        }

        if !self.realm_openable {
            return Some("Backup file cannot be opened as Realm database".to_string());
        }

        None
    }
}

/// 验证备份完整性
pub fn verify_backup(metadata: &BackupMetadata) -> Result<BackupVerification> {
    let path = &metadata.backup_path;

    debug!("Verifying backup at: {:?}", path);

    // 1. 检查文件是否存在
    let exists = path.exists();
    if !exists {
        warn!("Backup file does not exist: {:?}", path);
        return Ok(BackupVerification {
            exists: false,
            size_matches: false,
            hash_matches: None,
            realm_openable: false,
            tables_exist: None,
        });
    }

    // 2. 验证文件大小
    let size_matches = match fs::metadata(path) {
        Ok(meta) => meta.len() == metadata.original_size,
        Err(e) => {
            warn!("Failed to get backup file metadata: {}", e);
            false
        }
    };

    if !size_matches {
        warn!(
            "Backup file size mismatch: expected {}, got {:?}",
            metadata.original_size,
            fs::metadata(path).map(|m| m.len())
        );
    }

    // 3. 验证 SHA256（如果有）
    let hash_matches = if let Some(expected_hash) = &metadata.original_sha256 {
        match calculate_sha256(path) {
            Ok(actual_hash) => Some(actual_hash == *expected_hash),
            Err(e) => {
                warn!("Failed to calculate backup hash: {}", e);
                None
            }
        }
    } else {
        None
    };

    // 4. 验证 Realm 可打开性
    // TODO: 使用 realm-codec 尝试打开文件
    // 目前只检查文件是否可读
    let realm_openable = fs::File::open(path).is_ok();

    if !realm_openable {
        warn!("Backup file cannot be opened: {:?}", path);
    }

    // 5. 验证关键表存在
    // TODO: 使用 realm-codec 检查表
    let tables_exist = None;

    Ok(BackupVerification {
        exists,
        size_matches,
        hash_matches,
        realm_openable,
        tables_exist,
    })
}

/// 计算文件的 SHA256 哈希
fn calculate_sha256(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_verify_nonexistent_backup() {
        let metadata = BackupMetadata {
            id: "test".to_string(),
            timestamp: Utc::now(),
            operation: None,
            original_size: 100,
            backup_path: "/nonexistent/backup".into(),
            original_sha256: None,
            verified: false,
            can_restore: false,
        };

        let result = verify_backup(&metadata).unwrap();
        assert!(!result.exists);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_verify_existing_backup() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "test data")?;
        temp_file.flush()?;

        let metadata = BackupMetadata {
            id: "test".to_string(),
            timestamp: Utc::now(),
            operation: None,
            original_size: 9, // "test data".len()
            backup_path: temp_file.path().to_path_buf(),
            original_sha256: None,
            verified: false,
            can_restore: false,
        };

        let result = verify_backup(&metadata)?;
        assert!(result.exists);
        assert!(result.size_matches);
        assert!(result.realm_openable); // 文件可读
        assert!(result.is_valid());

        Ok(())
    }

    #[test]
    fn test_calculate_sha256() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "test")?;
        temp_file.flush()?;

        let hash = calculate_sha256(temp_file.path())?;
        // "test" 的 SHA256
        assert_eq!(
            hash,
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );

        Ok(())
    }

    #[test]
    fn test_verification_failure_reason() {
        let verification = BackupVerification {
            exists: false,
            size_matches: false,
            hash_matches: None,
            realm_openable: false,
            tables_exist: None,
        };

        assert!(verification.failure_reason().is_some());
        assert!(!verification.is_valid());
    }
}
