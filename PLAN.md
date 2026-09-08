# 开发计划

## 项目概览

no-realm 是一个用 Rust 开发的库，用于安全地读取和修改 osu!lazer 的 Realm 数据库。

## 开发阶段

### ✅ 阶段 0: 安全基础设施 (已完成)

**目标**: 建立完整的备份、恢复和安全机制，确保所有操作都在保护下进行。

#### 0.1 错误处理 ✅
- [x] 定义统一的错误类型
- [x] 实现错误转换和传播
- [x] 添加详细的错误信息

#### 0.2 备份系统 ✅
- [x] 实现 `BackupManager`
- [x] 支持备份创建、验证、恢复
- [x] 实现备份策略（保留数量、压缩等）
- [x] SHA-256 完整性验证
- [x] 备份清理机制

#### 0.3 哈希文件管理 ✅
- [x] 实现 `hash` 模块基础结构
- [x] 文件存储路径计算
- [x] 哈希一致性验证（占位）

#### 0.4 安全守卫 ✅
- [x] 实现 RAII `SafetyGuard`
- [x] 自动备份和恢复
- [x] Panic 安全保证
- [x] `safe_operation` 便捷函数
- [x] 完整的测试覆盖（提交、回滚、panic 恢复）

**测试状态**: 37 个测试全部通过 ✅

---

### ✅ 阶段 1: 数据模型 (已完成)

**目标**: 定义所有核心数据结构。

#### 1.1 基础模型 ✅
- [x] `RealmFile` - 文件引用和存储路径
- [x] `BeatmapMetadata` - Beatmap 元数据
- [x] `BeatmapInfo` - 单个 Beatmap 信息
- [x] `BeatmapSetInfo` - BeatmapSet 完整信息
- [x] `SkinInfo` - 皮肤信息
- [x] `BeatmapCollection` - 收藏夹信息

#### 1.2 模型方法 ✅
- [x] `RealmFile::storage_path()` - 计算存储路径
- [x] `SkinInfo::new()` / `is_deleted()` - 皮肤创建和状态
- [x] `BeatmapCollection` CRUD 方法（add/remove/contains）
- [x] 完整的单元测试

---

### ✅ 阶段 2: 操作接口（框架）(已完成)

**目标**: 定义操作接口，为后续 Realm 集成做准备。

#### 2.1 Beatmap 操作 ✅
- [x] `list_all()` - 列出所有 BeatmapSet
- [x] `get_by_id()` - 根据 UUID 查询
- [x] `get_by_online_id()` - 根据在线 ID 查询
- [x] `search()` - 搜索功能
- [x] `delete()` - 安全删除（带备份）
- [x] `soft_delete()` - 软删除
- [x] 接口测试

#### 2.2 Skin 操作 ✅
- [x] `list_all()` / `get_by_id()` / `search()`
- [x] `create()` - 创建皮肤（带备份）
- [x] `update()` - 更新皮肤（带备份）
- [x] `delete()` - 删除皮肤（带备份）
- [x] 接口测试

#### 2.3 Collection 操作 ✅
- [x] `list_all()` / `get_by_id()` / `get_by_name()`
- [x] `create()` - 创建收藏夹（带备份）
- [x] `rename()` - 重命名（带备份）
- [x] `add_beatmap()` / `remove_beatmap()` - 管理内容（带备份）
- [x] `delete()` - 删除收藏夹（带备份）
- [x] 接口测试

**当前状态**: 所有操作接口已定义，返回占位实现。所有接口都已集成 `SafetyGuard` 保护。

---

### 🚧 阶段 3: Realm 集成 (下一步)

**目标**: 实现真正的 Realm 数据库访问。

#### 3.1 技术选型
- [ ] 调研 Rust Realm 绑定选项
  - 选项 A: realm-rust (官方，但可能不成熟)
  - 选项 B: 通过 C++ FFI 调用 Realm SDK
  - 选项 C: 使用 SQLite 模式（Realm 可导出为 SQLite）
- [ ] 确定技术方案
- [ ] 创建概念验证（PoC）

#### 3.2 基础集成
- [ ] 实现 Realm 连接管理
- [ ] 实现基础查询
- [ ] 实现写入操作
- [ ] 事务支持

#### 3.3 完整实现
- [ ] 实现所有 Beatmap 操作
- [ ] 实现所有 Skin 操作
- [ ] 实现所有 Collection 操作
- [ ] 集成测试（真实数据库）

---

