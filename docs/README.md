# No-realm 文档索引

本目录包含项目的所有技术文档。

## 📋 文档列表

### 核心文档

- **[IMPLEMENTATION_STATUS.md](./IMPLEMENTATION_STATUS.md)** - 实现状态报告 ⭐ **最新**
  - 功能实现进度（60%）
  - 测试状态（86 个测试通过）
  - 已知问题和解决方案
  - 下一步计划

- **[SAFETY.md](./SAFETY.md)** - 安全架构设计文档 ⭐ **必读**
  - 数据损坏风险分析
  - 安全机制设计（备份、进程检测、安全守卫）
  - 完整的代码示例和实施清单

- **[RESEARCH.md](./RESEARCH.md)** - 技术调研报告
  - Realm 数据库访问方案对比
  - Realm 文件格式分析
  - osu!lazer Schema 分析

- **[SQLITE_EXPORT_GUIDE.md](./SQLITE_EXPORT_GUIDE.md)** - SQLite 导出指南
  - 如何从 Realm 导出数据到 SQLite
  - 数据迁移方案

- **[SUMMARY.md](./SUMMARY.md)** - 早期技术调研总结（历史文档）
  - realm-codec 可行性验证
  - 安全优先的开发计划

## 🔍 快速导航

### 新用户入门
1. 先读 [IMPLEMENTATION_STATUS.md](./IMPLEMENTATION_STATUS.md) 了解当前进度
2. 再读 [SAFETY.md](./SAFETY.md) 理解安全机制
3. 查看 [RESEARCH.md](./RESEARCH.md) 了解技术细节

### 开发者指南
1. [IMPLEMENTATION_STATUS.md](./IMPLEMENTATION_STATUS.md) - 查看功能状态和待办事项
2. [SAFETY.md](./SAFETY.md) - 参考安全实现模式
3. [RESEARCH.md](./RESEARCH.md) - 了解 Realm 格式细节

## 📊 项目状态

**当前版本：** 0.1.0  
**完成度：** 约 60%  
**测试覆盖：** 86 个单元测试全部通过

### 已实现 ✅
- Realm 文件格式解析（Modern24）
- Schema 解析
- 对象反序列化器
- Beatmap 读取功能
- Collection 读取功能（基础）
- 备份系统
- 进程检测
- 安全守卫机制

### 进行中 🔄
- Collection 创建/修改/删除
- Link 和 LinkList 完整解析
- 时间戳转换修复

### 计划实现 📅
- Beatmap 删除功能
- 完整的 CRUD API
- 性能优化

## 🔗 外部参考

### Realm 相关
- [Realm Core File Format](https://github.com/realm/realm-core/blob/master/doc/file_format.md)
- [Realm Database Wikipedia](https://en.wikipedia.org/wiki/Realm_(database))

### osu!lazer 相关
- [ppy/osu Repository](https://github.com/ppy/osu)
- [Beatmap class terminology](https://github.com/ppy/osu/wiki/Beatmap-class-terminology)
- [osu!lazer Realm 读取示例](https://gist.github.com/taoky/a98233b13a16dd4a50cef202fe46b298)

## 📝 文档更新日志

- 2024-09-10: 添加 IMPLEMENTATION_STATUS.md，完整的功能状态报告
- 2024-09-08: 初始文档创建（SAFETY.md, RESEARCH.md, SUMMARY.md）

