//! Schema 解析器
//!
//! 基于 Realm v24 格式解析 Schema 定义

use crate::error::{Error, Result};
use crate::realm::format::{ArrayView, RefOrTagged};
use std::collections::HashMap;

/// Realm Schema
#[derive(Debug, Clone)]
pub struct Schema {
    /// 表定义 (table_key -> TableDef)
    pub tables: HashMap<u8, TableDef>,

    /// 表名到索引的映射
    pub table_names: HashMap<String, u8>,
}

/// 表定义
#[derive(Debug, Clone)]
pub struct TableDef {
    /// 表索引
    pub key: u8,

    /// 表名
    pub name: String,

    /// 字段定义
    pub columns: Vec<ColumnDef>,
}

/// 列/字段定义
#[derive(Debug, Clone)]
pub struct ColumnDef {
    /// 字段名
    pub name: String,

    /// 字段类型
    pub column_type: ColumnType,

    /// 是否可空
    pub nullable: bool,

    /// 是否是主键
    pub is_primary_key: bool,
}

/// Realm 列类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    /// 整数 (int64)
    Int,
    /// 布尔值
    Bool,
    /// 字符串
    String,
    /// 二进制数据
    Binary,
    /// 时间戳
    Timestamp,
    /// 浮点数 (float)
    Float,
    /// 双精度浮点数 (double)
    Double,
    /// 对象引用 (link to another table)
    Link { target_table: u8 },
    /// 对象列表 (linklist)
    LinkList { target_table: u8 },
    /// UUID (16 bytes)
    Uuid,
}

impl Schema {
    /// 创建空 Schema
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            table_names: HashMap::new(),
        }
    }

    /// 添加表定义
    pub fn add_table(&mut self, table: TableDef) {
        self.table_names.insert(table.name.clone(), table.key);
        self.tables.insert(table.key, table);
    }

    /// 通过索引获取表
    pub fn get_table(&self, key: u8) -> Option<&TableDef> {
        self.tables.get(&key)
    }

    /// 通过名称获取表
    pub fn get_table_by_name(&self, name: &str) -> Option<&TableDef> {
        self.table_names.get(name)
            .and_then(|key| self.tables.get(key))
    }

    /// 创建 osu!lazer 的硬编码 Schema
    ///
    /// 基于 osu!lazer 的 Realm 模型定义
    /// https://github.com/ppy/osu/tree/master/osu.Game/Database
    pub fn osu_lazer_schema() -> Self {
        let mut schema = Self::new();

        // Table 0: BeatmapSetInfo
        schema.add_table(TableDef {
            key: 0,
            name: "BeatmapSetInfo".to_string(),
            columns: vec![
                ColumnDef {
                    name: "ID".to_string(),
                    column_type: ColumnType::Uuid,
                    nullable: false,
                    is_primary_key: true,
                },
                ColumnDef {
                    name: "OnlineID".to_string(),
                    column_type: ColumnType::Int,
                    nullable: false,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "DateAdded".to_string(),
                    column_type: ColumnType::Timestamp,
                    nullable: false,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "Beatmaps".to_string(),
                    column_type: ColumnType::LinkList { target_table: 1 },
                    nullable: false,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "Files".to_string(),
                    column_type: ColumnType::LinkList { target_table: 5 },
                    nullable: false,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "Status".to_string(),
                    column_type: ColumnType::Int,
                    nullable: false,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "DeletePending".to_string(),
                    column_type: ColumnType::Bool,
                    nullable: false,
                    is_primary_key: false,
                },
            ],
        });

        // Table 1: BeatmapInfo
        schema.add_table(TableDef {
            key: 1,
            name: "BeatmapInfo".to_string(),
            columns: vec![
                ColumnDef {
                    name: "ID".to_string(),
                    column_type: ColumnType::Uuid,
                    nullable: false,
                    is_primary_key: true,
                },
                ColumnDef {
                    name: "DifficultyName".to_string(),
                    column_type: ColumnType::String,
                    nullable: false,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "StarRating".to_string(),
                    column_type: ColumnType::Double,
                    nullable: false,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "Length".to_string(),
                    column_type: ColumnType::Double,
                    nullable: false,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "BPM".to_string(),
                    column_type: ColumnType::Double,
                    nullable: false,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "Hash".to_string(),
                    column_type: ColumnType::String,
                    nullable: false,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "Metadata".to_string(),
                    column_type: ColumnType::Link { target_table: 2 },
                    nullable: false,
                    is_primary_key: false,
                },
            ],
        });

        // Table 2: BeatmapMetadata
        schema.add_table(TableDef {
            key: 2,
            name: "BeatmapMetadata".to_string(),
            columns: vec![
                ColumnDef {
                    name: "ID".to_string(),
                    column_type: ColumnType::Uuid,
                    nullable: false,
                    is_primary_key: true,
                },
                ColumnDef {
                    name: "Title".to_string(),
                    column_type: ColumnType::String,
                    nullable: false,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "TitleUnicode".to_string(),
                    column_type: ColumnType::String,
                    nullable: true,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "Artist".to_string(),
                    column_type: ColumnType::String,
                    nullable: false,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "ArtistUnicode".to_string(),
                    column_type: ColumnType::String,
                    nullable: true,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "Author".to_string(),
                    column_type: ColumnType::String,
                    nullable: false,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "Source".to_string(),
                    column_type: ColumnType::String,
                    nullable: true,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "Tags".to_string(),
                    column_type: ColumnType::String,
                    nullable: true,
                    is_primary_key: false,
                },
            ],
        });

        // Table 4: BeatmapCollection
        schema.add_table(TableDef {
            key: 4,
            name: "BeatmapCollection".to_string(),
            columns: vec![
                ColumnDef {
                    name: "ID".to_string(),
                    column_type: ColumnType::Uuid,
                    nullable: false,
                    is_primary_key: true,
                },
                ColumnDef {
                    name: "Name".to_string(),
                    column_type: ColumnType::String,
                    nullable: false,
                    is_primary_key: false,
                },
                ColumnDef {
                    name: "BeatmapMD5Hashes".to_string(),
                    column_type: ColumnType::String, // 实际是 List<string>
                    nullable: false,
                    is_primary_key: false,
                },
            ],
        });

        // Table 5: RealmFile
        schema.add_table(TableDef {
            key: 5,
            name: "RealmFile".to_string(),
            columns: vec![
                ColumnDef {
                    name: "Hash".to_string(),
                    column_type: ColumnType::String,
                    nullable: false,
                    is_primary_key: true,
                },
            ],
        });

        schema
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self::new()
    }
}

impl TableDef {
    /// 获取字段索引
    pub fn get_column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.name == name)
    }

    /// 获取字段定义
    pub fn get_column(&self, name: &str) -> Option<&ColumnDef> {
        self.columns.iter().find(|c| c.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osu_schema() {
        let schema = Schema::osu_lazer_schema();

        assert!(schema.get_table_by_name("BeatmapSetInfo").is_some());
        assert!(schema.get_table_by_name("BeatmapInfo").is_some());
        assert!(schema.get_table_by_name("BeatmapMetadata").is_some());

        let beatmap_set = schema.get_table(0).unwrap();
        assert_eq!(beatmap_set.name, "BeatmapSetInfo");
        assert!(beatmap_set.columns.len() > 0);
    }
}
