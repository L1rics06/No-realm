# No-realm 开发计划

## 项目架构设计

### 核心模块划分

```
no-realm/
├── src/
│   ├── lib.rs              # 主库入口
│   ├── backup/             # 🔒 备份模块（最高优先级）
│   │   ├── mod.rs
│   │   ├── strategy.rs     # 备份策略（保留数量、清理等）
│   │   ├── restore.rs      # 恢复与验证
│   │   └── integrity.rs    # 完整性检查
│   ├── hash/               # 🔐 哈希一致性模块
│   │   ├── mod.rs
│   │   ├── storage.rs      # 文件存储管理
│   │   └── validator.rs    # 哈希验证
│   ├── realm/              # Realm数据库操作模块
│   │   ├── mod.rs
│   │   ├── connection.rs   # 数据库连接管理
│   │   └── transaction.rs  # 事务处理（带自动回滚）
│   ├── models/             # 数据模型定义
│   │   ├── mod.rs
│   │   ├── beatmap.rs      # Beatmap相关模型
│   │   ├── skin.rs         # Skin相关模型
│   │   ├── collection.rs   # Collection相关模型
│   │   └── file.rs         # RealmFile模型（哈希映射）
│   ├── operations/         # 数据库操作接口
│   │   ├── mod.rs
│   │   ├── beatmap.rs      # Beatmap操作（只读+删除）
│   │   ├── skin.rs         # Skin操作（完整CRUD）
│   │   └── collection.rs   # Collection操作（完整CRUD）
│   ├── safety/             # 🛡️ 安全检查模块
│   │   ├── mod.rs
│   │   ├── validator.rs    # 操作前验证
│   │   └── guard.rs        # 安全守卫（RAII模式）
│   ├── error.rs            # 错误类型定义
│   └── utils.rs            # 工具函数
└── tests/
    ├── integration/        # 集成测试
    ├── fixtures/           # 测试数据
    └── backup/             # 备份恢复测试
```

### 依赖关系

需要添加的核心依赖：
- `realm` - Realm数据库绑定（需要调研Rust生态中是否有可用的）
- `serde` - 序列化/反序列化
- `thiserror` - 错误处理
- `uuid` - UUID生成
- `chrono` - 时间处理
- `sha2` - SHA-256哈希计算和验证
- `walkdir` - 文件系统遍历
- `fs_extra` - 高级文件操作（复制、备份）

备选方案（如果没有直接的Realm绑定）：
- 通过FFI调用C++的Realm SDK
- 或者研究Realm的底层存储格式，直接操作文件

## 安全性设计原则

### 🔴 第一原则：永不信任自己
所有写操作都假设可能失败，必须有回滚机制。

### 🔴 第二原则：Realm + 哈希存储双重一致性
- Realm 记录文件引用（`RealmFile` 对象）
- 实际文件存储在 `files/` 目录下，文件名为 SHA-256 哈希值
- **任何删除操作**：先删除 Realm 引用，确认成功后再删除文件
- **任何添加操作**：先写入文件，计算哈希，然后在 Realm 中创建引用
- **修改操作**：视为删除旧引用 + 添加新引用

### 🔴 第三原则：操作前必须备份
```rust
// 每个写操作的标准流程
pub fn safe_operation<T, F>(db: &RealmDatabase, operation: F) -> Result<T>
where
    F: FnOnce(&RealmDatabase) -> Result<T>,
{
    // 1. 创建备份
    let backup = db.create_backup()?;
    
    // 2. 验证备份可用性
    backup.verify()?;
    
    // 3. 执行操作（在事务中）
    let result = operation(db);
    
    match result {
        Ok(value) => {
            // 4. 成功：验证数据库完整性
            db.verify_integrity()?;
            Ok(value)
        }
        Err(e) => {
            // 5. 失败：自动恢复
            backup.restore()?;
            Err(e)
        }
    }
}
```

### 🔴 第四原则：备份策略
- **保留数量**：默认保留最近 5 个备份
- **自动清理**：超过数量时删除最旧的备份
- **命名规则**：`client.realm.backup.{timestamp}.{uuid}`
- **元数据**：记录备份时间、操作类型、文件大小
- **验证机制**：
  - 文件完整性（大小、哈希）
  - Realm 可读性（能否打开）
  - 关键表存在性检查

