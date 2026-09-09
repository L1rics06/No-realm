//! Example: Parse and validate Realm file headers
//!
//! This example demonstrates the new format parsing capabilities:
//! - Read and validate Realm file headers
//! - Detect format versions (v9 legacy, v24 modern)
//! - Check encryption status
//! - Display MVCC top references

use no_realm::realm::{RealmReader, RealmHeader, FormatVersion};
use std::env;
use std::process;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path-to-realm-file>", args[0]);
        eprintln!("");
        eprintln!("Example:");
        eprintln!("  {} ~/.local/share/osu/client.realm", args[0]);
        process::exit(1);
    }

    let path = &args[1];

    println!("=== Realm File Header Parser ===\n");
    println!("Reading: {}\n", path);

    match RealmReader::open(path) {
        Ok(reader) => {
            let header = reader.header();

            println!("✓ Valid Realm file!");
            println!("");
            println!("Header Information:");
            println!("  Format Version: {} ({})",
                header.file_format_version,
                match header.version() {
                    FormatVersion::Legacy9 => "Legacy - supported by realm-codec",
                    FormatVersion::Modern24 => "Modern - cluster-based format",
                    FormatVersion::Unknown(v) => "Unknown",
                }
            );
            println!("  Encrypted: {}", if header.is_encrypted() { "Yes ⚠️" } else { "No" });
            println!("");

            println!("MVCC (Copy-on-Write) Structure:");
            println!("  Top Ref 0: 0x{:016X}", header.top_refs()[0]);
            println!("  Top Ref 1: 0x{:016X}", header.top_refs()[1]);
            println!("  Active:    0x{:016X} (flags bit {})",
                header.active_top_ref(),
                header.flags & 0x01
            );
            println!("");

            println!("Raw Header Bytes:");
            println!("  Flags: 0x{:02X}", header.flags);
            println!("    - Bit 0 (active ref): {}", header.flags & 0x01);
            println!("    - Bit 7 (encrypted): {}", (header.flags & 0x80) >> 7);
            println!("  Reserved: 0x{:02X}", header.reserved);
            println!("");

            match header.version() {
                FormatVersion::Legacy9 => {
                    println!("ℹ️  This is a legacy format (v9) file.");
                    println!("   Can be read using realm-codec crate.");
                },
                FormatVersion::Modern24 => {
                    println!("ℹ️  This is a modern format (v24) file.");
                    println!("   Uses cluster B+Tree structure for data storage.");
                    println!("   Typically used by osu!lazer.");
                },
                FormatVersion::Unknown(v) => {
                    println!("⚠️  Unknown format version: {}", v);
                    println!("   This version is not currently supported.");
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to read Realm file:");
            eprintln!("  {}", e);
            process::exit(1);
        }
    }
}
