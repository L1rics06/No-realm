//! Realm 数据库连接管理
//!
//! 使用 realm-codec 进行底层文件操作。

use crate::error::{Error, Result};
use crate::realm::RealmConfig;
use log::{debug, info, warn};
use std::path::{Path, PathBuf};

/// Realm 数据库连接
///
/// 这是操作 osu!lazer Realm 数据库的主要入口点。
///
/// # 安全性
///
/// - 所有写操作都应该通过 `safety::safe_operation` 进行
/// - 直接使用此类型的写操作不会自动备份
///
/// # 示例
///
/// ```no_run
/// use no_realm::RealmDatabase;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let db = RealmDatabase::open("/path/to/client.realm")?;
/// // 进行操作...
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct RealmDatabase {
    config: RealmConfig,
    // TODO: 添加 realm-codec 的实际连接
    // realm: Option<RealmFile>,
}

impl RealmDatabase {
    /// 打开 Realm 数据库
    ///
    /// # 参数
    ///
    /// * `path` - 数据库文件路径
    ///
    /// # 错误
    ///
    /// - 文件不存在
    /// - 文件损坏
    /// - 权限不足
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = RealmConfig::new(path);
        config.validate()?;

        info!("Opening Realm database: {:?}", config.path);

        // TODO: 使用 realm-codec 打开文件
        // 目前先实现基础结构

        Ok(Self {
            config,
            // realm: None,
        })
    }

    /// 以只读模式打开数据库
    pub fn open_read_only<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = RealmConfig::new(path).read_only();
        config.validate()?;

        info!("Opening Realm database (read-only): {:?}", config.path);

        Ok(Self { config })
    }

    /// 使用配置打开数据库
    pub fn open_with_config(config: RealmConfig) -> Result<Self> {
        config.validate()?;

        info!("Opening Realm database with config: {:?}", config);

        Ok(Self { config })
    }

    /// 获取数据库文件路径
    pub fn path(&self) -> &Path {
        &self.config.path
    }

    /// 是否为只读模式
    pub fn is_read_only(&self) -> bool {
        self.config.read_only
    }

    /// 检查数据库是否可以打开
    ///
    /// 这是一个轻量级的检查，不会加载整个数据库。
    pub fn can_open(&self) -> bool {
        // TODO: 尝试打开 realm-codec
        self.config.path.exists() && self.config.path.is_file()
    }

    /// 获取数据库文件大小（字节）
    pub fn size(&self) -> Result<u64> {
        let metadata = std::fs::metadata(&self.config.path)?;
        Ok(metadata.len())
    }

    /// 关闭数据库
    ///
    /// Rust 的 RAII 会自动清理资源，但显式调用可以更早释放。
    pub fn close(self) {
        info!("Closing Realm database: {:?}", self.config.path);
        drop(self);
    }
}

// 实现 Drop trait 以确保资源正确清理
impl Drop for RealmDatabase {
    fn drop(&mut self) {
        debug!("Dropping RealmDatabase: {:?}", self.config.path);
        // TODO: 清理 realm-codec 资源
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_open_nonexistent_file() {
        let result = RealmDatabase::open("/nonexistent/path/to/file.realm");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::FileNotFound { .. }));
    }

    #[test]
    fn test_config_validation() {
        let config = RealmConfig::new("/nonexistent/file.realm");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_open_empty_file() -> Result<()> {
        // 创建一个临时文件
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "test")?;
        temp_file.flush()?;

        let path = temp_file.path();

        // 目前这会成功，因为我们还没有实际尝试解析 realm 文件
        let db = RealmDatabase::open(path)?;
        assert_eq!(db.path(), path);
        assert!(!db.is_read_only());

        Ok(())
    }

    #[test]
    fn test_read_only_mode() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "test")?;
        temp_file.flush()?;

        let db = RealmDatabase::open_read_only(temp_file.path())?;
        assert!(db.is_read_only());

        Ok(())
    }
}
