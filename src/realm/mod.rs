//! Realm 数据库访问模块
//!
//! 提供 Realm 数据库的打开、读取、写入等基础操作。

use crate::error::{Error, Result};
use std::path::{Path, PathBuf};

mod connection;

pub use connection::RealmDatabase;

/// Realm 数据库配置
#[derive(Debug, Clone)]
pub struct RealmConfig {
    /// 数据库文件路径
    pub path: PathBuf,

    /// 是否只读模式
    pub read_only: bool,
}

impl RealmConfig {
    /// 创建新的配置
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            read_only: false,
        }
    }

    /// 设置为只读模式
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        if !self.path.exists() {
            return Err(Error::FileNotFound {
                path: self.path.clone(),
            });
        }

        if !self.path.is_file() {
            return Err(Error::other(format!("Path is not a file: {:?}", self.path)));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realm_config() {
        let config = RealmConfig::new("/tmp/test.realm");
        assert_eq!(config.path, PathBuf::from("/tmp/test.realm"));
        assert!(!config.read_only);

        let config = config.read_only();
        assert!(config.read_only);
    }
}
