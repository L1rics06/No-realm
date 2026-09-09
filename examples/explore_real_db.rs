//! 探索真实的 osu!lazer Realm 数据库结构
//!
//! 使用现有的解析器来分析 client.realm 文件

use no_realm::realm::{RealmReader, format::{ArrayView, RefOrTagged}};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let db_path = dirs::data_local_dir()
        .unwrap()
        .join("osu/client.realm");

    if !db_path.exists() {
        eprintln!("Database not found at: {:?}", db_path);
        return Ok(());
    }

    println!("=== Exploring osu!lazer Realm Database ===");
    println!("Path: {:?}", db_path);
    println!("Size: {} bytes", std::fs::metadata(&db_path)?.len());
    println!();

    // 1. Read header using RealmReader
    println!("--- Header Information ---");
    let mut reader = RealmReader::open(&db_path)?;
    let header = reader.header();

    println!("Format Version: {}", header.version());
    println!("Top Ref 0: 0x{:X}", header.top_ref_0);
    println!("Top Ref 1: 0x{:X}", header.top_ref_1);
    println!("Active Top Ref: 0x{:X}", header.active_top_ref());
    println!("Flags: 0x{:02X}", header.flags);
    println!("Encrypted: {}", header.is_encrypted());
    println!();

    // 2. Read raw file for deeper analysis
    println!("--- Raw Data Analysis ---");
    let mut file = File::open(&db_path)?;
    let file_size = file.metadata()?.len() as usize;

    // Read entire file
    let mut data = vec![0u8; file_size];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut data)?;

    // Try to parse array at active top ref
    let top_ref = header.active_top_ref() as usize;
    println!("Attempting to parse array at active top_ref: 0x{:X}", top_ref);

    if top_ref < file_size {
        match ArrayView::parse(&data, top_ref) {
            Ok(view) => {
                println!("✓ Successfully parsed root array!");
                println!("  Elements: {}", view.len());
                println!("  Width: {} bits", view.width());
                println!("  Is inner node: {}", view.header().is_inner_node());
                println!("  Has refs: {}", view.header().has_refs());

                // Print first few elements
                println!("  First 5 elements:");
                for i in 0..5.min(view.len()) {
                    if let Ok(rot) = view.get_ref_or_tagged(i) {
                        match rot {
                            RefOrTagged::Ref(r) => println!("    [{}] Ref: 0x{:X}", i, r.offset()),
                            RefOrTagged::Tagged(t) => println!("    [{}] Tagged: {}", i, t.value()),
                        }
                    }
                }
            }
            Err(e) => {
                println!("✗ Failed to parse root array: {}", e);
            }
        }
    }
    println!();

    // 3. Find strings for schema analysis
    println!("--- String Analysis (Schema Detection) ---");
    let strings = reader.find_strings()?;
    println!("Found {} strings in database", strings.len());

    // Look for osu-specific table names
    let table_keywords = [
        "BeatmapInfo", "BeatmapSet", "BeatmapMetadata",
        "Skin", "BeatmapCollection", "RealmFile",
        "Ruleset", "KeyBinding", "Score"
    ];

    println!("Detected tables:");
    for keyword in &table_keywords {
        if strings.iter().any(|s| s.contains(keyword)) {
            println!("  ✓ {}", keyword);
        }
    }
    println!();

    // 4. Analyze byte patterns
    println!("--- Byte Pattern Analysis ---");

    // Count array headers (0x41414141 checksum)
    let mut array_count = 0;
    for i in 0..data.len().saturating_sub(4) {
        if &data[i..i+4] == &[0x41, 0x41, 0x41, 0x41] {
            array_count += 1;
        }
    }
    println!("Potential array nodes (0x41414141 checksum): {}", array_count);

    // Look for common patterns
    let mut ref_count = 0;
    for i in (0..data.len().saturating_sub(8)).step_by(8) {
        let value = u64::from_le_bytes(data[i..i+8].try_into().unwrap());
        if value != 0 && (value & 1) == 0 && (value as usize) < file_size {
            ref_count += 1;
        }
    }
    println!("Potential refs (8-byte aligned, in-bounds): {}", ref_count);

    println!();
    println!("=== Analysis Complete ===");

    Ok(())
}
