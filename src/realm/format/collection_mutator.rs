//! Collection 修改器
//!
//! 提供 Collection 的创建、修改和删除功能

use crate::error::{Error, Result};
use crate::realm::format::deserializer::{Deserializer, Value};
use crate::realm::format::array::{ArrayView, ArrayHeader};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

/// Collection 修改器
pub struct CollectionMutator<'a> {
    data: &'a mut Vec<u8>,
    root_offset: usize,
}

impl<'a> CollectionMutator<'a> {
    /// 创建新的修改器
    pub fn new(data: &'a mut Vec<u8>, root_offset: usize) -> Result<Self> {
        Ok(Self { data, root_offset })
    }

    /// 创建新的 Collection
    pub fn create_collection(&mut self, name: &str) -> Result<usize> {
        // 1. 分配字符串空间
        let name_offset = self.allocate_string(name)?;

        // 2. 分配空的 MD5 列表
        let md5_list_offset = self.allocate_empty_list()?;

        // 3. 获取当前时间戳
        let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // 4. 创建对象数组（3个字段：Name, BeatmapMD5Hashes, LastModified）
        let obj_offset = self.allocate_object(&[
            (name_offset, false),       // Name (ref)
            (md5_list_offset, false),   // BeatmapMD5Hashes (ref)
            (timestamp as usize, true), // LastModified (tagged)
        ])?;

        // 5. 添加到 Collection 表
        self.add_to_table(4, obj_offset)?;

        Ok(obj_offset)
    }

    /// 删除 Collection（标记为 NULL）
    pub fn delete_collection(&mut self, row_index: usize) -> Result<()> {
        // 获取 Collection 表
        let deser = Deserializer::new(self.data, self.root_offset)?;
        let table_root = deser.get_table_root(4)?;

        if row_index >= table_root.len() {
            return Err(Error::other(format!("Row {} out of bounds", row_index)));
        }

        // 计算表中该行的偏移
        let table_offset = self.get_table_offset(4)?;
        let table_payload = table_offset + 8;

        // 假设表使用 32-bit width
        let row_offset = table_payload + row_index * 4;

        // 将该行设置为 NULL (0)
        LittleEndian::write_u32(&mut self.data[row_offset..row_offset+4], 0);

        Ok(())
    }

    /// 重命名 Collection
    pub fn rename_collection(&mut self, row_index: usize, new_name: &str) -> Result<()> {
        // 1. 分配新字符串
        let new_name_offset = self.allocate_string(new_name)?;

        // 2. 获取对象偏移
        let deser = Deserializer::new(self.data, self.root_offset)?;
        let table_root = deser.get_table_root(4)?;

        let obj_rot = table_root.get_ref_or_tagged(row_index)?;
        let obj_ref = match obj_rot {
            crate::realm::format::types::RefOrTagged::Ref(r) => r,
            _ => return Err(Error::other("Row is not an object")),
        };

        let obj_offset = obj_ref.offset() as usize;

        // 3. 修改对象的第一个字段（Name）
        let obj_payload = obj_offset + 8;

        // 假设对象使用 16-bit width
        LittleEndian::write_u16(&mut self.data[obj_payload..obj_payload+2], new_name_offset as u16);

        Ok(())
    }

    /// 分配字符串空间
    fn allocate_string(&mut self, s: &str) -> Result<usize> {
        let offset = self.data.len();

        // 写入数组头
        let bytes = s.as_bytes();
        self.write_array_header(bytes.len(), 3)?; // width = 8-bit

        // 写入字符串内容
        self.data.extend_from_slice(bytes);

        Ok(offset)
    }

    /// 分配空列表
    fn allocate_empty_list(&mut self) -> Result<usize> {
        let offset = self.data.len();

        // 写入空数组头
        self.write_array_header(0, 0)?; // 0 elements, 0-bit width

        Ok(offset)
    }

    /// 分配对象
    fn allocate_object(&mut self, fields: &[(usize, bool)]) -> Result<usize> {
        let offset = self.data.len();

        // 确定最小宽度
        let max_value = fields.iter()
            .map(|(v, is_tagged)| if *is_tagged { (*v << 1) | 1 } else { *v })
            .max()
            .unwrap_or(0);

        let width = if max_value <= 0xFFFF { 1 } else { 2 }; // 16-bit or 32-bit

        // 写入数组头
        self.write_array_header(fields.len(), width)?;

        // 写入字段
        for (value, is_tagged) in fields {
            let encoded = if *is_tagged {
                (value << 1) | 1
            } else {
                *value
            };

            if width == 1 {
                self.data.write_u16::<LittleEndian>(encoded as u16)?;
            } else {
                self.data.write_u32::<LittleEndian>(encoded as u32)?;
            }
        }

        Ok(offset)
    }

    /// 写入数组头
    fn write_array_header(&mut self, size: usize, width_enc: u8) -> Result<()> {
        // Checksum
        self.data.write_u32::<LittleEndian>(0x41414141)?;

        // Flags (width encoding in lower 3 bits)
        let flags = 0x40 | (width_enc & 0x07);
        self.data.write_u8(flags)?;

        // Size (3 bytes, big-endian)
        self.data.write_u8(((size >> 16) & 0xFF) as u8)?;
        self.data.write_u8(((size >> 8) & 0xFF) as u8)?;
        self.data.write_u8((size & 0xFF) as u8)?;

        Ok(())
    }

    /// 添加到表
    fn add_to_table(&mut self, table_key: u8, obj_offset: usize) -> Result<()> {
        // 找到表中第一个空槽位（NULL 或 Tagged(0)）
        let deser = Deserializer::new(self.data, self.root_offset)?;
        let table_root = deser.get_table_root(table_key)?;

        for i in 0..table_root.len() {
            let rot = table_root.get_ref_or_tagged(i)?;
            match rot {
                crate::realm::format::types::RefOrTagged::Ref(r) if r.is_null() => {
                    // 找到空槽位，填充
                    let table_offset = self.get_table_offset(table_key)?;
                    let table_payload = table_offset + 8;
                    let row_offset = table_payload + i * 4; // 假设 32-bit width

                    LittleEndian::write_u32(&mut self.data[row_offset..row_offset+4], obj_offset as u32);
                    return Ok(());
                }
                crate::realm::format::types::RefOrTagged::Tagged(t) if t.value() == 0 => {
                    // Tagged(0) 也是空槽位
                    let table_offset = self.get_table_offset(table_key)?;
                    let table_payload = table_offset + 8;
                    let row_offset = table_payload + i * 4;

                    LittleEndian::write_u32(&mut self.data[row_offset..row_offset+4], obj_offset as u32);
                    return Ok(());
                }
                _ => continue,
            }
        }

        Err(Error::other("No empty slot in table"))
    }

    /// 获取表的偏移
    fn get_table_offset(&self, table_key: u8) -> Result<usize> {
        let deser = Deserializer::new(self.data, self.root_offset)?;
        let table_root = deser.get_table_root(table_key)?;

        // 这里需要从 ArrayView 获取偏移，目前暂时返回错误
        Err(Error::other("Table offset extraction not yet implemented"))
    }
}
