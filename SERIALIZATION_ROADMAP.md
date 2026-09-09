# Realm 序列化/反序列化实现路线图

## 已完成的工作 ✅

### 1. 基础设施
- ✅ **类型系统** (`src/realm/format/types.rs`)
  - `RealmRef`: 8字节对齐的文件偏移引用
  - `TaggedValue`: 内联整数值 (LSB=1)
  - `RefOrTagged`: 引用或标签值的区分

- ✅ **数组解析器** (`src/realm/format/array.rs`)
  - `ArrayHeader`: 8字节数组头解析
  - `ArrayView`: 零拷贝数组访问
  - 支持 1/2/4/8/16/32/64 位元素宽度
  - 支持 bit-packed 数组

- ✅ **文件头解析** (`src/realm/format/header.rs`)
  - 24字节 Realm 头部解析
  - 版本检测 (v9 legacy, v24 modern)
  - 加密检测和拒绝
  - MVCC top_ref 选择

- ✅ **反序列化器骨架** (`src/realm/format/deserializer.rs`)
  - `Deserializer`: 上下文持有者
  - `Schema/TableDef/FieldDef`: Schema 结构定义
  - `RealmObject`: 反序列化的对象表示
  - `Value`: 支持所有 Realm 基础类型
  - 可以访问表根节点

- ✅ **序列化器骨架** (`src/realm/format/serializer.rs`)
  - `Serializer`: 输出缓冲区管理
  - 基础类型写入 (u8/u16/u32/u64/i64/f32/f64)
  - 数组头写入
  - 字符串/二进制/列表序列化
  - 8字节对齐支持

### 2. 实际数据库分析
通过 `examples/explore_real_db.rs` 和 `examples/explore_root_structure.rs` 分析了真实的 osu!lazer 数据库：

**数据库结构 (v24)**:
```
Root Array (11 elements at 0x4DFD0):
  [0] → Schema ref (0x30F0)
        └─ 17 elements: Schema definitions
  [1] → Tables ref (0x2CDD0)
        └─ 17 elements: Object table roots
           [0] → Table 0: 14 elements (BeatmapSet?)
           [1] → Table 1: 14 elements
           [2] → Table 2: 14 elements
           ...
  [2] → File size (tagged: 1048576)
  [3-5] → Metadata arrays (70 elements each)
  [6-7] → Version info (tagged values)
  [8] → Additional ref
  [9-10] → Flags (tagged values)
```

**发现**:
- ✅ 成功解析了根数组和表引用
- ✅ 识别了 17 个对象表
- ✅ 找到了 15466 个字符串
- ✅ 检测到 5502 个数组节点
- ✅ 验证了所有主要 osu! 表都存在 (BeatmapInfo, Skin, Collection等)

## 需要实现的功能 🚧

### Phase 1: Schema 解析 (高优先级)

**目标**: 完整理解 Realm v24 的 Schema 编码

**任务**:
1. **研究 Schema 格式**
   - Schema 数组的第一个元素是什么？(Tagged value: 3511172401368543926)
   - 其余元素是表定义的引用吗？
   - 表定义的内存布局是什么？

2. **实现 Schema 解析器**
   ```rust
   // src/realm/format/schema_parser.rs
   pub struct SchemaParser<'a> {
       data: &'a [u8],
   }
   
   impl SchemaParser<'_> {
       pub fn parse_table_def(&self, offset: usize) -> Result<TableDef> {
           // 解析表名
           // 解析字段列表
           // 解析字段类型和属性
       }
   }
   ```

3. **集成到 Deserializer**
   ```rust
   impl Deserializer<'_> {
       pub fn parse_schema(&mut self) -> Result<&Schema> {
           let schema_array = self.get_schema_array()?;
           let parser = SchemaParser::new(self.data);
           
           for i in 0..schema_array.len() {
               if let Some(table_def) = parser.parse_table_def_at(i) {
                   self.schema.tables.insert(i as u8, table_def);
               }
           }
       }
   }
   ```

**验证**:
- 能够解析出所有 17 个表的名称
- 能够识别每个表的字段
- 字段类型映射正确 (int/string/link/list等)

### Phase 2: 对象反序列化 (高优先级)

**目标**: 根据 Schema 反序列化对象

**任务**:
1. **理解对象布局**
   - 对象在 B+Tree 叶子节点中如何存储？
   - 字段是连续存储还是有特殊编码？
   - 可空字段如何标记？

