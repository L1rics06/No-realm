//! 测试 beatmap 操作功能

use no_realm::{Result, RealmDatabase, operations};
use std::env;

fn main() -> Result<()> {
    let db_path = env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example test_beatmap_operations <path-to-client.realm>");
            eprintln!("Example: cargo run --example test_beatmap_operations ~/.local/share/osu/client.realm");
            std::process::exit(1);
        });

    println!("Opening database: {}", db_path);

    let db = RealmDatabase::open(&db_path)?;

    println!("Database opened successfully");
    println!("Database size: {} bytes", db.size()?);

    // 测试 list_all 函数
    println!("\n=== Testing operations::beatmap::list_all ===");
    match operations::beatmap::list_all(&db) {
        Ok(beatmap_sets) => {
            println!("✅ Successfully retrieved {} beatmap sets", beatmap_sets.len());

            for (i, beatmap_set) in beatmap_sets.iter().enumerate() {
                println!("\nBeatmapSet #{}:", i);
                println!("  ID: {}", beatmap_set.id);
                println!("  OnlineID: {:?}", beatmap_set.online_id);
                println!("  DateAdded: {}", beatmap_set.date_added);
                println!("  Title: {}", beatmap_set.metadata.title);
                println!("  Artist: {}", beatmap_set.metadata.artist);
                println!("  Beatmaps: {} linked", beatmap_set.beatmaps.len());
                println!("  Files: {} linked", beatmap_set.files.len());
                println!("  Protected: {}", beatmap_set.protected);
                println!("  Deleted: {}", beatmap_set.deleted_at.is_some());
            }

            println!("\n✅ All tests passed!");
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
