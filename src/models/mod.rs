//! 数据模型定义
//!
//! 该模块定义了 osu!lazer Realm 数据库中的核心数据结构。
//!
//! # 模型结构
//!
//! - [`RealmFile`] - 文件存储引用（SHA-256 哈希）
//! - [`BeatmapSetInfo`] - Beatmap Set 信息
//! - [`BeatmapInfo`] - 单个 Beatmap 信息
//! - [`SkinInfo`] - 皮肤信息
//! - [`BeatmapCollection`] - 收藏夹信息
//!
//! # 设计说明
//!
//! 这些模型映射到 osu!lazer 的 Realm 数据库结构。
//! 由于 Rust 生态中没有成熟的 Realm 绑定，我们使用简化的结构。
//!
//! **重要**: 当前版本使用简化的内存模型，不直接操作 Realm 对象。
//! 这是为了避免 Realm C++ SDK 的复杂性，并保证跨平台兼容性。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 文件存储引用
///
/// osu!lazer 使用基于内容哈希的文件存储系统：
/// - 文件名 = SHA-256(文件内容)
/// - 存储位置: `files/<hash[0..1]>/<hash[2..3]>/<hash>`
///
/// 这确保了文件去重和内容完整性。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RealmFile {
    /// SHA-256 哈希值（小写十六进制）
    pub hash: String,

    /// 文件大小（字节）
    pub size: u64,
}

impl RealmFile {
    /// 创建新的文件引用
    pub fn new(hash: String, size: u64) -> Self {
        Self { hash, size }
    }

    /// 获取文件在存储中的相对路径
    ///
    /// 例如: `files/ab/cd/abcdef123456...`
    pub fn storage_path(&self) -> String {
        format!(
            "files/{}/{}/{}",
            &self.hash[0..2],
            &self.hash[2..4],
            &self.hash
        )
    }
}

/// Beatmap 元数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BeatmapMetadata {
    /// 标题
    pub title: String,

    /// Unicode 标题
    pub title_unicode: Option<String>,

    /// 艺术家
    pub artist: String,

    /// Unicode 艺术家
    pub artist_unicode: Option<String>,

    /// 来源
    pub source: Option<String>,

    /// 标签（空格分隔）
    pub tags: Option<String>,

    /// 音频文件名
    pub audio_file: String,

    /// 背景文件名
    pub background_file: Option<String>,

    /// 作者 ID
    pub author_id: Option<i64>,

    /// 作者用户名
    pub author: String,
}

/// 单个 Beatmap 信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BeatmapInfo {
    /// Beatmap ID (lazer 内部 UUID)
    pub id: Uuid,

    /// 在线 Beatmap ID
    pub online_id: Option<i64>,

    /// 难度名称 (Difficulty)
    pub difficulty_name: String,

    /// 星级难度
    pub star_rating: f64,

    /// 长度（毫秒）
    pub length: i64,

    /// BPM
    pub bpm: f64,

    /// .osu 文件引用
    pub file: RealmFile,

    /// MD5 哈希（用于在线匹配）
    pub md5_hash: String,

    /// 所属 BeatmapSet 的 ID
    pub beatmap_set_id: Uuid,
}

/// BeatmapSet 信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BeatmapSetInfo {
    /// BeatmapSet ID (lazer 内部 UUID)
    pub id: Uuid,

    /// 在线 BeatmapSet ID
    pub online_id: Option<i64>,

    /// 元数据
    pub metadata: BeatmapMetadata,

    /// 包含的所有 Beatmaps
    pub beatmaps: Vec<BeatmapInfo>,

    /// 所有相关文件（音频、图片、视频等）
    pub files: Vec<RealmFile>,

    /// 添加时间
    pub date_added: DateTime<Utc>,

    /// 是否已删除（软删除）
    pub deleted_at: Option<DateTime<Utc>>,

    /// 是否受保护（不可删除）
    pub protected: bool,
}

