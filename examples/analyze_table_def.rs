//! 分析表名附近的数据结构，理解表定义格式

use no_realm::realm::format::ArrayView;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = dirs::data_local_dir().unwrap().join("osu/client.realm");

    let mut file = File::open(&db_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    println!("=== 表定义结构分析 ===\n");

    // 找到的表名位置
    let table_names = [
        ("class_BeatmapSetInfo", 0x0), // 需要实际搜索
        ("class_BeatmapMetadata", 0x31F8),
        ("class_Skin", 0x34B8),
        ("class_BeatmapCollection", 0x3178),
        ("class_Ruleset", 0x33F8),
        ("class_KeyBinding", 0x32F8),
    ];

    for (name, _) in &table_names {
        if let Some(pos) = data.windows(name.len())
            .position(|window| window == name.as_bytes()) {

            println!("--- {} (位于 0x{:X}) ---", name, pos);

            // 向后查找可能包含这个字符串的数组
            let search_start = pos.saturating_sub(500);

            for offset in (search_start..pos).step_by(8).rev() {
                if let Ok(arr) = ArrayView::parse(&data, offset) {
                    let arr_end = offset + 8 + arr.header().payload_len();

                    // 检查字符串是否在这个数组的 payload 范围内
                    if offset + 8 <= pos && pos < arr_end {
                        println!("  字符串在数组中:");
                        println!("    数组偏移: 0x{:X}", offset);
                        println!("    元素数: {}", arr.len());
                        println!("    宽度: {} bits", arr.width());
                        println!("    Payload 长度: {} bytes", arr.header().payload_len());

                        // 尝试提取字符串（如果是字符串数组）
                        if arr.width() == 8 {
                            let mut string_bytes = Vec::new();
                            for i in 0..arr.len() {
                                if let Ok(byte) = arr.get(i) {
                                    if byte == 0 { break; }
                                    string_bytes.push(byte as u8);
                                }
                            }
                            let extracted = String::from_utf8_lossy(&string_bytes);
                            println!("    提取的字符串: {}", extracted);
                        }

                        // 向前查找可能是表定义的结构
                        println!("\n  向前查找表定义结构:");
                        for parent_offset in (offset.saturating_sub(1000)..offset).step_by(8).rev() {
                            if let Ok(parent) = ArrayView::parse(&data, parent_offset) {
                                // 表定义可能包含多个字段
                                if parent.len() > 5 && parent.len() < 50 {
                                    let parent_end = parent_offset + 8 + parent.header().payload_len();

                                    if parent_offset < offset && offset < parent_end {
                                        println!("    可能的表定义数组: 0x{:X}", parent_offset);
                                        println!("      元素数: {}", parent.len());
                                        println!("      宽度: {} bits", parent.width());

                                        // 打印前几个元素
                                        println!("      前 5 个元素:");
                                        for i in 0..parent.len().min(5) {
                                            if let Ok(elem) = parent.get_ref_or_tagged(i) {
                                                match elem {
                                                    no_realm::realm::format::RefOrTagged::Ref(r) => {
                                                        println!("        [{}] Ref: 0x{:X}", i, r.offset());

                                                        // 如果指向字符串数组，尝试读取
                                                        if r.offset() as usize == offset {
                                                            println!("          → 这是表名字符串!");
                                                        }
                                                    }
                                                    no_realm::realm::format::RefOrTagged::Tagged(t) => {
                                                        println!("        [{}] Tagged: {}", i, t.value());
                                                    }
                                                }
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        }

                        println!();
                        break;
                    }
                }
            }
        }
    }

    // 分析 Tables 引用处的数据
    println!("\n=== 分析 Root[1] (Tables 数组) ===");
    let root_offset = 0x4DFD0; // 从之前的探索得知
    let root = ArrayView::parse(&data, root_offset)?;
    let tables_rot = root.get_ref_or_tagged(1)?;

    if let no_realm::realm::format::RefOrTagged::Ref(tables_ref) = tables_rot {
        let tables_offset = tables_ref.offset() as usize;
        println!("Tables 数组位于: 0x{:X}", tables_offset);

        let tables = ArrayView::parse(&data, tables_offset)?;
        println!("Tables 数组: {} 个表", tables.len());

        // 对于前几个表，显示它们的内容
        for i in 0..tables.len().min(3) {
            if let Ok(no_realm::realm::format::RefOrTagged::Ref(table_ref)) = tables.get_ref_or_tagged(i) {
                let table_offset = table_ref.offset() as usize;
                println!("\nTable[{}] at 0x{:X}:", i, table_offset);

                if let Ok(table_root) = ArrayView::parse(&data, table_offset) {
                    println!("  元素数: {}", table_root.len());
                    println!("  宽度: {} bits", table_root.width());
                    println!("  Is inner: {}", table_root.header().is_inner_node());

                    // 这些可能是 B+Tree 的根节点
                    // 打印前几个元素
                    if table_root.len() <= 20 {
                        println!("  前 3 个元素:");
                        for j in 0..table_root.len().min(3) {
                            if let Ok(elem) = table_root.get_ref_or_tagged(j) {
                                match elem {
                                    no_realm::realm::format::RefOrTagged::Ref(r) => {
                                        println!("    [{}] Ref: 0x{:X}", j, r.offset());
                                    }
                                    no_realm::realm::format::RefOrTagged::Tagged(t) => {
                                        println!("    [{}] Tagged: {}", j, t.value());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
