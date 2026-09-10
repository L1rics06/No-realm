//! 读取 osu!lazer 数据库中的 Beatmap 信息
//!
//! 用于测试和验证反序列化功能

use no_realm::{RealmDatabase, Result};
use no_realm::realm::format::deserializer::Deserializer;
use std::env;

fn main() -> Result<()> {
    env_logger::init();

    let db_path = env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example read_beatmaps <path-to-client.realm>");
            std::process::exit(1);
        });

    println!("Opening database: {}", db_path);
    let db = RealmDatabase::open(&db_path)?;

    println!("Database size: {} bytes", db.size()?);

    // 读取 Realm 文件
    let data = std::fs::read(db.path())?;
    println!("Read {} bytes from file", data.len());

    // 解析文件头
    let header = no_realm::realm::format::RealmHeader::parse(&data)?;
    println!("\n=== Realm Header ===");
    println!("Version: {:?}", header.version());
    println!("Active top_ref: 0x{:x}", header.active_top_ref());

    // 创建反序列化器
    let deser = Deserializer::new(&data, header.active_top_ref() as usize)?;
    println!("\n=== Schema Info ===");
    println!("Table count: {}", deser.schema().tables.len());

    // 列出所有表
    for (key, table) in &deser.schema().tables {
        println!("  Table {}: {} ({} columns)", key, table.name, table.columns.len());
    }

    // 尝试读取 BeatmapSet 表 (假设 key = 1)
    println!("\n=== BeatmapSet Table ===");
    match deser.count_table_rows(1) {
        Ok(count) => {
            println!("Found {} BeatmapSet records", count);

            // 尝试读取第一个 BeatmapSet
            if count > 0 {
                println!("\nReading first BeatmapSet...");
                match read_first_beatmapset(&deser) {
                    Ok(()) => println!("Successfully read first BeatmapSet"),
                    Err(e) => println!("Error reading BeatmapSet: {}", e),
                }
            }
        }
        Err(e) => println!("Error counting rows: {}", e),
    }

    Ok(())
}

fn read_first_beatmapset(deser: &Deserializer) -> Result<()> {
    // 读取第一个 BeatmapSetInfo 对象（table_key = 0）
    match deser.read_object(0, 0) {
        Ok(obj) => {
            println!("\n=== First BeatmapSetInfo ===");

            // 打印所有字段
            for (name, value) in &obj.fields {
                match value {
                    no_realm::realm::format::deserializer::Value::Uuid(uuid) => {
                        println!("  {}: {:x?}", name, uuid);
                    }
                    no_realm::realm::format::deserializer::Value::Int(v) => {
                        println!("  {}: {}", name, v);
                    }
                    no_realm::realm::format::deserializer::Value::String(s) => {
                        println!("  {}: \"{}\"", name, s);
                    }
                    no_realm::realm::format::deserializer::Value::Timestamp(ts) => {
                        println!("  {}: {} (timestamp)", name, ts);
                    }
                    no_realm::realm::format::deserializer::Value::Bool(b) => {
                        println!("  {}: {}", name, b);
                    }
                    no_realm::realm::format::deserializer::Value::Link(table, offset) => {
                        println!("  {}: Link(table={}, offset={})", name, table, offset);
                    }
                    no_realm::realm::format::deserializer::Value::LinkList(items) => {
                        println!("  {}: LinkList({} items)", name, items.len());
                    }
                    _ => {
                        println!("  {}: {:?}", name, value);
                    }
                }
            }
        }
        Err(e) => {
            println!("Error reading first BeatmapSet: {}", e);
        }
    }

    // 读取前几个 BeatmapSetInfo
    println!("\n=== All BeatmapSets ===");
    let count = deser.count_table_rows(0)?;
    for i in 0..count.min(5) {
        match deser.read_object(0, i) {
            Ok(obj) => {
                let online_id = obj.fields.get("OnlineID")
                    .and_then(|v| if let no_realm::realm::format::deserializer::Value::Int(id) = v { Some(id) } else { None })
                    .unwrap_or(&-1);

                let date_added = obj.fields.get("DateAdded")
                    .and_then(|v| if let no_realm::realm::format::deserializer::Value::Timestamp(ts) = v { Some(ts) } else { None })
                    .unwrap_or(&0);

                println!("  [{}] OnlineID={}, DateAdded={}", i, online_id, date_added);
            }
            Err(e) => {
                println!("  [{}] Error: {}", i, e);
            }
        }
    }

    Ok(())
}
