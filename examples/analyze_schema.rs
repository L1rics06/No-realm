//! 深入分析 Realm Schema 编码格式

use no_realm::realm::{RealmReader, format::ArrayView};
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = dirs::data_local_dir().unwrap().join("osu/client.realm");

    let mut file = File::open(&db_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let reader = RealmReader::open(&db_path)?;
    let header = reader.header();
    let root_offset = header.active_top_ref() as usize;

    println!("=== Schema 深度分析 ===\n");

    // 1. 解析根数组获取 schema ref
    let root = ArrayView::parse(&data, root_offset)?;
    let schema_rot = root.get_ref_or_tagged(0)?;

    let schema_ref = match schema_rot {
        no_realm::realm::format::RefOrTagged::Ref(r) => r.offset() as usize,
        _ => {
            println!("Root[0] 不是引用!");
            return Ok(());
        }
    };

    println!("Schema 数组位于: 0x{:X}", schema_ref);
    let schema_array = ArrayView::parse(&data, schema_ref)?;

    println!("Schema 数组属性:");
    println!("  元素数: {}", schema_array.len());
    println!("  宽度: {} bits", schema_array.width());
    println!("  Has refs: {}", schema_array.header().has_refs());
    println!();

    // 2. 分析每个元素
    println!("Schema 元素详细分析:");
    for i in 0..schema_array.len() {
        let rot = schema_array.get_ref_or_tagged(i)?;

        print!("[{}] ", i);
        match rot {
            no_realm::realm::format::RefOrTagged::Ref(r) => {
                let offset = r.offset() as usize;
                println!("Ref: 0x{:X}", offset);

                if offset > 0 && offset < data.len() {
                    // 尝试解析为数组
                    if let Ok(array) = ArrayView::parse(&data, offset) {
                        println!("     → 数组: {} 元素, width={} bits",
                            array.len(), array.width());

                        // 尝试找到附近的字符串
                        let search_start = offset.saturating_sub(100);
                        let search_end = (offset + 500).min(data.len());
                        let nearby = String::from_utf8_lossy(&data[search_start..search_end]);

                        if let Some(class_pos) = nearby.find("class_") {
                            let class_str = &nearby[class_pos..];
                            if let Some(end) = class_str.find(|c: char| c == '\0' || c < ' ') {
                                println!("     → 可能的类名: {}", &class_str[..end]);
                            }
                        }

                        // 打印前几个元素
                        if array.len() <= 20 {
                            println!("     → 前 {} 个元素:", array.len().min(5));
                            for j in 0..array.len().min(5) {
                                if let Ok(elem) = array.get_ref_or_tagged(j) {
                                    match elem {
                                        no_realm::realm::format::RefOrTagged::Ref(r2) => {
                                            println!("        [{}] Ref: 0x{:X}", j, r2.offset());
                                        }
                                        no_realm::realm::format::RefOrTagged::Tagged(t) => {
                                            println!("        [{}] Tagged: {} (0x{:X})",
                                                j, t.value(), t.value());
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // 不是数组，看看是什么
                        let preview: Vec<String> = data[offset..offset.saturating_add(32).min(data.len())]
                            .iter()
                            .map(|b| format!("{:02X}", b))
                            .collect();
                        println!("     → 字节: {}", preview.join(" "));
                    }
                }
            }
            no_realm::realm::format::RefOrTagged::Tagged(t) => {
                println!("Tagged: {} (0x{:X})", t.value(), t.value());

                // 尝试解释这个值
                if t.value() < 256 {
                    println!("     → 可能是: 表数量或版本号");
                } else {
                    println!("     → 可能是: 元数据或标志");
                }
            }
        }
        println!();
    }

    // 3. 尝试在整个文件中搜索表名
    println!("\n=== 搜索已知表名 ===");
    let table_names = [
        "class_BeatmapSetInfo",
        "class_BeatmapInfo",
        "class_BeatmapMetadata",
        "class_Skin",
        "class_BeatmapCollection",
        "class_RealmFile",
        "class_Ruleset",
        "class_KeyBinding",
    ];

    for name in &table_names {
        if let Some(pos) = data.windows(name.len())
            .position(|window| window == name.as_bytes()) {
            println!("{}: 位于 0x{:X}", name, pos);

            // 看看这个位置附近的数据结构
            let start = pos.saturating_sub(50);
            let end = (pos + 100).min(data.len());

            // 检查是否在数组中
            for check_offset in (start..pos).step_by(8) {
                if let Ok(arr) = ArrayView::parse(&data, check_offset) {
                    if check_offset + 8 + arr.header().payload_len() > pos {
                        println!("  → 在数组中: offset=0x{:X}, {} 元素",
                            check_offset, arr.len());
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
