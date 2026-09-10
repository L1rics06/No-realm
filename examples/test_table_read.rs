//! 测试表读取

use no_realm::Result;
use no_realm::realm::format::deserializer::Deserializer;
use std::env;

fn main() -> Result<()> {
    let db_path = env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example test_table_read <path-to-client.realm>");
            std::process::exit(1);
        });

    let data = std::fs::read(&db_path)?;
    let header = no_realm::realm::format::RealmHeader::parse(&data)?;

    println!("Root offset: 0x{:x}", header.active_top_ref());

    let deser = Deserializer::new(&data, header.active_top_ref() as usize)?;

    // 尝试读取所有表的行数
    println!("\n=== Table Row Counts ===");
    for table_key in 0..10 {
        match deser.count_table_rows(table_key) {
            Ok(count) => {
                println!("Table[{}]: {} rows", table_key, count);
            }
            Err(e) => {
                println!("Table[{}]: Error - {}", table_key, e);
            }
        }
    }

    // 尝试读取 Table 0 的第一个对象
    println!("\n=== Reading Table 0, Row 0 ===");
    match deser.read_object(0, 0) {
        Ok(obj) => {
            println!("✓ Successfully read object");
            println!("Fields ({}):", obj.fields.len());
            for (name, value) in obj.fields.iter().take(5) {
                println!("  {}: {:?}", name, value);
            }
        }
        Err(e) => {
            println!("✗ Error: {}", e);
        }
    }

    Ok(())
}
