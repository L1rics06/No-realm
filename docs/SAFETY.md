# 安全架构设计文档

> **核心理念：宁可操作失败，也不能损坏数据**

## 问题分析

### osu!lazer 的数据存储架构

```
osu! lazer 数据目录
├── client.realm          # Realm 数据库（元数据）
├── client.realm.lock     # 数据库锁文件
├── files/                # 实际文件存储
│   ├── 1/
│   │   ├── 1a/
│   │   │   └── 1a2b3c...  # 文件名=SHA256哈希
│   │   └── 1b/
│   └── 2/
└── ...
```

### 双层架构的风险

#### 风险1：不一致删除
```
❌ 错误顺序：
1. 删除 files/ 中的文件
2. [系统崩溃/程序错误]
3. Realm 中仍有引用 → osu! 无法找到文件 → 崩溃

✅ 正确顺序：
1. 删除 Realm 中的引用
2. [即使崩溃，文件仍在，只是孤立]
3. 删除 files/ 中的文件
```

#### 风险2：哈希计算错误
```
❌ 错误流程：
1. 创建文件 foo.osk
2. 计算哈希 = abc123
3. 重命名文件为 abc123
4. 在 Realm 中创建引用 (hash=abc123)
5. [但文件内容实际是 def456] → 数据损坏

✅ 正确流程：
1. 创建临时文件
2. 边写入边流式计算 SHA256
3. 写入完成后得到最终哈希
4. 移动文件到 files/{hash[0]}/{hash[0..2]}/{hash}
5. 在 Realm 中创建引用
6. 验证：读取文件，重新计算哈希，对比
```

#### 风险3：部分失败
```
场景：批量删除 100 个 beatmaps
❌ 不安全：
- 删除了 50 个后崩溃
- 无法知道哪些已删除，哪些未删除
- 无法回滚

✅ 安全方案：
- 每次操作前创建备份
- 使用事务（全部成功或全部失败）
- 失败时自动恢复备份
```

## 安全机制设计

### 1. 备份模块 (`backup/`)

#### 备份策略
```rust
pub struct BackupStrategy {
    /// 保留的备份数量
    pub retain_count: usize,
    
    /// 备份存储位置
    pub backup_dir: PathBuf,
    
    /// 是否验证备份完整性
    pub verify_after_backup: bool,
    
    /// 是否包含 files/ 目录的快照
    pub include_files: bool,
}

impl Default for BackupStrategy {
    fn default() -> Self {
        Self {
            retain_count: 5,
            backup_dir: PathBuf::from("backups/"),
            verify_after_backup: true,
            include_files: false, // 只备份 Realm，文件太大
        }
    }
}
```

#### 备份文件命名
```
backups/
├── client.realm.backup.20260908T143022Z.a1b2c3d4
├── client.realm.backup.20260908T145511Z.e5f6g7h8
├── client.realm.backup.20260908T150033Z.i9j0k1l2
└── manifest.json  # 备份元数据
```

#### 备份元数据
```json
{
  "backups": [
    {
      "id": "a1b2c3d4",
      "timestamp": "2026-09-08T14:30:22Z",
      "operation": "delete_beatmap",
      "realm_size": 52428800,
      "realm_sha256": "abc123...",
      "verified": true,
      "can_restore": true
    }
  ]
}
```

#### 备份验证流程
```rust
pub struct BackupVerification {
    /// 文件是否存在
    pub exists: bool,
    
    /// 文件大小是否匹配
    pub size_matches: bool,
    
    /// SHA256 是否匹配
    pub hash_matches: bool,
    
    /// Realm 是否可以打开
    pub realm_openable: bool,
    
    /// 关键表是否存在
    pub tables_exist: bool,
}

impl Backup {
    pub fn verify(&self) -> Result<BackupVerification> {
        // 1. 检查文件存在
        // 2. 验证文件大小
        // 3. 计算并验证 SHA256
        // 4. 尝试打开 Realm (只读)
        // 5. 检查关键表 (BeatmapSet, Skin, etc.)
    }
}
```

### 2. 哈希一致性模块 (`hash/`)

