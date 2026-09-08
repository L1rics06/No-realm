# no-realm 使用指南

## 快速开始

### 1. 读取 Realm 数据库

```rust
use no_realm::{RealmDatabase, RealmReader};

fn main() -> no_realm::Result<()> {
    // 打开数据库
    let db = RealmDatabase::open("~/.local/share/osu/client.realm")?;
    println!("Database size: {} bytes", db.size()?);

    // 使用 RealmReader 分析结构
    let mut reader = RealmReader::open(db.path())?;
    println!("Format version: {}", reader.file_format_version());

    // 分析 Schema
    let schema = reader.analyze_schema()?;
    for (name, table) in &schema.tables {
        println!("Table: {}, Fields: {}", name, table.fields.len());
    }

    Ok(())
}
```

### 2. 查询数据（接口已定义）

```rust
use no_realm::RealmDatabase;

fn main() -> no_realm::Result<()> {
    let db = RealmDatabase::open("client.realm")?;
    let query = db.query()?;

    // 列出所有表（可用）
    let tables = query.list_tables()?;
    println!("Tables: {:?}", tables);

    // 查询皮肤（TODO：实现中）
    let skins = query.query_skins()?;
    
    // 查询收藏夹（TODO：实现中）
    let collections = query.query_collections()?;

    Ok(())
}
```

### 3. 修改数据（接口已定义）

```rust
use no_realm::{RealmDatabase, models::SkinInfo};
use no_realm::safety::safe_operation;
use no_realm::backup::BackupStrategy;

fn main() -> no_realm::Result<()> {
    let db = RealmDatabase::open("client.realm")?;

    // ⚠️ 所有修改都应该使用 safe_operation
    safe_operation(&db, &BackupStrategy::default(), |db| {
        let mut mutation = db.mutation()?;

        // 创建新皮肤（TODO：实现中）
        let skin = SkinInfo::new("My Skin".into(), Some("Author".into()));
        mutation.insert_skin(&skin)?;

        Ok(())
    })?;

    Ok(())
}
```

## 数据模型

### RealmFile - 文件引用

```rust
use no_realm::models::RealmFile;

let file = RealmFile::new(
    "abc123...".to_string(),  // SHA-256 哈希
    1024,                      // 文件大小
);

// 获取文件存储路径
println!("{}", file.storage_path());
// 输出: files/ab/c1/abc123...
```

### SkinInfo - 皮肤信息

```rust
use no_realm::models::SkinInfo;

let mut skin = SkinInfo::new("Skin Name".into(), Some("Author".into()));

// 检查是否已删除
if !skin.is_deleted() {
    println!("Skin is active");
}
```

### BeatmapCollection - 收藏夹

```rust
use no_realm::models::BeatmapCollection;

let mut collection = BeatmapCollection::new("Favorites".into());

// 添加 beatmap
collection.add_beatmap("md5_hash".to_string());

// 检查是否包含
if collection.contains_beatmap("md5_hash") {
    println!("Collection contains this beatmap");
}

// 移除 beatmap
collection.remove_beatmap("md5_hash");

println!("Collection size: {}", collection.size());
```

## 安全机制

### SafetyGuard - RAII 保护

```rust
use no_realm::safety::SafetyGuard;
use no_realm::backup::BackupStrategy;

{
    let _guard = SafetyGuard::new(
        db.path(),
        &BackupStrategy::default()
    )?;

    // 执行危险操作
    perform_operation()?;

    // 手动提交
    _guard.commit();
} // 如果 commit 未调用，Drop 会自动恢复
```

### safe_operation - 函数式包装

```rust
use no_realm::safety::safe_operation;

safe_operation(&db, &BackupStrategy::default(), |db| {
    // 所有操作都在备份保护下
    delete_beatmap(db, &id)?;
    Ok(())
})?; // 失败自动回滚
```

## 运行示例

```bash
# 分析 Realm 文件结构
cargo run --example analyze_realm ~/.local/share/osu/client.realm

# 查询数据库
cargo run --example query_db ~/.local/share/osu/client.realm

# 读取基础信息
cargo run --example read_db ~/.local/share/osu/client.realm
```

## 当前限制

1. **查询功能不完整** - 接口已定义，但返回空数据
2. **修改功能不完整** - 接口已定义，但操作尚未实现
3. **Realm 解析** - 目前使用启发式方法，需要完整的二进制解析

## 开发路线图

详见 [PROGRESS.md](./PROGRESS.md)

## 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test models
cargo test safety

# 运行集成测试
cargo test --test integration
```

## 许可证

MIT License
