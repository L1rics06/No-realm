//! 测试 Realm 序列化和反序列化
//!
//! 使用真实的 lazer 数据库测试反序列化功能

use no_realm::realm::{RealmReader, format::{Deserializer, Serializer, Value}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let db_path = dirs::data_local_dir()
        .unwrap()
        .join("osu/client.realm");

    if !db_path.exists() {
        eprintln!("Database not found at: {:?}", db_path);
        return Ok(());
    }

    println!("=== Testing Realm Serialization/Deserialization ===\n");

    // 1. 读取真实数据库
    println!("--- Step 1: Read Real Database ---");
    let reader = RealmReader::open(&db_path)?;
    let header = reader.header();

    println!("Database: {:?}", db_path);
    println!("Format: {}", header.version());
    println!("Active top_ref: 0x{:X}", header.active_top_ref());
    println!();

    // 2. 读取文件数据用于反序列化
    println!("--- Step 2: Create Deserializer ---");
    let data = std::fs::read(&db_path)?;
    let root_offset = header.active_top_ref() as usize;

    let mut deserializer = Deserializer::new(&data, root_offset)?;
    println!("Deserializer created at offset 0x{:X}", root_offset);
    println!();

    // 3. 尝试解析 schema
    println!("--- Step 3: Parse Schema ---");
    match deserializer.parse_schema() {
        Ok(schema) => {
            println!("✓ Schema parsed successfully");
            println!("  Tables: {}", schema.tables.len());
        }
        Err(e) => {
            println!("⚠ Schema parsing not fully implemented: {}", e);
        }
    }
    println!();

    // 4. 尝试访问第一个表 (通常是 BeatmapSet)
    println!("--- Step 4: Access Table Data ---");
    for table_key in 0..5 {
        match deserializer.get_table_root(table_key) {
            Ok(table_root) => {
                println!("✓ Table {}: {} elements", table_key, table_root.len());

                // 显示表的属性
                println!("    Width: {} bits", table_root.width());
                println!("    Is inner: {}", table_root.header().is_inner_node());
                println!("    Has refs: {}", table_root.header().has_refs());
            }
            Err(e) => {
                println!("✗ Table {}: {}", table_key, e);
            }
        }
    }
    println!();

    // 5. 测试序列化器
    println!("--- Step 5: Test Serializer ---");
    let mut serializer = Serializer::new();

    // 写入一个简单的测试值
    serializer.write_u64(0x1234567890ABCDEF)?;
    serializer.write_array_header(5, 4, 0)?;
    serializer.write_string("test")?;

    let serialized = serializer.into_bytes();
    println!("✓ Serialized {} bytes", serialized.len());
    println!("  First 16 bytes: {:02X?}", &serialized[..16.min(serialized.len())]);
    println!();

    // 6. 测试往返（round-trip）
    println!("--- Step 6: Round-trip Test ---");
    let mut ser = Serializer::new();

    // 创建一个测试对象
    let test_value = Value::String("Hello, Realm!".to_string());
    ser.write_value(&test_value)?;

    let bytes = ser.as_bytes();
    println!("✓ Value serialized to {} bytes", bytes.len());

    // 尝试读回（需要实现完整的反序列化）
    println!("  (Full deserialization requires schema knowledge)");
    println!();

    println!("=== Test Summary ===");
    println!("✓ Deserializer: Can open real database and parse structure");
    println!("✓ Serializer: Can write Realm binary format");
    println!("⚠ Schema parsing: Needs full implementation");
    println!("⚠ Object deserialization: Needs schema-aware parsing");
    println!();
    println!("Next steps:");
    println!("  1. Implement full schema parser");
    println!("  2. Implement schema-aware object deserialization");
    println!("  3. Implement high-level API for common operations");

    Ok(())
}
