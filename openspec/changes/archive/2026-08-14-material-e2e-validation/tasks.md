# 物料运行时端到端验证与质量基线 - 实施任务清单

> 来源：`four-layer-customization` tasks.md 7.10（假勾选修正）+ §21（21.1–21.7 整体移入）。本 change 是 `v0.2.1` stable 发布的准入门槛。

## 1. 迁移回归集成测试（原 7.10）

- [ ] 1.1 在 `crates/config` 的 `[dev-dependencies]` 新增 `peregrine_material`（path 依赖，仅测试用）
- [ ] 1.2 新建 `crates/config/tests/migration_regression.rs`：加载真实内置物料，对每种旧 style 构造典型参数组合，执行"旧 `build_shapes` 输出 vs 迁移后图层物料求值输出"逐元素、逐字段对比
- [ ] 1.3 覆盖 13 个用例：12 种 style + `toilet_paper` alias；`RandomOrb` 必须断言随机序列与旧 `SimpleRng` 完全一致
- [ ] 1.4 `CustomImage` 用例断言物料返回单个 `Element::Image` 且 path / scale / offset 字段保留
- [ ] 1.5 `cargo test -p peregrine_config` 全部通过（含新集成测试）

## 2. 真实配置迁移验证（原 21.1）

- [ ] 2.1 准备 5 份真实用户旧配置（覆盖 12 种样式，含自定义 PNG 路径、极端参数值）
- [ ] 2.2 在 Windows 实机逐份执行迁移，截图对比迁移前后视觉，确认零退化
- [ ] 2.3 确认每份迁移都生成 `.legacy.bak` 备份且可手动还原

## 3. 性能基线（原 21.2 / 21.5 / 9.8 复核）

- [ ] 3.1 静态物料缓存命中求值 < 1µs（编写计时测试或 benchmark）
- [ ] 3.2 单图层单次求值 < 100µs（静态物料、典型参数）
- [ ] 3.3 1080p / 3 图层 / 60fps 渲染单帧 < 8ms（release 构建，Windows 实机）
- [ ] 3.4 1080p / 5 图层 / 60fps 连续渲染 1 小时无明显掉帧（frame time 持续 < 16ms）

## 4. 资源基线（原 21.3 / 21.4）

- [ ] 4.1 内存基线：对比 v0.1.x，物料缓存 + Rhai engine 常驻内存增量 < 10MB（Windows 任务管理器 / ETW 实测）
- [ ] 4.2 体积基线：release 构建（opt-level=z + lto）二进制增量 < 500KB

## 5. 用户物料错误隔离（原 21.6）

- [ ] 5.1 语法错误物料：加载失败仅警告，引用它的图层渲染为空，其余图层正常
- [ ] 5.2 运行时异常物料（除零 / 类型错误）：求值失败跳过该图层，overlay 不崩溃
- [ ] 5.3 死循环物料：被 max_operations 终止，单次求值阻塞 < 50ms
- [ ] 5.4 调用未注册函数物料：返回"未定义函数"错误，不影响其他物料

## 6. 动态物料实效（原 21.7）

- [ ] 6.1 时钟物料每秒更新（肉眼确认 + 日志采样）
- [ ] 6.2 鼠标跟随物料延迟 < 50ms（高速移动鼠标观察）
- [ ] 6.3 键盘响应物料按键即时反馈（按下 / 松开均在 1 帧内反映）

## 7. 验收与归档

- [ ] 7.1 全部验收项通过，结果记录到本 change 的验证报告（可直接写在 tasks.md 备注或 PR 描述）
- [ ] 7.2 任一指标超标时另开修复 change，不在本 change 内改实现
- [ ] 7.3 全部通过后，`four-layer-customization` 与本 change 满足归档条件，可发 `v0.2.1` stable
