# No-realm 文档索引

本目录包含项目的所有技术文档。

## 📋 文档列表

### 核心文档

- **[SUMMARY.md](./SUMMARY.md)** - 技术调研总结与安全策略
  - 调研成果概述
  - 安全优先的开发计划
  - 下一步行动

- **[SAFETY.md](./SAFETY.md)** - 安全架构设计文档 ⭐ **必读**
  - 数据损坏风险分析
  - 双层架构（Realm + 哈希存储）详解
  - 安全机制设计（备份、哈希一致性、安全守卫）
  - 完整的代码示例和实施清单

- **[RESEARCH.md](./RESEARCH.md)** - 技术调研报告
  - Realm 数据库访问方案对比
  - realm-codec 详细分析
  - osu!lazer Schema 分析
  - 哈希存储机制研究

- **[PLAN.md](./PLAN.md)** - 开发计划和路线图
  - 6 个开发阶段详细规划
  - 技术难点与解决方案
  - 时间估计和交付物

## 🔍 快速导航

### 新手入门
1. 先读 [SUMMARY.md](./SUMMARY.md) 了解项目概况
2. 再读 [SAFETY.md](./SAFETY.md) 理解为什么安全如此重要
3. 查看 [PLAN.md](./PLAN.md) 了解开发路线

### 技术实现
1. [RESEARCH.md](./RESEARCH.md) - 了解技术选型
2. [SAFETY.md](./SAFETY.md) - 参考安全实现模式
3. [PLAN.md](./PLAN.md) - 查看当前阶段任务

## 📊 项目状态

当前阶段：**阶段 0 - 安全与备份基础设施**

- [ ] realm-codec 可行性验证
- [ ] 备份模块实现
- [ ] 哈希一致性模块实现
- [ ] 安全操作框架实现

## 🔗 外部参考

### Realm 相关
- [Realm Database Wikipedia](https://en.wikipedia.org/wiki/Realm_(database))
- [Realm Core Protocol](https://github.com/realm/realm-core/blob/master/doc/protocol.md)
- [realm-codec crate](https://crates.io/crates/realm-codec)

### osu!lazer 相关
- [ppy/osu Beatmap class terminology](https://github.com/ppy/osu/wiki/Beatmap-class-terminology)
- [osu!lazer Realm 读取示例](https://gist.github.com/taoky/a98233b13a16dd4a50cef202fe46b298)
- [osu-lazer-db-reader](https://github.com/yadPe/osu-lazer-db-reader)

### Rust FFI（备用方案）
- [rust-bindgen](https://github.com/rust-lang/rust-bindgen)
- [Rust FFI Guide](https://github.com/Michael-F-Bryan/rust-ffi-guide)