### 🔴 第五原则：渐进式安全等级
```rust
pub enum SafetyLevel {
    Paranoid,    // 每次操作都备份，保留所有备份
    Safe,        // 每次操作都备份，保留5个备份（默认）
    Balanced,    // 批量操作只备份一次，保留3个备份
    Fast,        // 只在会话开始时备份一次
}
```

## 开发阶段（重新规划）

### 🔒 阶段0：安全与备份基础设施 (2周) ⭐ **最高优先级**

**目标：** 在进行任何数据库操作前，建立完善的安全机制

**任务：**
1. **备份模块实现**
   - 备份创建：复制 `client.realm` 和相关文件
   - 备份验证：确保备份可以成功打开和读取
   - 备份恢复：安全地替换损坏的数据库
   - 备份策略：管理备份数量、清理旧备份
   
2. **完整性检查**
   - Realm 数据库完整性验证
   - 哈希文件一致性检查（Realm 引用 vs 实际文件）
   - 孤立文件检测（有文件但无 Realm 引用）
   - 丢失文件检测（有 Realm 引用但文件不存在）
   
3. **安全操作框架**
   - RAII 模式的安全守卫
   - 自动回滚事务
   - 操作前后的完整性验证
   
4. **测试验证**
   - 模拟数据库损坏场景
   - 验证备份恢复流程
   - 测试各种失败情况

**交付物：**
- `backup/` 模块完整实现
- `safety/` 模块完整实现
- 备份和恢复的集成测试
- 安全操作使用文档

**验收标准：**
- ✅ 能创建数据库备份并验证其完整性
- ✅ 能检测并恢复损坏的数据库
- ✅ 所有测试场景下都不会丢失数据
- ✅ 备份策略正常工作（保留指定数量）

### 阶段1：技术调研与基础设施 (1-2周)

**目标：** 确定技术方案，搭建项目基础

**任务：**
1. 调研Rust中Realm数据库的可用方案
   - 检查是否有realm-rs等现成的crate
   - 评估使用FFI绑定C++ Realm SDK的可行性
   - 研究是否可以通过低层协议直接读写Realm文件
   
