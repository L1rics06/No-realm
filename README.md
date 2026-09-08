# No-realm

<div align="center">

**一个安全、易用的 osu!lazer Realm 数据库操作库**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)

[English](#english) | [中文](#中文)

</div>

---

## 中文

### 这是什么？

No-realm 是一个用 Rust 编写的库，用于读取和修改 osu!lazer 的 Realm 数据库。该项目为第三方软件提供了一个安全、类型安全的接口来操作 osu!lazer 的游戏数据，而无需直接处理底层 Realm 数据库的复杂性。

### 功能特性

- ✅ **Beatmap 管理** - 列出、查询和删除 beatmap sets
- ✅ **皮肤管理** - 完整的 CRUD 操作（创建、读取、更新、删除）
- ✅ **收藏夹管理** - 完整的 CRUD 操作
- 🔒 **类型安全** - 利用 Rust 的类型系统确保数据完整性
- ⚡ **高性能** - 零成本抽象和高效的数据访问
- 🛡️ **事务支持** - 所有写操作都在事务中进行，保证数据一致性

### 修改边界

> [!IMPORTANT]
> 该库**不会**提供修改 beatmap 文件内容的功能！

出于数据完整性考虑，该库的修改范围严格限制为：

| 操作类型 | Beatmap | 皮肤 (Skin) | 收藏夹 (Collection) |
|---------|---------|------------|-------------------|
| 创建 (Create) | ❌ | ✅ | ✅ |
| 读取 (Read) | ✅ | ✅ | ✅ |
| 更新 (Update) | ❌ | ✅ | ✅ |
| 删除 (Delete) | ✅ | ✅ | ✅ |

### 安装

将以下内容添加到你的 `Cargo.toml`：

```toml
[dependencies]
no-realm = "0.1"
```

### 快速开始

```rust
use no_realm::{RealmDatabase, operations};

// 打开 osu!lazer 数据库
let db = RealmDatabase::open("/path/to/osu/client.realm")?;

// 列出所有 beatmap sets
let beatmaps = operations::beatmap::list_all(&db)?;
println!("找到 {} 个 beatmap sets", beatmaps.len());

// 创建一个新收藏夹
let collection = operations::collection::create(&db, "My Favorites")?;

// 添加 beatmap 到收藏夹
operations::collection::add_beatmap(&db, &collection.id, &beatmap_id)?;
```

### 项目状态

🚧 **开发中** - 该项目目前处于早期开发阶段。API 可能会发生变化。

查看 [PLAN.md](PLAN.md) 了解详细的开发计划和路线图。

### 贡献

> Our team fully supports and encourages the use of **AI/LLM tools** in the contribution process. Contributors are welcome to use AI for drafting, coding, documentation, debugging, reviewing, research, or any other part of an issue report or pull request.

我们的团队完全支持并鼓励在贡献过程中使用 **AI/LLM 工具**。贡献者可以自由使用 AI 来辅助编写代码、撰写文档、调试、代码审查、资料检索，以及创建 Issue 或 Pull Request 的其他任何环节。

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

### 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

### 致谢

- [ppy/osu](https://github.com/ppy/osu) - osu!lazer 游戏本体
- Realm Database - 提供强大的移动数据库解决方案

---

## English

### What is this?

No-realm is a Rust library for reading and modifying osu!lazer's Realm database. This project provides third-party software with a safe, type-safe interface to manipulate osu!lazer game data without dealing with the complexity of the underlying Realm database.

### Features

- ✅ **Beatmap Management** - List, query, and delete beatmap sets
- ✅ **Skin Management** - Full CRUD operations (Create, Read, Update, Delete)
- ✅ **Collection Management** - Full CRUD operations
- 🔒 **Type Safety** - Leverages Rust's type system to ensure data integrity
- ⚡ **High Performance** - Zero-cost abstractions and efficient data access
- 🛡️ **Transaction Support** - All write operations are transactional for data consistency

### Modification Boundaries

> [!IMPORTANT]
> This library does **NOT** provide functionality to modify beatmap file contents!

For data integrity, modification scope is strictly limited to:

| Operation | Beatmap | Skin | Collection |
|-----------|---------|------|------------|
| Create | ❌ | ✅ | ✅ |
| Read | ✅ | ✅ | ✅ |
| Update | ❌ | ✅ | ✅ |
| Delete | ✅ | ✅ | ✅ |

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
no-realm = "0.1"
```

### Quick Start

```rust
use no_realm::{RealmDatabase, operations};

// Open osu!lazer database
let db = RealmDatabase::open("/path/to/osu/client.realm")?;

// List all beatmap sets
let beatmaps = operations::beatmap::list_all(&db)?;
println!("Found {} beatmap sets", beatmaps.len());

// Create a new collection
let collection = operations::collection::create(&db, "My Favorites")?;

// Add beatmap to collection
operations::collection::add_beatmap(&db, &collection.id, &beatmap_id)?;
```

### Project Status

🚧 **Under Development** - This project is in early development. APIs may change.

See [PLAN.md](PLAN.md) for the detailed development plan and roadmap.

### Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### Acknowledgments

- [ppy/osu](https://github.com/ppy/osu) - The osu!lazer game itself
- Realm Database - For providing a powerful mobile database solution