2. **实现对象读取器**
   ```rust
   // src/realm/format/object_reader.rs
   pub struct ObjectReader<'a> {
       data: &'a [u8],
       schema: &'a Schema,
   }
   
   impl ObjectReader<'_> {
       pub fn read_object(&self, 
                          table_key: u8, 
                          offset: usize) -> Result<RealmObject> {
           let table_def = self.schema.get_table(table_key)?;
           
           let mut fields = HashMap::new();
           let mut cursor = offset;
           
           for (i, field_def) in table_def.fields.iter().enumerate() {
               let value = self.read_field(&mut cursor, field_def)?;
               fields.insert(i, value);
           }
           
           Ok(RealmObject { table_key, fields })
       }
       
       fn read_field(&self, cursor: &mut usize, field_def: &FieldDef) -> Result<Value> {
           match field_def.field_type {
               FieldType::Int => self.read_int(cursor),
               FieldType::String => self.read_string_ref(cursor),
               FieldType::Object(target) => self.read_object_ref(cursor, target),
               // ...
           }
       }
   }
   ```

3. **实现 B+Tree 遍历**
   ```rust
   pub struct BTreeIterator<'a> {
       data: &'a [u8],
       root: ArrayView<'a>,
   }
   
   impl<'a> Iterator for BTreeIterator<'a> {
       type Item = Result<usize>; // 返回对象偏移
       
       fn next(&mut self) -> Option<Self::Item> {
           // 遍历 B+Tree 返回所有叶子节点
       }
   }
   ```

**验证**:
- 能够读取 Table 0 中的所有对象
- 字段值正确解析
- 字符串字段能够正确读取
- 对象引用 (link) 能够跟随

### Phase 3: 对象序列化 (中优先级)

**目标**: 将修改后的对象写回 Realm 格式

**任务**:
1. **实现对象写入器**
   ```rust
   // src/realm/format/object_writer.rs
   pub struct ObjectWriter {
       serializer: Serializer,
       schema: Schema,
   }
   
   impl ObjectWriter {
       pub fn write_object(&mut self, obj: &RealmObject) -> Result<usize> {
           let table_def = self.schema.get_table(obj.table_key)?;
           let offset = self.serializer.position();
           
           for field_def in &table_def.fields {
               if let Some(value) = obj.fields.get(&field_def.index) {
                   self.write_field(value, field_def)?;
               } else if field_def.nullable {
                   self.write_null()?;
               } else {
                   return Err(Error::missing_field(field_def.name));
               }
           }
           
           Ok(offset)
       }
   }
   ```

2. **实现引用管理**
   ```rust
   pub struct RefManager {
       // 跟踪所有写入的对象及其偏移
       objects: HashMap<ObjectId, usize>,
       
       // 待回填的引用
       pending_refs: Vec<(usize, ObjectId)>,
   }
   
   impl RefManager {
       pub fn write_ref(&mut self, object_id: ObjectId) -> usize {
           if let Some(&offset) = self.objects.get(&object_id) {
               offset
           } else {
               // 预留空间稍后回填
               let placeholder = self.reserve_ref();
               self.pending_refs.push((placeholder, object_id));
               placeholder
           }
       }
       
       pub fn resolve_refs(&mut self) -> Result<()> {
           for (offset, object_id) in &self.pending_refs {
               let target = self.objects.get(object_id)?;
               self.write_u64_at(*offset, *target as u64)?;
           }
           Ok(())
       }
   }
   ```

3. **实现 B+Tree 构建**
   ```rust
   pub struct BTreeBuilder {
       // 构建新的 B+Tree
       // 支持插入、删除操作
   }
   ```

**验证**:
- 序列化单个对象
- 往返测试 (deserialize → serialize → deserialize)
- 修改对象并写回
- lazer 能够打开修改后的数据库

### Phase 4: 高级 API (中优先级)

**目标**: 提供类型安全、易用的 API

**任务**:
1. **定义强类型模型**
   ```rust
   // src/models/beatmap.rs (已存在，需要适配)
   impl BeatmapSetInfo {
       pub fn from_realm_object(obj: RealmObject, deser: &Deserializer) -> Result<Self> {
           // 从通用 RealmObject 转换
       }
       
       pub fn to_realm_object(&self) -> RealmObject {
           // 转换为通用 RealmObject
       }
   }
   ```

2. **实现查询构建器**
   ```rust
   // src/realm/query.rs (已存在，需要重写)
   impl QueryBuilder {
       pub fn beatmap_sets(&self) -> Result<Vec<BeatmapSetInfo>> {
           let deser = Deserializer::new(&self.data, self.root_offset)?;
           let table_root = deser.get_table_root(TABLE_BEATMAP_SET)?;
           
           let mut results = Vec::new();
           for obj_offset in BTreeIterator::new(&table_root) {
               let obj = deser.read_object_at(obj_offset?, TABLE_BEATMAP_SET)?;
               results.push(BeatmapSetInfo::from_realm_object(obj, &deser)?);
           }
           
           Ok(results)
       }
   }
   ```