2. 分析osu!lazer的Realm schema和哈希存储
   - 参考 [ppy/osu Beatmap class terminology](https://github.com/ppy/osu/wiki/Beatmap-class-terminology)
   - 理解 `RealmFile` 对象的结构
   - 研究 `files/` 目录的组织方式
   - 分析 BeatmapSetInfo, SkinInfo, BeatmapCollection 的依赖关系
   
3. 搭建项目基础结构
   - 配置Cargo.toml依赖
   - 设置错误处理框架
   - 建立测试框架

**交付物：**
- 技术方案选型文档
- 哈希存储机制分析文档
- 基础项目结构
- 核心数据模型定义

### 阶段2：Realm数据库访问层 (2-3周)


### 阶段2：Realm数据库访问层 (2-3周)

**目标：** 实现安全的Realm数据库读写

**任务：**
1. 实现数据库连接管理
   - 打开osu!lazer的client.realm文件
   - 实现连接池（如果需要）
   - 处理文件锁和并发访问
   
2. 实现事务支持
   - 读事务
   - 写事务
   - 事务回滚机制
   
3. 实现基础查询接口
   - 按ID查询
   - 条件查询
   - 列表查询

**交付物：**
- realm模块完整实现
- 单元测试覆盖
- 使用示例

### 阶段3：Beatmap操作 (1周)

**目标：** 实现Beatmap的只读和删除操作

**任务：**
1. 实现Beatmap数据模型映射
2. 实现读取操作
   - 列出所有beatmap sets
   - 根据ID查询beatmap
   - 根据条件过滤beatmap
3. 实现删除操作
   - 删除单个beatmap
   - 删除beatmap set
   - 级联删除相关数据

**限制：**
- 禁止修改beatmap文件内容
- 只提供元数据读取
- 删除操作需要确认机制

**交付物：**
- operations/beatmap.rs完整实现
- 集成测试
- API文档

### 阶段4：Skin操作 (1-2周)

**目标：** 实现Skin的完整CRUD操作

**任务：**
1. 实现Skin数据模型
2. 实现CRUD操作
   - Create: 创建新skin
   - Read: 读取skin信息
   - Update: 修改skin配置
   - Delete: 删除skin
3. 实现skin文件管理
   - 处理skin文件存储
   - 维护哈希索引

**交付物：**
- operations/skin.rs完整实现
- 集成测试
- API文档

### 阶段5：Collection操作 (1周)

**目标：** 实现Collection的完整CRUD操作

**任务：**
1. 实现Collection数据模型
2. 实现CRUD操作
   - Create: 创建collection
   - Read: 读取collection
   - Update: 修改collection（添加/删除beatmap）
   - Delete: 删除collection
3. 实现collection与beatmap的关联管理

**交付物：**
- operations/collection.rs完整实现
- 集成测试
- API文档

### 阶段6：文档与示例 (1周)

**目标：** 完善文档和示例代码

**任务：**
1. 编写完整的API文档
2. 提供使用示例
   - 基础用法示例
   - 进阶用法示例
   - 错误处理示例
3. 编写中英双语文档
4. 创建FAQ

**交付物：**
- 完整的rustdoc文档
- examples/目录下的示例代码
- 更新README.md

## 技术难点与风险

### 🔴 CRITICAL - 数据安全风险
**风险：** Realm + 哈希存储的双层架构，操作不当会导致数据库永久损坏
**影响：** 用户丢失所有游戏数据
**缓解措施：**
- ✅ 阶段0优先实现备份和恢复机制
- ✅ 所有写操作都在安全守卫保护下执行
- ✅ 操作前后进行完整性验证
- ✅ 详细的操作日志用于问题诊断

### 🔴 CRITICAL - 哈希一致性
**风险：** Realm 引用的文件哈希与实际文件不匹配
**场景示例：**
- 删除 Beatmap 但文件仍存在 → 磁盘空间浪费
- 删除文件但 Realm 引用仍存在 → osu! 崩溃或错误
- 文件被外部修改但哈希未更新 → 数据不一致

**缓解措施：**
- ✅ 实现 `hash/validator.rs` 进行一致性检查
- ✅ 删除操作：先删 Realm，后删文件
- ✅ 添加操作：先写文件，计算哈希，再建引用
- ✅ 提供 `verify_integrity()` 命令给用户

### 1. Realm数据库绑定
**难点：** Rust生态中可能没有成熟的Realm绑定
**解决方案：**
- 优先寻找现有crate
- 考虑使用FFI调用C++ SDK
- 最后考虑研究Realm文件格式直接操作

### 2. 并发访问控制
**难点：** osu!lazer运行时可能正在使用数据库
**解决方案：**
- 实现文件锁检测
- 提供只读模式
- 建议用户在osu!关闭时操作

### 3. 数据完整性
**难点：** 确保修改不会破坏数据库
**解决方案：**
- 所有写操作使用事务
- 实现数据验证
- 提供数据库备份建议

### 4. Schema版本兼容性
**难点：** osu!lazer的schema可能会更新
**解决方案：**
- 实现schema版本检测
- 支持多个schema版本
- 提供版本兼容性文档

## 参考资源

基于网络调研，以下资源可能有帮助：

1. [osu!lazer Realm读取示例](https://gist.github.com/taoky/a98233b13a16dd4a50cef202fe46b298) - 展示了如何读取osu Realm数据库
2. [ppy/osu Beatmap class terminology](https://github.com/ppy/osu/wiki/Beatmap-class-terminology) - 官方的Beatmap类说明
3. [osu-lazer-db-reader](https://github.com/yadPe/osu-lazer-db-reader) - 另一个Realm读取项目
4. [BeatmapExporter](https://github.com/kabiiQ/BeatmapExporter) - osu!lazer文件导出工具
5. [CollectionDowngrader](https://github.com/PinNaCode/CollectionDowngrader) - Collection格式转换工具

## 下一步行动

1. 立即：完善README.md，添加项目说明和安装指南
2. 本周：开始阶段1的技术调研
3. 创建GitHub Issues跟踪各个阶段的任务
4. 设置CI/CD流程（GitHub Actions）
