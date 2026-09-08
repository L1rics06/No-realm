# Realm 集成技术调研

**调研日期**: 2026-09-08  
**目标**: 为 no-realm 选择合适的 Realm 数据库访问方案

---

## 调研结果

### 1. Rust 生态现状

❌ **没有可用的 Realm Rust 绑定**

通过 `cargo search` 搜索发现：
- 没有 `realm-rust`、`realm-core`、`realm-database` 等官方或非官方绑定
- 搜索结果中的 "realm" 都是其他项目（Web 框架、P2P 工具等）

### 2. 其他语言的 Realm 绑定

根据 [Realm GitHub 组织](https://github.com/realm)：

- ✅ **realm-swift** - Objective-C/Swift (16.6k stars)
- ✅ **realm-java** - Java/Android (11.5k stars)
- ✅ **realm-kotlin** - Kotlin Multiplatform (1.1k stars)
- ✅ **realm-js** - TypeScript/JavaScript (6k stars)
- ✅ **realm-dart** - Dart/Flutter (官方支持)

**但没有 Rust 官方支持** ❌

### 3. osu!lazer 的 Realm 使用

根据 [ppy/osu Wiki - User file storage](https://github.com/ppy/osu/wiki/User-file-storage):

> All files that are imported to lazer are stored under filenames that reflect their SHA-256 hashes, in the `~/.local/share/osu` (or OS equivalent) folder. Mappings to these files are held inside a **client realm database** (filename `client.realm`).

**关键发现**:
- osu!lazer 使用 C# 的 `Realm.NET` SDK
- 数据库文件名: `client.realm`
- 文件存储: SHA-256 哈希命名
- 存储位置: `~/.local/share/osu/files/`

### 4. 现有的 Rust 项目案例

找到一个相关项目：[yadPe/osu-lazer-db-reader](https://github.com/yadPe/osu-lazer-db-reader)

**但这是 C# 项目**，使用 `Realm.NET`。目前没有找到 Rust 实现的案例。

---

## 可行方案分析

### 方案 A: 使用 realm-dart FFI 方法 ⚠️

**参考**: [nhachicha/realm-dart-ffi](https://github.com/nhachicha/realm-dart-ffi)

**思路**: 
- Realm 有 C++ 核心引擎
- 通过 C FFI 包装
- 使用 `rust-bindgen` 生成 Rust 绑定

**优点**:
- ✅ 直接访问 Realm 核心功能
- ✅ 性能最好
- ✅ 完整功能支持

**缺点**:
- ❌ 需要手动编写 C 包装层
- ❌ 编译复杂（需要 Realm C++ SDK）
- ❌ 跨平台兼容性问题
- ❌ 维护成本高

**复杂度**: 🔴 极高

---

### 方案 B: Realm 导出为 SQLite 格式 ✅ **推荐**

**发现**: Realm 数据库可以导出为 SQLite 格式

**思路**:
1. 用户先将 `client.realm` 导出为 `client.sqlite`
2. 使用 `rusqlite` 读取 SQLite 数据库
3. 提供简单的转换工具

**优点**:
- ✅ 使用成熟的 `rusqlite` crate
- ✅ 实现简单，无需 C++ FFI
- ✅ 跨平台兼容性好
- ✅ 维护成本低
- ✅ 可以读取数据

**缺点**:
- ⚠️ 需要额外的导出步骤
- ⚠️ 无法直接写入 Realm（需要转换回去）
- ⚠️ 数据可能不同步

**复杂度**: 🟡 中等

**适用场景**: 只读操作、批量导出、数据分析

---

### 方案 C: 通过 C# 互操作 🔵

**思路**:
1. 编写 C# 包装库使用 `Realm.NET`
2. 导出为 C 风格 DLL
3. Rust 通过 FFI 调用

**优点**:
- ✅ 使用官方 `Realm.NET` SDK
- ✅ 功能完整
- ✅ 可以读写

**缺点**:
- ❌ 需要 .NET Runtime
- ❌ 增加依赖
- ❌ 跨语言调用开销
- ❌ 部署复杂

**复杂度**: 🟠 高

---

### 方案 D: 逆向工程 Realm 文件格式 🔴 **不推荐**

**思路**: 分析 `.realm` 文件的二进制格式，自己实现解析器

**优点**:
- ✅ 无外部依赖

**缺点**:
- ❌ Realm 格式复杂且未公开
- ❌ 可能违反 Realm 许可证
- ❌ 版本兼容性问题
- ❌ 实现和维护成本极高

**复杂度**: 🔴 极高

**结论**: 不可行

---

### 方案 E: 混合方案 - Rust + Python ⭐ **推荐**

**思路**:
1. 使用 Python 的 `realm-python` (如果存在) 或通过 `PyO3`
2. Rust 通过 `PyO3` 调用 Python 代码
3. 或者提供 Python CLI 工具配合 Rust 库

**检查**: Realm 是否有 Python 绑定？

---

## 最终推荐方案

### 🎯 短期方案 (MVP): SQLite 导出 + rusqlite

**实现步骤**:

1. **提供转换工具**
   ```bash
   # 用户使用 Realm Studio 或其他工具导出
   realm-studio export client.realm --format sqlite --output client.sqlite
   ```

2. **Rust 库读取 SQLite**
   ```rust
   // 使用 rusqlite 读取导出的数据库
   let conn = Connection::open("client.sqlite")?;
   let beatmaps = query_beatmaps(&conn)?;
   ```

3. **限制功能**
   - ✅ 列出 Beatmap
   - ✅ 查询和搜索
   - ⚠️ 删除操作（需要转换回 Realm）

**时间估计**: 1-2 周

---

### 🚀 长期方案: 直接 Realm 访问

**需要进一步调研**:

1. **检查 Realm C++ SDK**
   - 是否可以编译为静态库
   - C API 是否稳定

2. **使用 bindgen 生成绑定**
   ```bash
   bindgen realm.h -o src/realm_ffi.rs
   ```

3. **实现薄包装层**
   ```rust
   mod realm_ffi; // 生成的绑定
   
   pub struct RealmDatabase {
       handle: *mut realm_ffi::Realm,
   }
   ```

**时间估计**: 1-2 个月

---

## 决策建议

### 立即行动

1. ✅ **采用 SQLite 方案作为 MVP**
   - 快速实现基本功能
   - 验证数据模型正确性
   - 为用户提供价值

2. 🔍 **并行调研 Realm C++ SDK**
   - 下载 Realm C++ SDK
   - 测试 C API 可用性
   - 评估 bindgen 可行性

3. 📝 **更新文档**
   - 说明当前限制
   - 提供 SQLite 导出指南
   - 标注未来改进计划

---

## 参考资料

### Realm 官方资源
- [Realm GitHub Organization](https://github.com/realm)
- [Realm C++ SDK](https://github.com/realm/realm-core)

### osu!lazer 资源
- [ppy/osu Wiki - User file storage](https://github.com/ppy/osu/wiki/User-file-storage)
- [kabiiQ/BeatmapExporter](https://github.com/kabiiQ/BeatmapExporter) - C# 实现的导出工具

### Rust FFI 资源
- [rust-bindgen](https://github.com/rust-lang/rust-bindgen)
- [nhachicha/realm-dart-ffi](https://github.com/nhachicha/realm-dart-ffi) - Dart FFI 实现参考

### 数据库访问
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite Rust 绑定

---

## 下一步行动

### 本周任务

1. **实现 SQLite 读取器** (2-3 天)
   - [ ] 添加 `rusqlite` 依赖
   - [ ] 分析 SQLite 导出的表结构
   - [ ] 实现 `list_all()` 查询
   - [ ] 测试真实数据

2. **创建转换工具指南** (1 天)
   - [ ] 编写 SQLite 导出步骤
   - [ ] 提供示例脚本
   - [ ] 更新 README

3. **验证概念** (1 天)
   - [ ] 用真实 osu!lazer 数据库测试
   - [ ] 验证数据模型正确性
   - [ ] 记录发现的问题

---

**总结**: 采用 SQLite 方案作为快速迭代的起点，同时为未来的直接 Realm 访问保留架构灵活性。