### 📋 阶段 4: 哈希文件操作 (待定)

**目标**: 实现文件系统操作，管理 `files/` 目录。

#### 4.1 文件存储
- [ ] 计算 SHA-256 哈希
- [ ] 写入文件到 `files/` 目录
- [ ] 创建 `RealmFile` 引用

#### 4.2 文件删除
- [ ] 删除 `RealmFile` 引用
- [ ] 删除物理文件
- [ ] 孤儿文件检测和清理

#### 4.3 完整性验证
- [ ] 验证哈希一致性
- [ ] 检测缺失文件
- [ ] 检测损坏文件
- [ ] 自动修复功能

---

### 🎯 阶段 5: 高级功能 (未来)

#### 5.1 导入导出
- [ ] 导出 BeatmapSet 为 .osz
- [ ] 导入 .osz 文件
- [ ] 导出/导入皮肤 (.osk)

#### 5.2 批量操作
- [ ] 批量删除 Beatmap
- [ ] 批量导入
- [ ] 批量修改收藏夹

#### 5.3 性能优化
- [ ] 查询优化
- [ ] 缓存策略
- [ ] 异步操作支持

---

## 测试策略

### 当前测试覆盖

| 模块 | 单元测试 | 集成测试 | 状态 |
|------|---------|---------|------|
| `error` | ✅ 2 | - | 完成 |
| `backup` | ✅ 11 | - | 完成 |
| `safety` | ✅ 5 | - | 完成 |
| `models` | ✅ 3 | - | 完成 |
| `operations::beatmap` | ✅ 3 | ⏳ | 接口完成 |
| `operations::skin` | ✅ 2 | ⏳ | 接口完成 |
| `operations::collection` | ✅ 2 | ⏳ | 接口完成 |
| `realm` | ✅ 5 | ⏳ | 基础完成 |
| `hash` | ⏳ | ⏳ | 待实现 |

**总计**: 37 个测试，全部通过 ✅

### 测试优先级

1. **安全测试** (最高优先级) ✅
   - 备份和恢复
   - Panic 恢复
   - 数据一致性

2. **功能测试** (进行中) 🚧
   - CRUD 操作
   - 边界条件
   - 错误处理

3. **集成测试** (待实现) ⏳
   - 真实数据库操作
   - 多步骤事务
   - 并发安全

---

## 技术债务

### 当前已知问题

1. **未使用的导入** (低优先级)
   - `backup/integrity.rs`: 未使用的 `Error`
   - `backup/restore.rs`: 未使用的 `std::io::Write`
   - `realm/connection.rs`: 未使用的 `warn`, `PathBuf`
   - 修复: `cargo fix --lib`

2. **未读取的字段** (低优先级)
   - `BackupManager::manifest_path` - 预留给未来的元数据功能

3. **Realm 集成缺失** (高优先级)
   - 所有操作当前返回占位数据
   - 需要选择 Realm 绑定方案

---

## 依赖项

```toml
[dependencies]
chrono = "0.4"           # 时间处理
log = "0.4"              # 日志
serde = "1.0"            # 序列化
serde_json = "1.0"       # JSON 支持
sha2 = "0.10"            # SHA-256 哈希
uuid = "1.0"             # UUID 支持
thiserror = "1.0"        # 错误派生

[dev-dependencies]
tempfile = "3.8"         # 临时文件测试
```

---

## 下一步行动

### 立即任务

1. **清理警告** ⚡
   ```bash
   cargo fix --lib
   cargo clippy --fix --lib
   ```

2. **技术调研** 🔍
   - 调研 Realm Rust 绑定
   - 测试 realm-rust crate
   - 评估 C++ FFI 可行性

3. **文档完善** 📝
   - 更新 README 反映当前进度
   - 添加架构图
   - 编写贡献指南

### 短期目标 (1-2 周)

- [ ] 完成 Realm 技术选型
- [ ] 实现基础 Realm 查询
- [ ] 实现一个完整的端到端功能（例如：列出所有 Beatmap）

### 中期目标 (1 个月)

- [ ] 完成所有 CRUD 操作
- [ ] 实现文件管理功能
- [ ] 达到 80% 测试覆盖率

---

## 贡献指南

当前欢迎以下贡献：

1. **Realm 集成** - 帮助实现数据库访问
2. **测试** - 添加更多测试用例
3. **文档** - 改进文档和示例
4. **Bug 修复** - 修复已知问题

---

## 许可证

MIT License
