use no_realm::RealmReader;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let db_path = env::args()
        .nth(1)
        .unwrap_or_else(|| {
            dirs::data_local_dir()
                .map(|p| p.join("osu/client.realm"))
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "client.realm".to_string())
        });

    println!("Analyzing Realm file: {}", db_path);
    println!();

    let mut reader = RealmReader::open(&db_path)?;
    println!("✅ Realm file opened");
    println!("   File format version: {}", reader.file_format_version());
    println!();

    // 分析 schema
    println!("Analyzing schema...");
    let schema = reader.analyze_schema()?;

    println!("Found {} tables:", schema.tables.len());
    for (name, table) in &schema.tables {
        println!("\n📦 Table: {}", name);
        println!("   Fields: {}", table.fields.len());

        // 显示前 10 个字段
        for (i, field) in table.fields.iter().take(10).enumerate() {
            println!("   {}. {} ({})", i + 1, field.name, field.field_type);
        }

        if table.fields.len() > 10 {
            println!("   ... and {} more", table.fields.len() - 10);
        }
    }

    println!("\n");
    println!("Finding all strings...");
    let strings = reader.find_strings()?;

    // 过滤并显示看起来像字段名的字符串
    let field_names: Vec<_> = strings.iter()
        .filter(|s| {
            s.len() > 3
            && s.len() < 30
            && s.chars().next().unwrap().is_uppercase()
            && s.chars().all(|c| c.is_alphanumeric())
        })
        .take(50)
        .collect();

    println!("Potential field names:");
    for (i, name) in field_names.iter().enumerate() {
        print!("  {}", name);
        if (i + 1) % 5 == 0 {
            println!();
        } else {
            print!(",  ");
        }
    }
    println!("\n");

    Ok(())
}
