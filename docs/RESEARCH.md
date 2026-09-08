# 技术调研报告 - Realm 数据库访问方案

## 调研日期
2026-09-08

## 调研目标
确定在 Rust 中访问和操作 osu!lazer Realm 数据库的最佳方案。

## 发现的 Rust Crates

### 1. realm-codec v0.1.1 ⭐ **推荐方案**

**链接：** https://crates.io/crates/realm-codec  
**仓库：** https://github.com/gloiiire/pinkha  
**文档：** https://docs.rs/realm-codec

**描述：**
- 纯 Rust 实现的 Realm 二进制数据库文件解析器和写入器
- 支持 Realm v9 文件格式
- 可以读取表、构建新文件
- MIT OR Apache-2.0 双许可证

**优势：**
- ✅ 纯 Rust，无需 FFI
- ✅ 直接操作二进制文件，无需依赖 Realm SDK
- ✅ 轻量级
- ✅ 可以读写操作

**劣势：**
- ⚠️ 版本较新 (0.1.1)，可能不稳定
- ⚠️ 需要验证是否支持 osu!lazer 使用的 Realm 版本
- ⚠️ 文档可能不够完善
- ⚠️ 需要手动处理 schema 映射

**评估：**
- 需要立即测试是否能打开 osu!lazer 的 client.realm 文件
- 需要验证是否能读取 BeatmapSetInfo 等对象
- 如果可行，这是最理想的方案

### 2. realm-web-rs v0.1.0

**链接：** https://crates.io/crates/realm-web-rs  
**仓库：** https://github.com/codecrafter404/realm-web-rs

**描述：**
- 实现了 realm-web npm package 的 Rust 版本
- 用于 MongoDB Realm（云服务）

**评估：**
- ❌ 不适用 - 这是为 MongoDB Realm 云服务设计的，不是本地 Realm 数据库

### 3. 其他 "realm" crates

通过 `cargo search realm` 发现的其他 crates：
- `realm` - Rust/Elm 全栈 web 框架（不相关）
- `realm-cli` - Docker 沙箱环境（不相关）
- `realm-rs` - 2D RPG 工具包（不相关）

**结论：** 没有其他适用的 crates

## 备选方案

### 方案A：使用 realm-codec ⭐ **首选**

**实施步骤：**
1. 添加依赖 `realm-codec = "0.1"`
2. 尝试打开 osu!lazer 的 client.realm 文件
3. 解析 schema 并映射到 Rust 结构体
4. 实现读写操作

**优势：**
- 纯 Rust，类型安全
- 无需额外安装 Realm SDK
- 完全控制底层操作

**风险：**
- Crate 可能不成熟
- 可能需要贡献代码修复 bug
- 需要深入理解 Realm 文件格式

**时间估计：** 1-2 周验证可行性

### 方案B：FFI 绑定 Realm C++ Core

**Realm Core 仓库：** https://github.com/realm/realm-core

**实施步骤：**
1. 下载并编译 Realm C++ Core
2. 使用 `rust-bindgen` 生成 Rust FFI 绑定
3. 创建安全的 Rust 封装层
4. 实现内存管理和错误处理

**优势：**
- 使用官方 SDK，稳定可靠
- 功能完整
- 有完整文档

**劣势：**
- 需要 C++ 编译环境
- FFI 增加复杂性
- 内存安全需要仔细处理
- 跨平台构建复杂

**时间估计：** 2-3 周

### 方案C：直接解析 Realm 文件格式