3. **实现修改构建器**
   ```rust
   // src/realm/mutation.rs (已存在，需要重写)
   impl MutationBuilder {
       pub fn delete_beatmap_set(&mut self, id: Uuid) -> Result<()> {
           // 1. 找到对象
           // 2. 删除所有关联的 RealmFile
           // 3. 从 B+Tree 中移除
           // 4. 更新引用计数
       }
       
       pub fn add_collection(&mut self, name: &str) -> Result<Uuid> {
           // 1. 创建新的 Collection 对象
           // 2. 生成 UUID
           // 3. 插入到 B+Tree
           // 4. 返回 UUID
       }
   }
   ```

**验证**:
- 能够列出所有铺面
- 能够创建收藏夹
- 能够删除铺面
- lazer 正确显示修改

### Phase 5: 完整集成 (低优先级)

**目标**: 与现有系统集成

**任务**:
1. **集成安全机制**
   - 所有写操作通过 `SafetyGuard`
   - 自动备份和恢复
   - 哈希文件一致性验证

2. **性能优化**
   - Schema 缓存
   - 延迟加载
   - 批量操作

3. **错误处理增强**
   - 详细的损坏检测
   - 恢复建议
   - 中英双语错误信息

## 技术挑战

### 1. Schema 编码格式
**问题**: Realm v24 的 Schema 二进制格式未公开文档

**策略**:
- 逆向分析真实数据库的 Schema 部分
- 对比 Realm Core 源代码 (C++)
- 通过已知表结构反推编码规则
- 使用 Realm Studio 查看 Schema 作为参考

### 2. B+Tree 结构
**问题**: Realm v24 使用 cluster-based B+Tree，与 v9 不同

**策略**:
- 分析 `is_inner_node` 标志的含义
- 理解内部节点和叶子节点的区别
- 实现递归遍历算法
- 参考 realm-core 的 `Array` 和 `BPlusTree` 实现

### 3. 字符串存储
**问题**: 短字符串可能内联，长字符串存储为数组

**策略**:
- 实现 15 字节阈值检测
- 内联字符串使用 SSO (Small String Optimization)
- 长字符串跟随引用读取数组

### 4. 对象引用 (Links)
**问题**: 跨表引用需要正确处理

**策略**:
- 引用存储为 (table_key, row_index) 对
- 实现引用跟随功能
- 处理悬空引用和循环引用

## 测试策略

### 单元测试
- 每个解析器组件的独立测试
- 使用手工构造的最小 Realm 数据

### 集成测试
- 使用真实的 lazer 数据库
- 只读测试（不修改用户数据）
- 在隔离环境中的写入测试

### 往返测试 (Round-trip)
```rust
#[test]
fn test_roundtrip() {
    let original = read_database("test.realm");
    let objects = deserialize_all(&original);
    let reconstructed = serialize_all(&objects);
    assert_eq!(original, reconstructed);
}
```

### 兼容性测试
- 序列化后的数据库能被 osu!lazer 打开
- 修改后的数据在 lazer 中正确显示
- 不破坏现有功能

## 当前状态总结

✅ **可以做的**:
- 打开真实 lazer 数据库
- 解析文件头和根结构
- 访问表的根节点
- 识别数组节点
- 序列化基础类型

❌ **不能做的**:
- 完整解析 Schema
- 反序列化对象（缺少 Schema）
- 修改并写回数据库
- 类型安全的高级 API

## 下一步行动

**立即可做**:
1. 深入研究 Schema 数组的编码
2. 手工解析一个表定义作为样本
3. 实现基础的对象读取（硬编码 Schema）

**需要研究**:
1. Realm Core 源代码中的 Schema 编码
2. B+Tree 的详细布局
3. 字符串和二进制数据的存储方式

**里程碑**:
- [ ] M1: 能够解析完整 Schema
- [ ] M2: 能够读取任意对象
- [ ] M3: 能够修改并写回对象
- [ ] M4: 通过 lazer 兼容性测试
- [ ] M5: 发布 v0.2.0

## 参考资源

- **Realm Core**: https://github.com/realm/realm-core
- **osu!lazer**: https://github.com/ppy/osu
- **现有解析工具**: 
  - realm-codec (仅支持 v9)
  - Realm Studio (官方工具)
  
## 工作量估算

- Phase 1 (Schema 解析): 2-3 天
- Phase 2 (对象反序列化): 3-5 天
- Phase 3 (对象序列化): 3-5 天
- Phase 4 (高级 API): 2-3 天
- Phase 5 (完整集成): 1-2 天

**总计**: 约 2-3 周全职工作

---

*最后更新: 2026-09-09*
*状态: 基础架构完成，开始 Phase 1*
