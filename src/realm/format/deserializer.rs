//! Realm 反序列化器 - 简化版本
//!
//! 使用硬编码 Schema 进行对象反序列化

use crate::error::{Error, Result};
use crate::realm::format::{ArrayView, RefOrTagged};
use crate::realm::format::schema::{Schema, TableDef, ColumnType};
use std::collections::HashMap;

/// Realm 反序列化上下文
pub struct Deserializer<'a> {
    data: &'a [u8],
    root_offset: usize,
    schema: Schema,
}

/// Realm 对象
#[derive(Debug, Clone)]
pub struct RealmObject {
    pub table_key: u8,
    pub fields: HashMap<String, Value>,
}

/// Realm 值类型
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Int(i64),
    Bool(bool),
    String(String),
    Binary(Vec<u8>),
    Timestamp(i64),
    Float(f32),
    Double(f64),
    Uuid([u8; 16]),
    Link(u8, u64),
    LinkList(Vec<u64>),
}

impl<'a> Deserializer<'a> {
    pub fn new(data: &'a [u8], root_offset: usize) -> Result<Self> {
        if root_offset >= data.len() {
            return Err(Error::database_corrupted(format!(
                "Root offset out of bounds"
            )));
        }

        Ok(Self {
            data,
            root_offset,
            schema: Schema::osu_lazer_schema(),
        })
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn get_table_root(&self, table_key: u8) -> Result<ArrayView<'a>> {
        let root = ArrayView::parse(self.data, self.root_offset)?;
        let tables_rot = root.get_ref_or_tagged(1)?;

        let tables_ref = match tables_rot {
            RefOrTagged::Ref(r) => r,
            _ => return Err(Error::database_corrupted("Root[1] not a ref")),
        };

        let tables_offset = tables_ref.offset() as usize;
        let tables_array = ArrayView::parse(self.data, tables_offset)?;

        if table_key as usize >= tables_array.len() {
            return Err(Error::other(format!("Table {} out of range", table_key)));
        }

        let table_rot = tables_array.get_ref_or_tagged(table_key as usize)?;
        let table_ref = match table_rot {
            RefOrTagged::Ref(r) => r,
            RefOrTagged::Tagged(_) => return Err(Error::other(format!("Table {} is empty", table_key))),
        };

        ArrayView::parse(self.data, table_ref.offset() as usize)
    }

    pub fn read_string(&self, offset: usize) -> Result<String> {
        if offset == 0 {
            return Ok(String::new());
        }

        if offset >= self.data.len() {
            return Err(Error::database_corrupted("String offset out of bounds"));
        }

        let array = ArrayView::parse(self.data, offset)?;

        if array.width() != 8 && array.width() != 0 {
            return Err(Error::other(format!("Invalid string width: {}", array.width())));
        }

        let mut bytes = Vec::new();
        for i in 0..array.len() {
            let byte = array.get(i)? as u8;
            if byte == 0 { break; }
            bytes.push(byte);
        }

        String::from_utf8(bytes).map_err(|e| Error::other(format!("Invalid UTF-8: {}", e)))
    }

    pub fn count_table_rows(&self, table_key: u8) -> Result<usize> {
        let table_root = self.get_table_root(table_key)?;
        Ok(table_root.len())
    }
}

// Re-export types
pub use crate::realm::format::schema::{Schema as SchemaType, TableDef as TableDefType, ColumnDef, ColumnType as FieldType};
