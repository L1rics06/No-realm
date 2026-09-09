//! 深入探索 Realm 根节点结构
//!
//! 理解根数组的布局，这是反序列化的起点

use no_realm::realm::{RealmReader, format::{ArrayView, RefOrTagged}};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = dirs::data_local_dir()
        .unwrap()
        .join("osu/client.realm");

    println!("=== Realm Root Structure Deep Dive ===\n");

    let mut file = File::open(&db_path)?;
    let file_size = file.metadata()?.len() as usize;
    let mut data = vec![0u8; file_size];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut data)?;

    let reader = RealmReader::open(&db_path)?;
    let header = reader.header();
    let root_offset = header.active_top_ref() as usize;

    println!("Root array at offset: 0x{:X}", root_offset);
    let root = ArrayView::parse(&data, root_offset)?;

    println!("Root array properties:");
    println!("  Elements: {}", root.len());
    println!("  Width: {} bits", root.width());
    println!("  Has refs: {}", root.header().has_refs());
    println!("  Is inner: {}", root.header().is_inner_node());
    println!();

    // Realm 根数组通常包含：
    // [0] = schema ref (指向 schema 定义)
    // [1] = object table refs (指向对象数据的 B+Tree)
    // [2] = file size
    // [3+] = 其他元数据

    println!("Root elements breakdown:");
    for i in 0..root.len() {
        let rot = root.get_ref_or_tagged(i)?;
        print!("  [{}] ", i);

        match rot {
            RefOrTagged::Ref(r) => {
                let offset = r.offset();
                println!("Ref: 0x{:X}", offset);

                // 尝试解析这个引用指向的数据
                if offset < file_size as u64 && offset > 0 {
                    if let Ok(array) = ArrayView::parse(&data, offset as usize) {
                        println!("      → Array: {} elements, width={}, inner={}, has_refs={}",
                            array.len(), array.width(),
                            array.header().is_inner_node(),
                            array.header().has_refs());

                        // 如果是小数组，打印前几个元素
                        if array.len() <= 20 {
                            println!("      → First elements:");
                            for j in 0..array.len().min(5) {
                                if let Ok(elem) = array.get_ref_or_tagged(j) {
                                    match elem {
                                        RefOrTagged::Ref(r2) => println!("        [{}] Ref: 0x{:X}", j, r2.offset()),
                                        RefOrTagged::Tagged(t) => println!("        [{}] Tagged: {}", j, t.value()),
                                    }
                                }
                            }
                        }
                    } else {
                        // 不是数组，可能是其他结构
                        let preview = &data[offset as usize..].iter()
                            .take(16)
                            .map(|b| format!("{:02X}", b))
                            .collect::<Vec<_>>()
                            .join(" ");
                        println!("      → Raw bytes: {}", preview);
                    }
                }
            }
            RefOrTagged::Tagged(t) => {
                println!("Tagged: {} (0x{:X})", t.value(), t.value());
            }
        }
    }

    println!("\n=== Schema Exploration ===");
    // 第一个引用通常是 schema
    if let Ok(RefOrTagged::Ref(schema_ref)) = root.get_ref_or_tagged(0) {
        let schema_offset = schema_ref.offset() as usize;
        println!("Schema ref at: 0x{:X}", schema_offset);

        if schema_offset < file_size {
            if let Ok(schema_array) = ArrayView::parse(&data, schema_offset) {
                println!("Schema array: {} elements", schema_array.len());

                // Schema 通常包含表定义列表
                for i in 0..schema_array.len().min(10) {
                    if let Ok(elem) = schema_array.get_ref_or_tagged(i) {
                        match elem {
                            RefOrTagged::Ref(table_ref) => {
                                let table_offset = table_ref.offset() as usize;
                                if table_offset < file_size {
                                    // 尝试在附近找到表名字符串
                                    let end = (table_offset + 200).min(file_size);
                                    let nearby = String::from_utf8_lossy(&data[table_offset..end]);
                                    if let Some(pos) = nearby.find("class_") {
                                        let class_name = nearby[pos..].split(|c: char| !c.is_alphanumeric() && c != '_')
                                            .next()
                                            .unwrap_or("unknown");
                                        println!("  Table [{}]: {} at 0x{:X}", i, class_name, table_offset);
                                    } else {
                                        println!("  Table [{}]: Ref at 0x{:X}", i, table_offset);
                                    }
                                }
                            }
                            RefOrTagged::Tagged(t) => {
                                println!("  Schema [{}]: Tagged value {}", i, t.value());
                            }
                        }
                    }
                }
            }
        }
    } else {
        println!("Root[0] is not a ref, investigating actual structure...");
    }

    println!("\n=== Object Tables Exploration ===");
    // 第二个引用通常是对象表列表
    if root.len() > 1 {
        if let Ok(RefOrTagged::Ref(tables_ref)) = root.get_ref_or_tagged(1) {
            let tables_offset = tables_ref.offset() as usize;
            println!("Object tables ref at: 0x{:X}", tables_offset);

            if let Ok(tables_array) = ArrayView::parse(&data, tables_offset) {
                println!("Tables array: {} elements", tables_array.len());

                // 每个表可能是一个 B+Tree 根
                for i in 0..tables_array.len().min(10) {
                    if let Ok(rot) = tables_array.get_ref_or_tagged(i) {
                        match rot {
                            RefOrTagged::Ref(r) => {
                                let offset = r.offset() as usize;
                                if let Ok(tree) = ArrayView::parse(&data, offset) {
                                    println!("  Table [{}] at 0x{:X}: {} elements, inner={}",
                                        i, offset, tree.len(), tree.header().is_inner_node());
                                }
                            }
                            RefOrTagged::Tagged(t) => {
                                println!("  Table [{}]: Empty (tagged {})", i, t.value());
                            }
                        }
                    }
                }
            }
        }
    }

    println!("\n=== Complete ===");
    Ok(())
}
