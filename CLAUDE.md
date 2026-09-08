# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

no-realm 是一个用 Rust 开发的库，用于读取和修改 osu!lazer 的 Realm 数据库。该项目旨在为第三方软件提供一个安全的接口来操作 osu!lazer 的游戏数据。

**修改边界（严格限制）：**
- ✅ 删除 lazer 铺面（beatmaps）
- ✅ 创建、修改、删除 lazer 皮肤（skins）
- ✅ 创建、修改、删除 lazer 收藏夹（collections）
- ❌ **绝对禁止**修改铺面内容本身

## 🔴 CRITICAL: 安全第一原则

> **核心理念：宁可操作失败，也不能损坏数据库**

### 数据损坏风险

osu!lazer 使用 **Realm 数据库 + SHA-256 哈希文件存储**的双层架构：
- Realm 存储元数据和文件引用（`RealmFile` 对象）
- 实际文件存储在 `files/` 目录，文件名为其 SHA-256 哈希值
- 不一致的操作会导致**永久性数据损坏**

### 必须遵守的规则

1. **阶段0优先** - 在实现任何业务功能前，必须先完成备份和安全机制
2. **操作前备份** - 所有写操作必须通过 `SafetyGuard` 或 `safe_operation` 进行
3. **哈希一致性** - 删除顺序：先删 Realm 引用，后删文件；添加顺序：先写文件，后建引用
4. **完整性验证** - 每次操作后验证数据库和文件系统的一致性
5. **详细文档** - 所有安全相关代码必须有详细注释说明为什么这样做

详细信息请阅读 [SAFETY.md](./SAFETY.md)

## Development Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run a specific test
cargo test <test_name>

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Run with verbose output
cargo test -- --nocapture
```

## Architecture Notes

This is a library crate intended to be used by third-party applications. The architecture is organized around safety:

### Module Priority Order

1. **backup/** (🔒 阶段0) - 备份创建、验证、恢复、策略管理
2. **hash/** (🔐 阶段0) - 哈希文件存储管理和验证
3. **safety/** (🛡️ 阶段0) - RAII 安全守卫和操作验证
4. **realm/** - Realm 数据库连接和事务
5. **models/** - 数据模型定义（Beatmap, Skin, Collection, RealmFile）
6. **operations/** - 业务操作接口

### Key Design Patterns

**RAII Safety Guard:**
```rust
{
    let _guard = SafetyGuard::new(db.path(), &BackupStrategy::default())?;
    // 执行危险操作
    operation()?;
    _guard.commit(); // 成功才提交
} // 失败时 Drop 自动恢复备份
```

**Safe Operation Wrapper:**
```rust
safe_operation(&db, &BackupStrategy::default(), |db| {
    // 所有操作都在备份保护下
    operations::beatmap::delete(db, id)?;
    Ok(())
})?; // 失败自动回滚
```

## Important Constraints

- This is a **library crate**, not a binary application
- All database modifications must be transactional and safe
- Never expose APIs that allow modifying beatmap file contents
- **Always use SafetyGuard for write operations**
- **Always verify hash consistency for file operations**
- Provide clear error messages in both English and Chinese

## Testing Strategy

### Test Hierarchy
1. Unit tests - 单个函数的正确性
2. Integration tests - 模块间交互
3. **Safety tests** (最重要) - 损坏场景和恢复测试

### Required Test Scenarios
- ✅ Normal backup and restore
- ✅ Database corruption detection
- ✅ Panic during operation (auto-recovery)
- ✅ Hash consistency validation
- ✅ Orphaned file detection
- ✅ Missing file detection
- ✅ Partial operation failure

## Current Status

🚧 Phase 0 (In Progress): Safety and Backup Infrastructure

See [PLAN.md](./PLAN.md) for the full development roadmap.