**参考资料：**
- [Realm Protocol Documentation](https://github.com/realm/realm-core/blob/master/doc/protocol.md)
- [Realm File Format Research](https://link.springer.com/chapter/10.1007/978-3-030-98467-0_8)

**实施步骤：**
1. 研究 Realm 文件格式规范
2. 手动实现二进制解析器
3. 实现 B+ 树等数据结构

**评估：**
- ❌ 不推荐 - 过于复杂且容易出错
- 这实际上就是重新实现 realm-codec

## osu!lazer Realm Schema 分析

根据网络调研，osu!lazer 使用以下主要对象：

### 关键类型

1. **BeatmapSetInfo** / **BeatmapSet**
   - 包含一组 beatmap 的元数据
   - 引用 `RealmFile` 对象
   
2. **BeatmapInfo** / **Beatmap**
   - 单个难度的信息
   - 属于一个 BeatmapSet

3. **SkinInfo** / **Skin**
   - 皮肤元数据
   - 引用皮肤文件

4. **BeatmapCollection**
   - 收藏夹
   - 包含 beatmap 引用列表

5. **RealmFile** ⭐ **关键**
   - 文件引用对象
   - 包含 SHA-256 哈希值
   - 多个对象可以引用同一个 `RealmFile`

**参考资料：**
- [ppy/osu Beatmap class terminology](https://github.com/ppy/osu/wiki/Beatmap-class-terminology)
- [osu!lazer Realm 读取示例](https://gist.github.com/taoky/a98233b13a16dd4a50cef202fe46b298)

### Schema 版本

osu!lazer 的 Realm schema 会随版本更新。需要：
1. 实现 schema 版本检测
2. 支持多个版本
3. 提供版本兼容性警告

## 哈希存储机制分析

### 文件存储结构

```
osu!lazer 数据目录/
├── client.realm           # Realm 数据库
├── files/                 # 哈希文件存储
│   ├── {hash[0]}/
│   │   ├── {hash[0..2]}/
│   │   │   └── {full_hash}
```

**示例：**
- 文件哈希：`1a2b3c4d5e6f...`
- 存储路径：`files/1/1a/1a2b3c4d5e6f...`

### RealmFile 对象结构（推测）

```rust
struct RealmFile {
    hash: String,           // SHA-256 哈希
    size: i64,             // 文件大小（字节）
    // 可能还有其他字段
}
```

### 引用计数机制（推测）

- 多个对象可以引用同一个 `RealmFile`
- 删除操作需要检查引用计数
- 只有当没有任何引用时才应该删除物理文件

**关键实现：**
```rust
fn delete_file_if_unreferenced(db: &RealmDatabase, hash: &str) -> Result<()> {
    // 1. 查询所有引用此哈希的对象
    let references = db.count_file_references(hash)?;
    
    // 2. 只有在没有引用时才删除
    if references == 0 {
        let path = hash_to_path(hash);
        fs::remove_file(path)?;
    }
    
    Ok(())
}
```

## 推荐实施方案

### 阶段 0.1：realm-codec 可行性验证 (3-5 天)

**目标：** 确认 realm-codec 是否能用于 osu!lazer

**任务：**
1. 创建测试项目
2. 添加 `realm-codec` 依赖
3. 尝试打开真实的 osu!lazer `client.realm` 文件
4. 尝试读取至少一个表
5. 尝试解析 `BeatmapSetInfo` 或其他简单对象

**成功标准：**
- ✅ 能打开文件不崩溃
- ✅ 能列出所有表
- ✅ 能读取至少一条记录

**如果失败：** 转到方案B（FFI 绑定）

### 阶段 0.2：Schema 映射 (3-4 天)

**任务：**
1. 分析 osu!lazer 的 Realm schema
2. 定义 Rust 结构体映射
3. 实现序列化/反序列化
4. 测试读取各种对象

**交付物：**
- `src/models/` 下的所有数据模型
- 单元测试

### 阶段 0.3：哈希存储验证 (2-3 天)

**任务：**
1. 实现 `hash/storage.rs`
2. 验证文件路径生成逻辑
3. 测试 SHA-256 计算
4. 实现引用计数查询

**交付物：**
- `hash/` 模块
- 完整性检查工具

## 依赖更新

基于调研结果，更新 `Cargo.toml`：

```toml
[dependencies]
# Realm 数据库访问
realm-codec = "0.1"

# 错误处理
thiserror = "2.0"
anyhow = "1.0"

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# UUID 支持
uuid = { version = "1.0", features = ["v4", "serde"] }

# 时间处理
chrono = { version = "0.4", features = ["serde"] }

# 哈希计算
sha2 = "0.10"

# 文件操作
walkdir = "2.0"
fs_extra = "1.3"

# 日志
log = "0.4"

[dev-dependencies]
# 测试框架
pretty_assertions = "1.4"
tempfile = "3.0"
# 用于测试的 Realm 文件生成
test-case = "3.0"
```

## 风险评估

### 高风险
1. **realm-codec 不兼容** (概率: 30%)
   - 缓解：准备方案B（FFI）
   
2. **Schema 版本不匹配** (概率: 20%)
   - 缓解：支持多版本，提供版本检测

### 中风险
3. **哈希存储理解不正确** (概率: 15%)
   - 缓解：详细测试，参考其他项目

4. **引用计数逻辑复杂** (概率: 10%)
   - 缓解：保守策略，宁可不删除文件

## 下一步行动

1. **立即：** 创建 `realm-codec` 可行性验证项目
2. **本周：** 完成阶段 0.1
3. **如果成功：** 继续阶段 0.2 和 0.3
4. **如果失败：** 启动方案B（FFI 绑定）

## 参考资源

**Realm 相关：**
- [Realm Database Wikipedia](https://en.wikipedia.org/wiki/Realm_(database))
- [Realm Core Protocol](https://github.com/realm/realm-core/blob/master/doc/protocol.md)
- [Realm File Format Research](https://link.springer.com/chapter/10.1007/978-3-030-98467-0_8)

**osu!lazer 相关：**
- [ppy/osu Beatmap class terminology](https://github.com/ppy/osu/wiki/Beatmap-class-terminology)
- [osu!lazer Realm 读取示例](https://gist.github.com/taoky/a98233b13a16dd4a50cef202fe46b298)
- [osu-lazer-db-reader](https://github.com/yadPe/osu-lazer-db-reader)

**Rust FFI：**
- [rust-bindgen](https://github.com/rust-lang/rust-bindgen)
- [Rust FFI Guide](https://github.com/Michael-F-Bryan/rust-ffi-guide)
