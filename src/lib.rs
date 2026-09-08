//! # no-realm
//!
//! 一个安全、易用的 osu!lazer Realm 数据库操作库。
//!
//! ## 安全第一
//!
//! 该库的设计原则是：**宁可操作失败，也不能损坏数据库**。
//!
//! 所有写操作都在备份保护下进行，失败时自动恢复。
//!
//! ## 快速开始
//!
//! ```no_run
//! use no_realm::RealmDatabase;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // 打开 osu!lazer 数据库
//! let db = RealmDatabase::open("/path/to/osu/client.realm")?;
//!
//! // 所有操作都会自动备份
//! // 失败时自动恢复
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod backup;
pub mod error;
pub mod hash;
pub mod models;
pub mod operations;
pub mod realm;
pub mod safety;
pub mod sqlite;

pub use error::{Error, Result};
pub use realm::{RealmDatabase, RealmReader};

/// 库的版本号
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
