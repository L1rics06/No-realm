//! 测试 Collection 读取功能

use no_realm::{Result, RealmDatabase, operations};
use std::env;

fn main() -> Result<()> {
    let db_path = env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example test_collections <path-to-client.realm>");
            eprintln!("Example: cargo run --example test_collections ~/.local/share/osu/client.realm");
            std::process::exit(1);
        });

    println!("Opening database: {}", db_path);

    let db = RealmDatabase::open(&db_path)?;

    println!("Database opened successfully");
    println!("Database size: {} bytes\n", db.size()?);

    // 测试 list_all 函数
    println!("=== Testing operations::collection::list_all ===");
    match operations::collection::list_all(&db) {
        Ok(collections) => {
            println!("✅ Successfully retrieved {} collections\n", collections.len());

            for (i, collection) in collections.iter().enumerate() {
                println!("Collection #{}:", i);
                println!("  Name: {}", collection.name);
                println!("  ID: {}", collection.id);
                println!("  LastModified: {}", collection.last_modified);
                println!("  Beatmaps: {} MD5 hashes", collection.beatmap_md5_hashes.len());

                // 显示前几个 MD5 哈希
                for (j, md5) in collection.beatmap_md5_hashes.iter().take(5).enumerate() {
                    println!("    [{}]: {}", j, md5);
                }

                if collection.beatmap_md5_hashes.len() > 5 {
                    println!("    ... and {} more", collection.beatmap_md5_hashes.len() - 5);
                }

                println!();
            }

            println!("✅ All tests passed!");
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