/// 皮肤信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkinInfo {
    /// 皮肤 ID (lazer 内部 UUID)
    pub id: Uuid,

    /// 皮肤名称
    pub name: String,

    /// 皮肤创建者
    pub creator: Option<String>,

    /// 实例化信息（JSON 序列化的配置）
    pub instantiation_info: Option<String>,

    /// 皮肤文件列表
    pub files: Vec<RealmFile>,

    /// 创建时间
    pub date_added: DateTime<Utc>,

    /// 是否已删除（软删除）
    pub deleted_at: Option<DateTime<Utc>>,

    /// 哈希值（用于去重）
    pub hash: String,

    /// 是否受保护（不可删除）
    pub protected: bool,
}

impl SkinInfo {
    /// 创建新的皮肤信息
    pub fn new(name: String, creator: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            creator,
            instantiation_info: None,
            files: Vec::new(),
            date_added: Utc::now(),
            deleted_at: None,
            hash: String::new(), // 应该在添加文件后计算
            protected: false,
        }
    }

    /// 检查皮肤是否已删除
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

/// 收藏夹信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BeatmapCollection {
    /// 收藏夹 ID (lazer 内部 UUID)
    pub id: Uuid,

    /// 收藏夹名称
    pub name: String,

    /// 包含的 Beatmap MD5 列表
    ///
    /// **注意**: lazer 使用 MD5 而不是 UUID 来引用收藏夹中的 beatmap。
    /// 这是为了兼容旧版 osu!stable 的收藏夹格式。
    pub beatmap_md5_hashes: Vec<String>,

    /// 创建时间
    pub date_added: DateTime<Utc>,

    /// 最后修改时间
    pub last_modified: DateTime<Utc>,
}

impl BeatmapCollection {
    /// 创建新的收藏夹
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            beatmap_md5_hashes: Vec::new(),
            date_added: now,
            last_modified: now,
        }
    }

    /// 添加 Beatmap 到收藏夹
    pub fn add_beatmap(&mut self, md5_hash: String) {
        if !self.beatmap_md5_hashes.contains(&md5_hash) {
            self.beatmap_md5_hashes.push(md5_hash);
            self.last_modified = Utc::now();
        }
    }

    /// 从收藏夹移除 Beatmap
    pub fn remove_beatmap(&mut self, md5_hash: &str) -> bool {
        if let Some(pos) = self.beatmap_md5_hashes.iter().position(|h| h == md5_hash) {
            self.beatmap_md5_hashes.remove(pos);
            self.last_modified = Utc::now();
            true
        } else {
            false
        }
    }

    /// 检查收藏夹是否包含指定 Beatmap
    pub fn contains_beatmap(&self, md5_hash: &str) -> bool {
        self.beatmap_md5_hashes.contains(&md5_hash.to_string())
    }

    /// 获取收藏夹大小
    pub fn size(&self) -> usize {
        self.beatmap_md5_hashes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realm_file_storage_path() {
        let file = RealmFile::new(
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            1024,
        );

        assert_eq!(
            file.storage_path(),
            "files/ab/cd/abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        );
    }

    #[test]
    fn test_skin_info_creation() {
        let skin = SkinInfo::new("Test Skin".to_string(), Some("Creator".to_string()));

        assert_eq!(skin.name, "Test Skin");
        assert_eq!(skin.creator, Some("Creator".to_string()));
        assert!(!skin.is_deleted());
        assert!(!skin.protected);
        assert!(skin.files.is_empty());
    }

    #[test]
    fn test_collection_operations() {
        let mut collection = BeatmapCollection::new("Favorites".to_string());

        assert_eq!(collection.name, "Favorites");
        assert_eq!(collection.size(), 0);

        // 添加 beatmap
        collection.add_beatmap("hash1".to_string());
        assert_eq!(collection.size(), 1);
        assert!(collection.contains_beatmap("hash1"));

        // 添加重复的 beatmap
        collection.add_beatmap("hash1".to_string());
        assert_eq!(collection.size(), 1);

        // 添加另一个 beatmap
        collection.add_beatmap("hash2".to_string());
        assert_eq!(collection.size(), 2);

        // 移除 beatmap
        assert!(collection.remove_beatmap("hash1"));
        assert_eq!(collection.size(), 1);
        assert!(!collection.contains_beatmap("hash1"));

        // 移除不存在的 beatmap
        assert!(!collection.remove_beatmap("hash999"));
    }
}
