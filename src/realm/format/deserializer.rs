//! Realm 反序列化器
//!
//! 将 Realm 二进制数据反序列化为 Rust 对象

use crate::error::{Error, Result};
use crate::realm::format::{ArrayView, RefOrTagged, RealmRef};
use std::collections::HashMap;

/// Realm 反序列化上下文
///
/// 持有文件数据和解析状态，提供类型安全的反序列化接口
pub struct Deserializer<'a> {
    /// 完整的文件数据
    data: &'a [u8],

    /// 根节点偏移
    root_offset: usize,

    /// Schema 信息缓存
    schema: Option<Schema>,
}

/// Realm Schema 定义
#[derive(Debug, Clone)]
pub struct Schema {
    /// 表定义映射 (table_key -> TableDef)
    pub tables: HashMap<u8, TableDef>,
}

/// 表定义
#[derive(Debug, Clone)]
pub struct TableDef {
    /// 表索引 (0-based)
    pub index: u8,

    /// 表名 (如 "class_BeatmapInfo")
    pub name: String,

    /// 字段定义
    pub fields: Vec<FieldDef>,
}

/// 字段定义
#[derive(Debug, Clone)]
pub struct FieldDef {
    /// 字段名
    pub name: String,

    /// 字段类型
    pub field_type: FieldType,

    /// 是否可空
    pub nullable: bool,
}

/// Realm 字段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    /// 整数
    Int,
    /// 布尔值
    Bool,
    /// 字符串
    String,
    /// 二进制数据
    Data,
    /// 日期时间
    Date,
    /// 浮点数
    Float,
    /// 双精度浮点数
    Double,
    /// UUID
    Uuid,
    /// 对象引用 (link)
    Object(u8), // table_key
    /// 对象列表 (linklist)
    List(u8), // table_key
}

/// Realm 对象 - 反序列化的结果
#[derive(Debug, Clone)]
pub struct RealmObject {
    /// 表索引
    pub table_key: u8,

    /// 字段值映射 (field_index -> Value)
    pub fields: HashMap<usize, Value>,
}

/// Realm 值类型
#[derive(Debug, Clone)]
pub enum Value {
    /// 空值
    Null,
    /// 整数
    Int(i64),
    /// 布尔值
    Bool(bool),
    /// 字符串
    String(String),
    /// 二进制数据
    Data(Vec<u8>),
    /// 日期时间 (Unix 时间戳)
    Date(i64),
    /// 浮点数
    Float(f32),
    /// 双精度浮点数
    Double(f64),
    /// UUID
    Uuid([u8; 16]),
    /// 对象引用 (table_key, object_offset)
    ObjectRef(u8, u64),
    /// 对象列表
    List(Vec<Value>),
}

impl<'a> Deserializer<'a> {
    /// 创建新的反序列化器
    ///
    /// # 参数
    /// - `data`: 完整的 Realm 文件数据
    /// - `root_offset`: 根节点的偏移量 (从 RealmHeader::active_top_ref() 获取)
    pub fn new(data: &'a [u8], root_offset: usize) -> Result<Self> {
        if root_offset >= data.len() {
            return Err(Error::database_corrupted(format!(
                "Root offset 0x{:X} exceeds file size {} / 根偏移超出文件大小",
                root_offset, data.len()
            )));
        }

        Ok(Self {
            data,
            root_offset,
            schema: None,
        })
    }

    /// 解析 Schema
    pub fn parse_schema(&mut self) -> Result<&Schema> {
        if self.schema.is_some() {
            return Ok(self.schema.as_ref().unwrap());
        }

        // 解析根数组
        let root = ArrayView::parse(self.data, self.root_offset)?;

        // 根数组布局 (Realm v24):
        // [0] = schema ref
        // [1] = tables ref
        // [2] = file size (tagged)
        // [3+] = 其他元数据

        if root.len() < 2 {
            return Err(Error::database_corrupted(
                "Root array too small / 根数组太小".to_string()
            ));
        }

        // 获取 schema ref
        let schema_rot = root.get_ref_or_tagged(0)?;
        let schema_ref = match schema_rot {
            RefOrTagged::Ref(r) => r,
            _ => return Err(Error::database_corrupted(
                "Root[0] is not a ref / 根[0]不是引用".to_string()
            )),
        };

        // 解析 schema 数组
        let schema_offset = schema_ref.offset() as usize;
        let schema_array = ArrayView::parse(self.data, schema_offset)?;

        // TODO: 完整的 Schema 解析需要理解 Realm 的 schema 编码格式
        // 现在先创建一个占位符
        let schema = Schema {
            tables: HashMap::new(),
        };

        self.schema = Some(schema);
        Ok(self.schema.as_ref().unwrap())
    }

