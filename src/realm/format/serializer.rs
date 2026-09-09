//! Realm 序列化器
//!
//! 将 Rust 对象序列化为 Realm 二进制格式

use crate::error::{Error, Result};
use crate::realm::format::deserializer::{RealmObject, Value, FieldType};
use byteorder::{ByteOrder, LittleEndian};

/// Realm 序列化器
///
/// 负责将 Rust 对象序列化为 Realm 二进制格式
pub struct Serializer {
    /// 输出缓冲区
    buffer: Vec<u8>,

    /// 当前写入位置
    position: usize,
}

impl Serializer {
    /// 创建新的序列化器
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            position: 0,
        }
    }

    /// 创建带初始容量的序列化器
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            position: 0,
        }
    }

    /// 获取序列化后的数据
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }

    /// 获取当前缓冲区引用
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// 写入 u8
    pub fn write_u8(&mut self, value: u8) -> Result<()> {
        self.buffer.push(value);
        self.position += 1;
        Ok(())
    }

    /// 写入 u16 (小端)
    pub fn write_u16(&mut self, value: u16) -> Result<()> {
        let mut buf = [0u8; 2];
        LittleEndian::write_u16(&mut buf, value);
        self.buffer.extend_from_slice(&buf);
        self.position += 2;
        Ok(())
    }

    /// 写入 u32 (小端)
    pub fn write_u32(&mut self, value: u32) -> Result<()> {
        let mut buf = [0u8; 4];
        LittleEndian::write_u32(&mut buf, value);
        self.buffer.extend_from_slice(&buf);
        self.position += 4;
        Ok(())
    }

    /// 写入 u64 (小端)
    pub fn write_u64(&mut self, value: u64) -> Result<()> {
        let mut buf = [0u8; 8];
        LittleEndian::write_u64(&mut buf, value);
        self.buffer.extend_from_slice(&buf);
        self.position += 8;
        Ok(())
    }

    /// 写入 i64 (小端)
    pub fn write_i64(&mut self, value: i64) -> Result<()> {
        let mut buf = [0u8; 8];
        LittleEndian::write_i64(&mut buf, value);
        self.buffer.extend_from_slice(&buf);
        self.position += 8;
        Ok(())
    }

    /// 写入 f32 (小端)
    pub fn write_f32(&mut self, value: f32) -> Result<()> {
        let mut buf = [0u8; 4];
        LittleEndian::write_f32(&mut buf, value);
        self.buffer.extend_from_slice(&buf);
        self.position += 4;
        Ok(())
    }

    /// 写入 f64 (小端)
    pub fn write_f64(&mut self, value: f64) -> Result<()> {
        let mut buf = [0u8; 8];
        LittleEndian::write_f64(&mut buf, value);
        self.buffer.extend_from_slice(&buf);
        self.position += 8;
        Ok(())
    }

    /// 写入数组头 (8 字节)
    ///
    /// # 参数
    /// - `size`: 元素数量
    /// - `width_encoding`: 宽度编码 (0-7)
    /// - `flags`: 标志字节
    pub fn write_array_header(&mut self, size: u32, width_encoding: u8, flags: u8) -> Result<()> {
        // Checksum: 0x41414141
        self.write_u32(0x41414141)?;

        // Flags
        self.write_u8(flags | (width_encoding & 0x07))?;

        // Size (3 bytes, big-endian order in the bytes)
        self.write_u8((size >> 16) as u8)?;
        self.write_u8((size >> 8) as u8)?;
        self.write_u8(size as u8)?;

        Ok(())
    }

    /// 写入 Realm 对象
    ///
    /// 将对象的字段按顺序写入
    pub fn write_object(&mut self, obj: &RealmObject) -> Result<()> {
        // TODO: 完整实现需要根据 schema 确定字段顺序和类型

        // 按字段索引排序
        let mut fields: Vec<_> = obj.fields.iter().collect();
        fields.sort_by_key(|(idx, _)| *idx);

        for (_, value) in fields {
            self.write_value(value)?;
        }

        Ok(())
    }

    /// 写入 Realm 值
    pub fn write_value(&mut self, value: &Value) -> Result<()> {
        match value {
            Value::Null => {
                // Null 通常表示为 0
                self.write_u64(0)?;
            }
            Value::Int(v) => {
                self.write_i64(*v)?;
            }
            Value::Bool(v) => {
                self.write_u8(if *v { 1 } else { 0 })?;
            }
            Value::String(s) => {
                self.write_string(s)?;
            }
            Value::Data(bytes) => {
                self.write_binary(bytes)?;
            }
            Value::Date(ts) => {
                self.write_i64(*ts)?;
            }
            Value::Float(v) => {
                self.write_f32(*v)?;
            }
            Value::Double(v) => {
                self.write_f64(*v)?;
            }
            Value::Uuid(bytes) => {
                self.buffer.extend_from_slice(bytes);
                self.position += 16;
            }
            Value::ObjectRef(table_key, offset) => {
                // 对象引用存储为 ref
                self.write_u64(*offset)?;
            }
            Value::List(items) => {
                // 列表需要写入数组结构
                self.write_list(items)?;
            }
        }

        Ok(())
    }

    /// 写入字符串
    ///
    /// Realm 字符串存储格式：
    /// - 短字符串 (<=15字节): 可能内联
    /// - 长字符串: 存储为字节数组
    pub fn write_string(&mut self, s: &str) -> Result<()> {
        let bytes = s.as_bytes();

        // 写入数组头 (width=8 表示字节数组)
        self.write_array_header(bytes.len() as u32, 4, 0)?;

        // 写入字节
        self.buffer.extend_from_slice(bytes);
        self.position += bytes.len();

        Ok(())
    }

    /// 写入二进制数据
    fn write_binary(&mut self, data: &[u8]) -> Result<()> {
        // 类似字符串，存储为字节数组
        self.write_array_header(data.len() as u32, 4, 0)?;
        self.buffer.extend_from_slice(data);
        self.position += data.len();

        Ok(())
    }

    /// 写入列表
    fn write_list(&mut self, items: &[Value]) -> Result<()> {
        // TODO: 根据元素类型确定最佳的数组编码
        // 现在简单地按 64-bit 存储
        self.write_array_header(items.len() as u32, 7, 0x40)?; // width=64, has_refs

        for item in items {
            self.write_value(item)?;
        }

        Ok(())
    }

    /// 对齐到 8 字节边界
    ///
    /// Realm 要求所有数组节点和引用都是 8 字节对齐的
    pub fn align_to_8(&mut self) -> Result<()> {
        let remainder = self.position % 8;
        if remainder != 0 {
            let padding = 8 - remainder;
            for _ in 0..padding {
                self.write_u8(0)?;
            }
        }
        Ok(())
    }

    /// 获取当前位置（作为潜在的引用偏移）
    pub fn position(&self) -> usize {
        self.position
    }

    /// 预留空间并返回偏移量
    ///
    /// 用于稍后回填数据
    pub fn reserve(&mut self, size: usize) -> usize {
        let offset = self.position;
        self.buffer.resize(self.buffer.len() + size, 0);
        self.position += size;
        offset
    }

    /// 在指定偏移量写入 u64
    pub fn write_u64_at(&mut self, offset: usize, value: u64) -> Result<()> {
        if offset + 8 > self.buffer.len() {
            return Err(Error::other(format!(
                "Write offset {} + 8 exceeds buffer size {} / 写入偏移越界",
                offset, self.buffer.len()
            )));
        }

        LittleEndian::write_u64(&mut self.buffer[offset..offset + 8], value);
        Ok(())
    }
}

