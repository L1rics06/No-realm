use no_realm::{RealmDatabase, RealmReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let db_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            let default_path = dirs::data_local_dir()
                .map(|p| p.join("osu/client.realm"))
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "client.realm".to_string());
            default_path
        });

    println!("Opening database: {}", db_path);

    let db = RealmDatabase::open(&db_path)?;
    println!("✅ Database opened successfully");
    println!("   Path: {}", db.path().display());
    println!("   Size: {} bytes", db.size()?);
    println!("   Read-only: {}", db.is_read_only());

    // 尝试使用 RealmReader
    println!("\nReading Realm file format...");
    let mut reader = RealmReader::open(&db_path)?;
    println!("   File format version: {}", reader.file_format_version());

    // 尝试列出 beatmaps
    println!("\nQuerying beatmaps...");
    let beatmaps = reader.query_beatmap_sets()?;
    println!("Found {} beatmap sets", beatmaps.len());

    Ok(())
}
