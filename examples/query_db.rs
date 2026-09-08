use no_realm::{RealmDatabase, RealmReader, Result};
use std::env;

fn main() -> Result<()> {
    env_logger::init();

    let db_path = env::args()
        .nth(1)
        .unwrap_or_else(|| {
            dirs::data_local_dir()
                .map(|p| p.join("osu/client.realm"))
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "client.realm".to_string())
        });

    println!("Opening Realm database: {}", db_path);
    println!();

    // 使用 RealmReader 分析结构
    let mut reader = RealmReader::open(&db_path)?;
    println!("✅ Realm file opened successfully");
    println!("   File format version: {}", reader.file_format_version());
    println!();

    // 分析 schema
    println!("📊 Analyzing Realm schema...");
    match reader.analyze_schema() {
        Ok(schema) => {
            println!("Found {} tables:", schema.tables.len());
            for (name, table) in &schema.tables {
                println!("  📦 {}", name);
                println!("     Fields: {}", table.fields.len());
            }
            println!();
        }
        Err(e) => {
            println!("⚠️  Failed to analyze schema: {}", e);
            println!();
        }
    }

    // 查找字符串
    println!("🔍 Analyzing Realm content...");
    match reader.find_strings() {
        Ok(strings) => {
            let relevant: Vec<_> = strings
                .iter()
                .filter(|s| {
                    s.len() > 3
                        && s.len() < 40
                        && s.chars().any(|c| c.is_uppercase())
                        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
                })
                .take(30)
                .collect();

            println!("Found {} potential field/table names:", relevant.len());
            for (i, name) in relevant.iter().enumerate() {
                print!("  {}", name);
                if (i + 1) % 4 == 0 {
                    println!();
                } else {
                    print!(", ");
                }
            }
            println!("\n");
        }
        Err(e) => {
            println!("⚠️  Failed to find strings: {}", e);
            println!();
        }
    }

    // 尝试通过 RealmDatabase 打开
    println!("📚 Testing RealmDatabase interface...");
    match RealmDatabase::open(&db_path) {
        Ok(db) => {
            println!("✅ RealmDatabase opened");
            println!("   Path: {}", db.path().display());
            println!("   Size: {} bytes", db.size()?);
            println!();

            // 注意：当前查询接口尚未完全实现
            println!("ℹ️  Query/mutation interfaces are in development");
            println!("   Use RealmReader for now to analyze database structure");
        }
        Err(e) => {
            println!("⚠️  Failed to open with RealmDatabase: {}", e);
        }
    }

    Ok(())
}