impl Default for Serializer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serializer_primitives() {
        let mut ser = Serializer::new();

        ser.write_u8(0x42).unwrap();
        ser.write_u16(0x1234).unwrap();
        ser.write_u32(0x12345678).unwrap();
        ser.write_u64(0x123456789ABCDEF0).unwrap();

        let bytes = ser.into_bytes();
        assert_eq!(bytes[0], 0x42);
        assert_eq!(&bytes[1..3], &[0x34, 0x12]); // little-endian
    }

    #[test]
    fn test_array_header() {
        let mut ser = Serializer::new();
        ser.write_array_header(10, 4, 0x40).unwrap();

        let bytes = ser.as_bytes();
        assert_eq!(bytes.len(), 8);
        assert_eq!(&bytes[0..4], &[0x41, 0x41, 0x41, 0x41]); // checksum
        assert_eq!(bytes[4], 0x44); // flags | width_encoding
        assert_eq!(bytes[7], 10); // size (low byte)
    }

    #[test]
    fn test_align_to_8() {
        let mut ser = Serializer::new();
        ser.write_u8(0x42).unwrap(); // position = 1

        assert_eq!(ser.position(), 1);
        ser.align_to_8().unwrap();
        assert_eq!(ser.position(), 8);
        assert_eq!(ser.as_bytes().len(), 8);
    }

    #[test]
    fn test_string_serialization() {
        let mut ser = Serializer::new();
        ser.write_string("hello").unwrap();

        let bytes = ser.as_bytes();
        // 8 bytes header + 5 bytes data
        assert_eq!(bytes.len(), 13);

        // Check array header
        assert_eq!(&bytes[0..4], &[0x41, 0x41, 0x41, 0x41]);

        // Check string content
        assert_eq!(&bytes[8..13], b"hello");
    }
}
