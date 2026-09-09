//! 查看 Realm 数据库中的 Collection 结构
//!
//! 用于分析真实的数据库表结构

use rusqlite::Connection;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-client.realm>", args[0]);
        eprintln!("Example: {} ~/.local/share/osu/client.realm", args[0]);
        std::process::exit(1);
    }

    let db_path = &args[1];
    println!("Opening Realm database: {}", db_path);

    let conn = Connection::open(db_path)?;

    // 列出所有表
    println!("\n=== Tables ===");
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?;
    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    for table in &tables {
        println!("  - {}", table);
    }

    // 查找 Collection 相关的表
    println!("\n=== Collection Tables ===");
    for table in &tables {
        if table.to_lowercase().contains("collection") {
            println!("\nTable: {}", table);

            // 获取表结构
            let mut stmt = conn.prepare(&format!("PRAGMA table_info('{}')", table))?;
            let columns: Vec<(i32, String, String)> = stmt
                .query_map([], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })?
                .collect::<Result<Vec<_>, _>>()?;

            println!("Columns:");
            for (cid, name, col_type) in columns {
                println!("  {}: {} ({})", cid, name, col_type);
            }

            // 查看前几行数据
            let count_query = format!("SELECT COUNT(*) FROM '{}'", table);
            let count: i64 = conn.query_row(&count_query, [], |row| row.get(0))?;
            println!("Row count: {}", count);

            if count > 0 {
                println!("Sample data (first 3 rows):");
                let sample_query = format!("SELECT * FROM '{}' LIMIT 3", table);
                let mut stmt = conn.prepare(&sample_query)?;
                let column_count = stmt.column_count();

                let rows = stmt.query_map([], |row| {
                    let mut values = Vec::new();
                    for i in 0..column_count {
                        let value: String = match row.get::<_, Option<Vec<u8>>>(i)? {
                            Some(bytes) => {
                                if bytes.len() <= 32 {
                                    format!("{:?}", bytes)
                                } else {
                                    format!("<binary {} bytes>", bytes.len())
                                }
                            }
                            None => match row.get::<_, Option<String>>(i) {
                                Ok(Some(s)) => s,
                                Ok(None) => "NULL".to_string(),
                                Err(_) => match row.get::<_, Option<i64>>(i) {
                                    Ok(Some(n)) => n.to_string(),
                                    Ok(None) => "NULL".to_string(),
                                    Err(_) => match row.get::<_, Option<f64>>(i) {
                                        Ok(Some(f)) => f.to_string(),
                                        Ok(None) => "NULL".to_string(),
                                        Err(_) => "<unknown>".to_string(),
                                    },
                                },
                            },
                        };
                        values.push(value);
                    }
                    Ok(values)
                })?;

                for (idx, values) in rows.enumerate() {
                    let values = values?;
                    println!("  Row {}: {:?}", idx + 1, values);
                }
            }
        }
    }

    Ok(())
}