#### 文件存储管理
```rust
pub struct HashStorage {
    /// files/ 根目录
    root: PathBuf,
}

impl HashStorage {
    /// 存储文件并返回哈希
    pub fn store_file(&self, source: &Path) -> Result<String> {
        // 1. 流式计算 SHA256
        let hash = self.calculate_sha256(source)?;
        
        // 2. 确定目标路径
        let dest = self.hash_to_path(&hash);
        
        // 3. 创建目录结构
        fs::create_dir_all(dest.parent().unwrap())?;
        
        // 4. 复制文件
        fs::copy(source, &dest)?;
        
        // 5. 验证复制后的文件哈希
        let verify_hash = self.calculate_sha256(&dest)?;
        if verify_hash != hash {
            fs::remove_file(&dest)?;
            return Err(Error::HashMismatch { ... });
        }
        
        Ok(hash)
    }
    
    /// 验证文件哈希
    pub fn verify_file(&self, hash: &str) -> Result<bool> {
        let path = self.hash_to_path(hash);
        let actual_hash = self.calculate_sha256(&path)?;
        Ok(actual_hash == hash)
    }
    
    /// 删除文件
    pub fn remove_file(&self, hash: &str) -> Result<()> {
        // 只有在 Realm 中没有引用时才删除
        let path = self.hash_to_path(hash);
        fs::remove_file(path)?;
        Ok(())
    }
    
    /// 查找孤立文件（有文件但无 Realm 引用）
    pub fn find_orphaned_files(&self, db: &RealmDatabase) -> Result<Vec<String>> {
        // 1. 遍历 files/ 目录获取所有文件
        // 2. 查询 Realm 中的所有 RealmFile
        // 3. 找出差集
    }
}
```

#### 完整性验证器
```rust
pub struct IntegrityCheck {
    pub total_files: usize,
    pub verified_files: usize,
    pub missing_files: Vec<String>,      // Realm 引用但文件不存在
    pub orphaned_files: Vec<String>,     // 文件存在但无 Realm 引用
    pub corrupted_files: Vec<String>,    // 哈希不匹配
}

impl HashStorage {
    pub fn check_integrity(&self, db: &RealmDatabase) -> Result<IntegrityCheck> {
        // 1. 获取 Realm 中所有 RealmFile
        // 2. 验证每个文件是否存在
        // 3. 验证每个文件的哈希
        // 4. 查找孤立文件
    }
}
```

### 3. 安全操作框架 (`safety/`)

#### 安全守卫（RAII 模式）
```rust
pub struct SafetyGuard {
    backup: Backup,
    db_path: PathBuf,
    committed: bool,
}

impl SafetyGuard {
    pub fn new(db_path: &Path, strategy: &BackupStrategy) -> Result<Self> {
        // 1. 创建备份
        let backup = Backup::create(db_path, strategy)?;
        
        // 2. 验证备份
        if strategy.verify_after_backup {
            backup.verify()?.ensure_valid()?;
        }
        
        Ok(Self {
            backup,
            db_path: db_path.to_path_buf(),
            committed: false,
        })
    }
    
    pub fn commit(mut self) {
        self.committed = true;
        // 不恢复备份
    }
}

impl Drop for SafetyGuard {
    fn drop(&mut self) {
        if !self.committed {
            // 操作失败或 panic，恢复备份
            eprintln!("操作未提交，正在恢复备份...");
            if let Err(e) = self.backup.restore(&self.db_path) {
                eprintln!("⚠️ 备份恢复失败: {}", e);
                eprintln!("备份位置: {}", self.backup.path().display());
            }
        }
    }
}
```

#### 安全操作包装器
```rust
pub fn safe_operation<T, F>(
    db: &RealmDatabase,
    strategy: &BackupStrategy,
    operation: F,
) -> Result<T>
where
    F: FnOnce(&RealmDatabase) -> Result<T>,
{
    // 1. 创建安全守卫（自动备份）
    let guard = SafetyGuard::new(db.path(), strategy)?;
    
    // 2. 执行操作
    let result = operation(db)?;
    
    // 3. 验证完整性
    db.verify_integrity()?;
    
    // 4. 提交（告诉 guard 不要恢复）
    guard.commit();
    
    Ok(result)
}
```

#### 使用示例
```rust
// 用户代码
let db = RealmDatabase::open("path/to/client.realm")?;

// 方式1：使用安全包装器
safe_operation(&db, &BackupStrategy::default(), |db| {
    operations::beatmap::delete(db, beatmap_id)?;
    Ok(())
})?;

// 方式2：手动管理
{
    let _guard = SafetyGuard::new(db.path(), &BackupStrategy::default())?;
    operations::beatmap::delete(&db, beatmap_id)?;
    _guard.commit();
} // 如果 delete 失败，这里会自动恢复

// 方式3：批量操作
safe_operation(&db, &BackupStrategy::default(), |db| {
    for id in beatmap_ids {
        operations::beatmap::delete(db, id)?;
    }
    Ok(())
})?; // 全部成功或全部回滚
```

