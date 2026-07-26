## ADDED Requirements

### Requirement: 迁移视觉等价性回归测试

系统 SHALL 提供集成测试 `crates/config/tests/migration_regression.rs`：对每种旧 `CrosshairStyle` 构造典型参数组合，执行迁移后对生成的图层调用真实物料求值，输出 MUST 与旧 `build_shapes` 在参数等价映射下逐元素、逐字段相等。覆盖 MUST 包含 12 种 style、`toilet_paper` alias，以及 `RandomOrb` 的随机序列一致性。

#### Scenario: 全部 13 个迁移回归用例通过

- **WHEN** 运行 `cargo test -p peregrine_config`
- **THEN** 迁移回归集成测试 MUST 包含 13 个用例（12 style + alias）
- **AND** 每个用例的迁移后物料求值输出 MUST 与旧 `build_shapes` 输出逐元素相等
- **AND** `RandomOrb` 用例 MUST 断言随机序列与旧 `SimpleRng` 完全一致

### Requirement: 渲染与求值性能基线

stable 发布前 MUST 在 Windows 实机（release 构建）验证以下性能基线：静态物料缓存命中求值 < 1µs；单图层单次求值 < 100µs；1080p / 3 图层 / 60fps 单帧渲染 < 8ms；1080p / 5 图层 / 60fps 连续 1 小时 frame time 持续 < 16ms。

#### Scenario: 长时间渲染无掉帧

- **WHEN** 在 1080p 屏幕以 5 个图层、60fps 连续渲染 1 小时
- **THEN** frame time MUST 持续低于 16ms
- **AND** 无内存随时间持续增长（无泄漏）

#### Scenario: 静态物料缓存命中接近零开销

- **WHEN** 同一图层在参数与屏幕尺寸不变的情况下连续求值
- **THEN** 缓存命中时单次求值耗时 MUST < 1µs

### Requirement: 资源占用基线

对比 v0.1.x stable，物料缓存与 Rhai engine 带来的常驻内存增量 MUST < 10MB；release 构建（opt-level=z + lto）二进制体积增量 MUST < 500KB。

#### Scenario: 二进制体积增量在预算内

- **WHEN** 对比 v0.1.x 与当前版本的 release 二进制
- **THEN** 体积增量 MUST < 500KB

### Requirement: 用户物料错误隔离端到端验证

四类错误场景 MUST 逐一在实机验证且不崩溃：语法错误、运行时异常、死循环、调用未注册函数。任何场景下其余物料与图层 MUST 正常工作。

#### Scenario: 死循环物料不阻塞渲染

- **WHEN** 用户物料包含 `loop {}` 且被图层引用
- **THEN** 求值 SHALL 被 `max_operations` 终止且单次阻塞 < 50ms
- **AND** overlay 持续渲染其余图层

### Requirement: 动态物料实效验证

动态物料 MUST 在实机验证达到以下体验标准：时钟物料每秒更新；鼠标跟随物料延迟 < 50ms；键盘响应物料按键状态在 1 帧内反映。

#### Scenario: 鼠标跟随延迟可接受

- **WHEN** 高速移动鼠标观察鼠标跟随物料
- **THEN** 视觉延迟 MUST < 50ms

### Requirement: 真实配置迁移零退化验收

stable 发布前 MUST 使用至少 5 份覆盖 12 种样式的真实旧配置在 Windows 实机执行迁移，逐份确认视觉零退化，且每份迁移 MUST 生成可手动还原的 `.legacy.bak` 备份。

#### Scenario: 真实旧配置迁移后视觉一致

- **WHEN** 用 5 份真实旧配置分别启动新版本
- **THEN** 每份配置迁移后的渲染效果 MUST 与旧版肉眼一致
- **AND** 配置目录 MUST 存在对应的 `.legacy.bak` 文件
