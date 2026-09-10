//! 列出 osu!lazer 数据库中的所有 Beatmap

use no_realm::Result;
use no_realm::realm::format::deserializer::{Deserializer, Value};
use std::env;

fn main() -> Result<()> {
    let db_path = env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example list_beatmaps <path-to-client.realm>");
            eprintln!("Example: cargo run --example list_beatmaps ~/.local/share/osu/client.realm");
            std::process::exit(1);
        });

    println!("Opening database: {}", db_path);

    let data = std::fs::read(&db_path)?;
    let header = no_realm::realm::format::RealmHeader::parse(&data)?;
    let deser = Deserializer::new(&data, header.active_top_ref() as usize)?;

    // 读取所有 BeatmapSetInfo (table 0)
    println!("\n=== BeatmapSet List ===");
    let beatmapsets = deser.iter_table_objects(0)?;

    println!("Found {} valid beatmap sets\n", beatmapsets.len());

    for (row_index, obj) in beatmapsets.iter() {
        // 提取字段
        let online_id = extract_int(&obj.fields, "OnlineID").unwrap_or(-1);
        let date_added = extract_timestamp(&obj.fields, "DateAdded").unwrap_or(0);
        let status = extract_int(&obj.fields, "Status").unwrap_or(-1);

        println!("BeatmapSet #{}", row_index);
        println!("  OnlineID: {}", online_id);
        println!("  DateAdded: {}", date_added);
        println!("  Status: {}", status);

        // 显示 UUID
        if let Some(Value::Uuid(uuid)) = obj.fields.get("ID") {
            println!("  UUID: {}", format_uuid(uuid));
        }

        // 显示链接的 beatmaps 数量
        if let Some(Value::LinkList(beatmaps)) = obj.fields.get("Beatmaps") {
            println!("  Beatmaps: {} linked", beatmaps.len());
        }

        // 显示文件数量
        if let Some(Value::LinkList(files)) = obj.fields.get("Files") {
            println!("  Files: {} linked", files.len());
        }

        println!();
    }

    println!("Total: {} beatmap sets", beatmapsets.len());

    Ok(())
}

fn extract_int(fields: &std::collections::HashMap<String, Value>, name: &str) -> Option<i64> {
    fields.get(name).and_then(|v| {
        if let Value::Int(n) = v {
            Some(*n)
        } else {
            None
        }
    })
}

fn extract_timestamp(fields: &std::collections::HashMap<String, Value>, name: &str) -> Option<i64> {
    fields.get(name).and_then(|v| {
        if let Value::Timestamp(ts) = v {
            Some(*ts)
        } else {
            None
        }
    })
}

fn format_uuid(uuid: &[u8; 16]) -> String {
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        uuid[0], uuid[1], uuid[2], uuid[3],
        uuid[4], uuid[5],
        uuid[6], uuid[7],
        uuid[8], uuid[9],
        uuid[10], uuid[11], uuid[12], uuid[13], uuid[14], uuid[15]
    )
}
