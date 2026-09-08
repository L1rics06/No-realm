use no_realm::RealmDatabase;

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

    // 尝试列出 beatmaps
    println!("\nQuerying beatmaps...");
    let beatmaps = no_realm::operations::beatmap::list_all(&db)?;
    println!("Found {} beatmap sets", beatmaps.len());

    Ok(())
}
