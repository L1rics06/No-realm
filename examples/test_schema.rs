//! 测试硬编码 Schema 功能

use no_realm::realm::{RealmReader, format::{Deserializer, Schema}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = dirs::data_local_dir().unwrap().join("osu/client.realm");

    if !db_path.exists() {
        eprintln!("Database not found");
        return Ok(());
    }

    println!("=== 测试 Schema 和基础反序列化 ===\n");

    // 1. 打开数据库
    let reader = RealmReader::open(&db_path)?;
    let header = reader.header();

    println!("数据库: {:?}", db_path);
    println!("版本: {}", header.version());
    println!();

    // 2. 创建反序列化器
    let data = std::fs::read(&db_path)?;
    let root_offset = header.active_top_ref() as usize;
    let deserializer = Deserializer::new(&data, root_offset)?;

    // 3. 查看 Schema
    let schema = deserializer.schema();
    println!("=== Schema 信息 ===");
    println!("表数量: {}", schema.tables.len());
    println!();

    for (key, table) in schema.tables.iter() {
        println!("表 {}: {}", key, table.name);
        println!("  列数: {}", table.columns.len());
        for (i, col) in table.columns.iter().enumerate() {
            println!("    [{}] {} ({:?}) {}",
                i,
                col.name,
                col.column_type,
                if col.nullable { "nullable" } else { "" }
            );
        }
        println!();
    }

    // 4. 统计每个表的行数
    println!("=== 表数据统计 ===");
    for key in 0..5 {
        match deserializer.count_table_rows(key) {
            Ok(count) => {
                let table_name = schema.get_table(key)
                    .map(|t| t.name.as_str())
                    .unwrap_or("未知");
                println!("表 {} ({}): {} 行", key, table_name, count);
            }
            Err(e) => {
                println!("表 {}: 错误 - {}", key, e);
            }
        }
    }
    println!();

    // 5. 测试字符串读取
    println!("=== 字符串读取测试 ===");

    // 从表 2 (BeatmapMetadata) 尝试读取
    if let Ok(table_root) = deserializer.get_table_root(2) {
        println!("BeatmapMetadata 表:");
        println!("  根数组元素: {}", table_root.len());
        println!("  宽度: {} bits", table_root.width());

        // 尝试读取前几个元素作为引用
        for i in 0..table_root.len().min(3) {
            if let Ok(rot) = table_root.get_ref_or_tagged(i) {
                match rot {
                    no_realm::realm::format::RefOrTagged::Ref(r) => {
                        println!("  [{}] Ref: 0x{:X}", i, r.offset());

                        // 尝试读取为字符串
                        if let Ok(s) = deserializer.read_string(r.offset() as usize) {
                            if !s.is_empty() && s.len() < 100 {
                                println!("      → 字符串: \"{}\"", s);
                            }
                        }
                    }
                    no_realm::realm::format::RefOrTagged::Tagged(t) => {
                        println!("  [{}] Tagged: {}", i, t.value());
                    }
                }
            }
        }
    }

    println!("\n=== 完成 ===");
    Ok(())
}