### 4. 操作验证器 (`safety/validator.rs`)

```rust
pub trait Validator {
    fn validate_before(&self, db: &RealmDatabase) -> Result<()>;
    fn validate_after(&self, db: &RealmDatabase) -> Result<()>;
}

pub struct DeleteBeatmapValidator {
    beatmap_id: Uuid,
}

impl Validator for DeleteBeatmapValidator {
    fn validate_before(&self, db: &RealmDatabase) -> Result<()> {
        // 1. Beatmap 是否存在
        // 2. 是否被 Collection 引用（警告但允许）
        // 3. 相关文件是否存在
        Ok(())
    }
    
    fn validate_after(&self, db: &RealmDatabase) -> Result<()> {
        // 1. Beatmap 已被删除
        // 2. RealmFile 引用已删除
        // 3. 如果是最后一个引用，文件已删除
        // 4. Collection 中的引用已清理
        Ok(())
    }
}
```

## 实施检查清单

### 阶段0：备份与安全 (2周)
- [ ] 实现 `backup/mod.rs` - 备份创建
- [ ] 实现 `backup/strategy.rs` - 备份策略和清理
- [ ] 实现 `backup/restore.rs` - 恢复和验证
- [ ] 实现 `backup/integrity.rs` - 完整性检查
- [ ] 实现 `hash/storage.rs` - 文件存储管理
- [ ] 实现 `hash/validator.rs` - 哈希验证
- [ ] 实现 `safety/guard.rs` - RAII 安全守卫
- [ ] 实现 `safety/validator.rs` - 操作验证
- [ ] 编写测试：正常备份恢复
- [ ] 编写测试：损坏场景模拟
- [ ] 编写测试：panic 时自动恢复
- [ ] 编写测试：哈希一致性检查
- [ ] 编写测试：孤立文件检测
- [ ] 文档：安全操作使用指南

### 验收标准
✅ 所有测试场景下都不会丢失数据
✅ 备份可以成功恢复
✅ 哈希一致性检查正常工作
✅ SafetyGuard 在 panic 时正确恢复

## 用户可见的安全功能

### 1. 完整性检查命令
```rust
let check = db.verify_integrity()?;
println!("总文件数: {}", check.total_files);
println!("验证通过: {}", check.verified_files);
println!("丢失文件: {}", check.missing_files.len());
println!("孤立文件: {}", check.orphaned_files.len());
println!("损坏文件: {}", check.corrupted_files.len());
```

### 2. 清理孤立文件
```rust
let orphaned = db.find_orphaned_files()?;
println!("发现 {} 个孤立文件，占用 {} MB", 
         orphaned.len(), 
         orphaned.total_size_mb());

// 用户确认后清理
db.clean_orphaned_files()?;
```

### 3. 手动备份和恢复
```rust
// 创建手动备份
let backup = db.create_backup_with_name("before_major_cleanup")?;

// 查看所有备份
let backups = db.list_backups()?;

// 恢复特定备份
db.restore_backup(&backup.id)?;
```

## 性能考虑

### 备份性能
- Realm 文件通常 50-500MB
- 备份操作（复制）约需 0.5-5 秒
- 不备份 files/ 目录（太大，且可重新下载）
- 增量备份？（未来优化）

### 哈希计算性能
- SHA256 计算速度：约 200-500 MB/s
- 大型皮肤文件（50MB）约需 0.1-0.25 秒
- 使用流式计算，不占用大量内存

### 建议
- 默认 SafetyLevel::Safe（每次操作都备份）
- 批量操作时用户可选择 SafetyLevel::Balanced
- 提供 `--skip-backup` 选项（仅供高级用户，显示警告）

## 总结

通过这个安全架构：
1. ✅ **永不丢失数据** - 操作前自动备份
2. ✅ **可以恢复** - 失败时自动回滚
3. ✅ **可以验证** - 完整性检查
4. ✅ **可以清理** - 安全删除孤立文件
5. ✅ **哈希一致性** - 双重验证

**开发顺序：先实现安全机制，再实现业务功能。**