    /// 获取表的根节点
    ///
    /// # 参数
    /// - `table_key`: 表索引 (0-based)
    pub fn get_table_root(&self, table_key: u8) -> Result<ArrayView<'a>> {
        // 解析根数组
        let root = ArrayView::parse(self.data, self.root_offset)?;

        // 获取 tables ref (root[1])
        let tables_rot = root.get_ref_or_tagged(1)?;
        let tables_ref = match tables_rot {
            RefOrTagged::Ref(r) => r,
            _ => return Err(Error::database_corrupted(
                "Root[1] is not a ref / 根[1]不是引用".to_string()
            )),
        };

        // 解析 tables 数组
        let tables_offset = tables_ref.offset() as usize;
        let tables_array = ArrayView::parse(self.data, tables_offset)?;

        // 获取指定表的 ref
        if table_key as usize >= tables_array.len() {
            return Err(Error::other(format!(
                "Table key {} out of range (max {}) / 表索引越界",
                table_key, tables_array.len()
            )));
        }

        let table_rot = tables_array.get_ref_or_tagged(table_key as usize)?;
        let table_ref = match table_rot {
            RefOrTagged::Ref(r) => r,
            RefOrTagged::Tagged(t) => {
                // Tagged 值表示空表
                return Err(Error::other(format!(
                    "Table {} is empty (tagged value {}) / 表为空",
                    table_key, t.value()
                )));
            }
        };

        // 解析表的根数组
        let table_offset = table_ref.offset() as usize;
        ArrayView::parse(self.data, table_offset)
    }

    /// 从数组中读取对象
    ///
    /// Realm 对象在数组中以连续字段的形式存储
    pub fn read_object(&self, array: &ArrayView<'a>, index: usize, table_key: u8) -> Result<RealmObject> {
        // TODO: 实现完整的对象反序列化
        // 需要根据 schema 知道每个字段的类型和位置

        let mut fields = HashMap::new();

        // 占位实现：读取原始值
        if index < array.len() {
            let value = array.get(index)?;
            fields.insert(0, Value::Int(value));
        }

        Ok(RealmObject {
            table_key,
            fields,
        })
    }

    /// 读取字符串
    ///
    /// Realm 字符串存储为：
    /// - 短字符串 (<=15字节): 内联存储
    /// - 长字符串: 存储为数组引用
    pub fn read_string(&self, offset: usize) -> Result<String> {
        if offset >= self.data.len() {
            return Err(Error::database_corrupted(format!(
                "String offset 0x{:X} out of bounds / 字符串偏移越界",
                offset
            )));
        }

        // 尝试解析为数组
        let array = ArrayView::parse(self.data, offset)?;

        // 字符串数组的 width 通常是 8 (字节)
        if array.width() != 8 {
            return Err(Error::other(format!(
                "Invalid string array width: {} / 无效的字符串数组宽度",
                array.width()
            )));
        }

        // 读取字节
        let mut bytes = Vec::with_capacity(array.len());
        for i in 0..array.len() {
            let byte = array.get(i)? as u8;
            bytes.push(byte);
        }

        // 转换为字符串
        String::from_utf8(bytes).map_err(|e| {
            Error::other(format!("Invalid UTF-8 in string: {} / 无效的 UTF-8", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realm::format::RealmHeader;

    fn create_minimal_realm() -> Vec<u8> {
        // 创建最小的测试 Realm 文件
        let mut data = Vec::new();

        // Header (24 bytes)
        data.extend_from_slice(&0x1000u64.to_le_bytes()); // top_ref_0
        data.extend_from_slice(&0x2000u64.to_le_bytes()); // top_ref_1
        data.extend_from_slice(b"T-DB");
        data.extend_from_slice(&24u16.to_le_bytes());
        data.push(0); // reserved
        data.push(0); // flags

        data
    }

    #[test]
    fn test_deserializer_creation() {
        let data = create_minimal_realm();
        let result = Deserializer::new(&data, 0x1000);

        // 因为 0x1000 超出了数据大小，应该失败
        assert!(result.is_err());
    }
}
