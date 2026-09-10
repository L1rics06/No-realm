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

    /// 迭代表中的所有有效对象（跳过 NULL 和 tagged 值）
    pub fn iter_table_objects(&self, table_key: u8) -> Result<Vec<(usize, RealmObject)>> {
        let row_count = self.count_table_rows(table_key)?;
        let mut objects = Vec::new();

        for row_index in 0..row_count {
            match self.read_object(table_key, row_index) {
                Ok(obj) => objects.push((row_index, obj)),
                Err(_) => {
                    // 跳过 NULL 或 tagged 值
                    continue;
                }
            }
        }

        Ok(objects)
    }

    /// 读取表中的一个对象
    pub fn read_object(&self, table_key: u8, row_index: usize) -> Result<RealmObject> {
        let table_def = self.schema.tables.get(&table_key)
            .ok_or_else(|| Error::other(format!("Table {} not found in schema", table_key)))?;

        let table_root = self.get_table_root(table_key)?;

        if row_index >= table_root.len() {
            return Err(Error::other(format!(
                "Row index {} out of bounds (table has {} rows)",
                row_index, table_root.len()
            )));
        }

        // 获取对象的引用
        let obj_rot = table_root.get_ref_or_tagged(row_index)?;
        let obj_ref = match obj_rot {
            RefOrTagged::Ref(r) => {
                // 检查 NULL 引用
                if r.is_null() {
                    return Err(Error::other(format!("Row {} is null (deleted or empty)", row_index)));
                }
                r
            }
            RefOrTagged::Tagged(_) => {
                // Tagged 值通常表示空槽位或删除标记
                return Err(Error::other(format!("Row {} is not an object (tagged value)", row_index)))
            }
        };

        let obj_offset = obj_ref.offset() as usize;
        let obj_array = ArrayView::parse(self.data, obj_offset)?;

        // 读取每个字段
        let mut fields = HashMap::new();

        for (col_idx, col_def) in table_def.columns.iter().enumerate() {
            if col_idx >= obj_array.len() {
                // 字段不存在，跳过
                continue;
            }

            let value = self.read_field(&obj_array, col_idx, &col_def.column_type)?;
            fields.insert(col_def.name.clone(), value);
        }

        Ok(RealmObject { table_key, fields })
    }

    /// 读取一个字段值
    fn read_field(&self, obj_array: &ArrayView, col_idx: usize, col_type: &ColumnType) -> Result<Value> {
        use ColumnType::*;

        match col_type {
            Int => {
                let val = obj_array.get(col_idx)?;
                Ok(Value::Int(val))
            }
            Bool => {
                let val = obj_array.get(col_idx)?;
                Ok(Value::Bool(val != 0))
            }
            String => {
                let rot = obj_array.get_ref_or_tagged(col_idx)?;
                match rot {
                    RefOrTagged::Ref(r) => {
                        let offset = r.offset() as usize;
                        if offset == 0 {
                            Ok(Value::String(std::string::String::new()))
                        } else {
                            let s = self.read_string(offset)?;
                            Ok(Value::String(s))
                        }
                    }
                    RefOrTagged::Tagged(t) => {
                        if t.value() == 0 {
                            Ok(Value::String(std::string::String::new()))
                        } else {
                            Ok(Value::Null)
                        }
                    }
                }
            }
            Timestamp => {
                let val = obj_array.get(col_idx)?;
                Ok(Value::Timestamp(val))
            }
            Uuid => {
                let rot = obj_array.get_ref_or_tagged(col_idx)?;
                match rot {
                    RefOrTagged::Ref(r) => {
                        let offset = r.offset() as usize;
                        if offset == 0 || offset + 16 > self.data.len() {
                            Ok(Value::Null)
                        } else {
                            let mut uuid = [0u8; 16];
                            uuid.copy_from_slice(&self.data[offset..offset + 16]);
                            Ok(Value::Uuid(uuid))
                        }
                    }
                    RefOrTagged::Tagged(_) => Ok(Value::Null),
                }
            }
            Link { target_table } => {
                let rot = obj_array.get_ref_or_tagged(col_idx)?;
                match rot {
                    RefOrTagged::Ref(r) => {
                        Ok(Value::Link(*target_table, r.offset()))
                    }
                    RefOrTagged::Tagged(_) => Ok(Value::Null),
                }
            }
            LinkList { target_table: _ } => {
                let rot = obj_array.get_ref_or_tagged(col_idx)?;
                match rot {
                    RefOrTagged::Ref(r) => {
                        let list_offset = r.offset() as usize;
                        if list_offset == 0 {
                            Ok(Value::LinkList(Vec::new()))
                        } else {
                            let list_array = ArrayView::parse(self.data, list_offset)?;
                            let mut items = Vec::new();
                            for i in 0..list_array.len() {
                                items.push(list_array.get(i)? as u64);
                            }
                            Ok(Value::LinkList(items))
                        }
                    }
                    RefOrTagged::Tagged(_) => Ok(Value::LinkList(Vec::new())),
                }
            }
            _ => Ok(Value::Null),
        }
    }
}

// Re-export types
pub use crate::realm::format::schema::{Schema as SchemaType, TableDef as TableDefType, ColumnDef, ColumnType as FieldType};
