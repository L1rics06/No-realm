//! 调试 Realm 文件结构

use no_realm::Result;
use std::env;

fn main() -> Result<()> {
    let db_path = env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example debug_realm <path-to-client.realm>");
            std::process::exit(1);
        });

    println!("Opening database: {}", db_path);

    // 读取 Realm 文件
    let data = std::fs::read(&db_path)?;
    println!("Read {} bytes from file", data.len());

    // 解析文件头
    let header = no_realm::realm::format::RealmHeader::parse(&data)?;
    println!("\n=== Realm Header ===");
    println!("Version: {:?}", header.version());
    println!("top_ref_0: 0x{:x}", header.top_ref_0);
    println!("top_ref_1: 0x{:x}", header.top_ref_1);
    println!("flags: 0x{:02x}", header.flags);
    println!("Active top_ref: 0x{:x}", header.active_top_ref());

    // 检查 top_ref 是否是 tagged 值
    let top_ref = header.active_top_ref();
    if (top_ref & 0x01) == 0 {
        println!("  -> This is a REF (8-byte aligned offset)");
    } else {
        println!("  -> This is a TAGGED value (inline integer)");
        let value = (top_ref as i64) >> 1;
        println!("     Decoded value: {}", value);
    }

    // 尝试读取 top_ref 位置的数据
    println!("\n=== Data at top_ref offset ===");
    let offset = top_ref as usize;
    if offset + 8 <= data.len() {
        let bytes = &data[offset..offset + 8];
        println!("Bytes at 0x{:x}: {:02x?}", offset, bytes);

        // 尝试解析为 array header
        let checksum = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let flags = bytes[4];
        // Size is 3 bytes little-endian: bytes[5] is LSB, bytes[7] is MSB
        let size = (bytes[5] as u32) | ((bytes[6] as u32) << 8) | ((bytes[7] as u32) << 16);

        println!("  Checksum: 0x{:08x}", checksum);
        println!("  Flags: 0x{:02x}", flags);
        println!("  Size: {}", size);

        if checksum == 0x41414141 {
            println!("  ✓ Valid array header!");
        } else {
            println!("  ✗ Invalid checksum - not an array header");

            // 也许这是一个 ref-or-tagged 值？
            let val = u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7]
            ]);
            println!("  As u64: 0x{:x}", val);
            if (val & 0x01) == 0 {
                println!("    -> This is a REF pointing to 0x{:x}", val);
            } else {
                println!("    -> This is a TAGGED value: {}", (val as i64) >> 1);
            }
        }
    } else {
        println!("Offset out of bounds!");
    }

    Ok(())
}
