//! 错误类型定义
//!
//! 提供中英双语错误信息。

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// no-realm 库的统一错误类型
#[derive(Error, Debug)]
pub enum Error {
    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Realm 数据库错误
    #[error("Realm database error: {message}\n数据库错误：{message}")]
    Realm {
        /// 错误信息
        message: String,
    },

    /// 备份错误
    #[error("Backup error: {message}\n备份错误：{message}")]
    Backup {
        /// 错误信息
        message: String,
    },

    /// 备份验证失败
    #[error("Backup verification failed: {reason}\n备份验证失败：{reason}")]
    BackupVerificationFailed {
        /// 失败原因
        reason: String,
    },

    /// 恢复失败
    #[error("Restore failed: {message}\n恢复失败：{message}")]
    RestoreFailed {
        /// 错误信息
        message: String,
    },

    /// 文件哈希不匹配
    #[error(
        "Hash mismatch for file {path:?}: expected {expected}, got {actual}\n\
         文件哈希不匹配 {path:?}：期望 {expected}，实际 {actual}"
    )]
    HashMismatch {
        /// 文件路径
        path: PathBuf,
        /// 期望的哈希值
        expected: String,
        /// 实际的哈希值
        actual: String,
    },

    /// 文件未找到
    #[error("File not found: {path:?}\n文件未找到：{path:?}")]
    FileNotFound {
        /// 文件路径
        path: PathBuf,
    },

    /// 文件引用未找到
    #[error("File reference not found: {hash}\n文件引用未找到：{hash}")]
    FileReferenceNotFound {
        /// 文件哈希
        hash: String,
    },

    /// 完整性检查失败
    #[error("Integrity check failed: {details}\n完整性检查失败：{details}")]
    IntegrityCheckFailed {
        /// 详细信息
        details: String,
    },

    /// 数据库已损坏
    #[error(
        "Database corrupted: {reason}\n数据库已损坏：{reason}\n\
         Please restore from backup / 请从备份恢复"
    )]
    DatabaseCorrupted {
        /// 损坏原因
        reason: String,
    },

    /// 操作被中止
    #[error("Operation aborted: {reason}\n操作被中止：{reason}")]
    OperationAborted {
        /// 中止原因
        reason: String,
    },

    /// 不支持的操作
    #[error("Unsupported operation: {operation}\n不支持的操作：{operation}")]
    UnsupportedOperation {
        /// 操作名称
        operation: String,
    },

    /// Schema 版本不兼容
    #[error(
        "Incompatible schema version: expected {expected}, got {actual}\n\
         Schema 版本不兼容：期望 {expected}，实际 {actual}"
    )]
    IncompatibleSchemaVersion {
        /// 期望的版本
        expected: String,
        /// 实际的版本
        actual: String,
    },

    /// 通用错误
    #[error("{message}")]
    Other {
        /// 错误信息
        message: String,
    },
}

/// 统一的 Result 类型
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// 创建一个 Realm 错误
    pub fn realm<S: Into<String>>(message: S) -> Self {
        Self::Realm {
            message: message.into(),
        }
    }

    /// 创建一个备份错误
    pub fn backup<S: Into<String>>(message: S) -> Self {
        Self::Backup {
            message: message.into(),
        }
    }

    /// 创建一个恢复失败错误
    pub fn restore_failed<S: Into<String>>(message: S) -> Self {
        Self::RestoreFailed {
            message: message.into(),
        }
    }

    /// 创建一个操作被中止错误
    pub fn operation_aborted<S: Into<String>>(reason: S) -> Self {
        Self::OperationAborted {
            reason: reason.into(),
        }
    }

    /// 创建一个通用错误
    pub fn other<S: Into<String>>(message: S) -> Self {
        Self::Other {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::HashMismatch {
            path: PathBuf::from("/test/file"),
            expected: "abc123".to_string(),
            actual: "def456".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("abc123"));
        assert!(msg.contains("def456"));
    }

    #[test]
    fn test_error_constructors() {
        let err = Error::realm("test error");
        assert!(matches!(err, Error::Realm { .. }));

        let err = Error::backup("backup failed");
        assert!(matches!(err, Error::Backup { .. }));
    }
}
