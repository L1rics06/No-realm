# SQLite 导出指南

本指南说明如何将 osu!lazer 的 Realm 数据库导出为 SQLite 格式，以便 no-realm 库读取。

---

## 为什么需要导出？

osu!lazer 使用 Realm 数据库存储游戏数据。目前 Rust 生态中没有成熟的 Realm 绑定，因此 no-realm 采用以下策略：

- **短期方案**: 读取导出的 SQLite 数据库
- **长期目标**: 实现直接的 Realm 访问

---

## 方法 1: 使用 Realm Studio (推荐)

### 步骤

1. **下载 Realm Studio**
   
   访问 [Realm Studio 下载页面](https://www.mongodb.com/docs/realm-studio/)
   
   或直接下载：
   - Windows: [Realm Studio.exe](https://studio-releases.realm.io/latest/download/win)
   - macOS: [Realm Studio.dmg](https://studio-releases.realm.io/latest/download/mac)
   - Linux: [Realm Studio.AppImage](https://studio-releases.realm.io/latest/download/linux)

2. **找到 osu!lazer 数据库**

   数据库位置：
   ```bash
   # Linux
   ~/.local/share/osu/client.realm
   
   # macOS
   ~/Library/Application Support/osu/client.realm
   
   # Windows
   %AppData%\osu\client.realm
   ```

3. **关闭 osu!lazer**
   
   ⚠️ **重要**: 必须先关闭 osu!lazer，否则数据库文件被锁定

4. **用 Realm Studio 打开数据库**
   
   - 启动 Realm Studio
   - File → Open Realm File
   - 选择 `client.realm`

5. **导出为 SQLite**
   
   - File → Export to...
   - 选择 "SQLite" 格式
   - 保存为 `client.sqlite`

6. **验证导出**
   
   使用 SQLite 工具验证：
   ```bash
   sqlite3 client.sqlite ".tables"
   ```
   
   应该看到类似这样的表：
   ```
   BeatmapCollection
   BeatmapInfo
   BeatmapMetadata
   BeatmapSetInfo
   RealmFile
   SkinInfo
   ...
   ```

---

## 方法 2: 使用命令行工具

### realm-cli (如果可用)

```bash
# 安装 Realm CLI
npm install -g realm-cli

# 导出
realm-cli export \
  --input ~/.local/share/osu/client.realm \
  --output client.sqlite \
  --format sqlite
```

---

## 方法 3: 使用 C# 脚本 (开发者)

如果你熟悉 C#，可以使用 `Realm.NET` SDK 编写导出脚本：

```csharp
using Realms;
using System.Data.SQLite;

var config = new RealmConfiguration("client.realm");
var realm = Realm.GetInstance(config);

// 导出逻辑...
```

参考: [kabiiQ/BeatmapExporter](https://github.com/kabiiQ/BeatmapExporter)

---

## 在 no-realm 中使用

### 基本用法

```rust
use no_realm::sqlite::SqliteDatabase;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 打开导出的 SQLite 数据库
    let db = SqliteDatabase::open("client.sqlite")?;
    
    // 查询数据
    let beatmaps = no_realm::sqlite::query_beatmap_sets(&db)?;
    
    println!("找到 {} 个 beatmap sets", beatmaps.len());
    
    Ok(())
}
```

---

## 数据同步

### ⚠️ 注意事项

- 导出的 SQLite 数据库是**快照**，不会自动更新
- 如果 osu!lazer 中的数据发生变化，需要重新导出
- 建议定期重新导出以保持数据最新

### 自动化导出

可以创建一个脚本定期导出：

```bash
#!/bin/bash
# export-realm.sh

# 1. 关闭 osu!lazer (可选)
pkill osu

# 2. 等待进程结束
sleep 2

# 3. 导出
realm-studio export \
  ~/.local/share/osu/client.realm \
  ~/.local/share/osu/client.sqlite

# 4. 验证
if [ -f ~/.local/share/osu/client.sqlite ]; then
    echo "导出成功"
else
    echo "导出失败"
    exit 1
fi
```

---

## 故障排除

### 问题 1: 无法打开 realm 文件

**错误**: `Unable to open Realm file`

**原因**: osu!lazer 正在运行，文件被锁定

**解决方案**: 完全关闭 osu!lazer，包括后台进程

### 问题 2: 导出的文件为空

**错误**: 导出的 SQLite 文件没有数据

**原因**: 导出工具版本不兼容

**解决方案**: 
1. 更新 Realm Studio 到最新版本
2. 尝试其他导出方法

### 问题 3: 表结构不匹配

**错误**: `no such table: BeatmapSetInfo`

**原因**: osu!lazer 版本更新，数据库结构变化

**解决方案**: 
1. 检查 no-realm 版本是否支持当前 osu!lazer 版本
2. 查看 [版本兼容性](../README.md#版本兼容性)

---

## 未来改进

no-realm 的长期目标是支持**直接读取 Realm 文件**，无需导出步骤。

跟踪进度: [Issue #1 - Realm C++ SDK 集成](https://github.com/L1rics/no-realm/issues/1)

---

## 参考资料

- [Realm Studio 文档](https://www.mongodb.com/docs/realm-studio/)
- [osu!lazer 文件存储说明](https://github.com/ppy/osu/wiki/User-file-storage)
- [SQLite 官方网站](https://www.sqlite.org/)

---

## 需要帮助？

如果遇到问题，请：

1. 查看 [FAQ](../FAQ.md)
2. 搜索 [已有 Issue](https://github.com/L1rics/no-realm/issues)
3. 创建新的 Issue 并附上：
   - osu!lazer 版本
   - 错误信息
   - 导出方法
