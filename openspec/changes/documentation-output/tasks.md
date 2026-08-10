# 通用文档优化产出 - 实施任务清单

> 来源：物料域任务源自 `four-layer-customization` tasks.md §19（假勾选修正）；遥测域任务源自 `add-glitchtip-telemetry` tasks.md §8 / §9.3 / §9.4（原「延后到代码稳定后统一编写」）。

## 1. 物料创作指南文档

- [x] 1.1 新增 `docs/guide/material-scripting.md`，按五步创作流程组织：选图元 → 定布局 → 抽参数 → 声明 defaults/schema → 验证
- [x] 1.2 文档覆盖三函数约定（`build` / `defaults` / `schema`）+ 参数 widget 类型清单（number / slider / color / select / toggle / image_path / text）
- [x] 1.3 文档覆盖动态输入 API：`time_ms()` / `mouse_pos()` / `key_down(code)` / `rand()`，含 `is_dynamic()` 声明与缓存行为说明
- [x] 1.4 文档覆盖沙箱限制（max_operations / 无文件 IO / 无网络）与常见错误排查
- [x] 1.5 文档包含至少一个完整可运行的物料示例（从 defaults/schema 到 build 全文）

## 2. 图层使用文档

- [x] 2.1 新增 `docs/guide/layers.md`：图层层概念、叠加顺序（数组序 = 渲染序）、变换（offset / scale / rotation）、样式（颜色 / 不透明度 / 混合）、可见性与锁定
- [x] 2.2 说明图层与 Profile 的关系，以及旧配置自动迁移后的图层形态（单图层）

## 3. 上报 Code 登记文档（遥测域）

- [x] 3.1 新增仓库根 `REPORT_CODES.md`，按号段登记所有已定义 Code（`PGR-0xxx` 启动 / `PGR-1xxx` panic / `PGR-2xxx` 后端 IO / `PGR-3xxx` 前端 / `PGR-4xxx` 遮盖层 / `PGR-5xxx` 操作域）
- [x] 3.2 每条 Code 标注语义、触发点、接线状态（已接线 / 预留 / 豁免）；与 `telemetry.rs::report_code` / `src/lib/telemetry.ts::REPORT_CODES` 常量逐条对齐核验

## 4. 用户隐私说明文档（遥测域）

- [x] 4.1 新增 `docs/guide/privacy.md`：数据匿名保证（不采集 IP / 用户名 / 机器名 / 路径用户名 / 设备 ID）、上报三类事件（启动 / 崩溃 / 错误）说明、开关位置与临时授权入口、`PEREGRINE_DISABLE_TELEMETRY` 编译期禁用选项
- [x] 4.2 新增 `docs/zh-cn/guide/privacy.md` 中文镜像

## 5. 开发文档遥测章节（遥测域）

- [x] 5.1 `docs/guide/development.md` 补充遥测开发章节：DSN 环境变量配置（`.env.development` / `.env.production` / `GLITCHTIP_DSN` / `GLITCHTIP_DSN_TEST`）、本地调试（无 DSN SDK 不初始化）、`PEREGRINE_DISABLE_TELEMETRY` 用法、`safe_try!` 使用约定（仅关键路径）、Code 登记治理流程
- [x] 5.2 `docs/zh-cn/guide/development.md` 同步补充

## 6. AGENTS.md 遥测章节（遥测域）

- [x] 6.1 `AGENTS.md` 新增「遥测模块」章节：模块定位（`src-tauri/src/telemetry.rs` + 前端 `src/lib/telemetry.ts`）、DSN 注入方式、隐私开关语义（`telemetry_enabled` 首次缺省 = 未授权）、`safe_try!` 约定、Code 治理约定、编译期禁用方式

## 7. 示例物料库

- [x] 7.1 新增 `crates/material/examples/` 目录，编写静态示例（如简易十字变体，仅 build/defaults/schema 三函数）
- [x] 7.2 编写时间动态示例（时钟，`is_dynamic() == true`，使用 `time_ms()`）
- [x] 7.3 编写输入动态示例（鼠标跟随点或键盘响应提示，使用 `mouse_pos()` / `key_down()`）
- [x] 7.4 为每个示例添加顶部中文注释（用途、参数说明、对应创作流程步骤）
- [x] 7.5 在 `crates/material` 新增单元测试：`Material::load` 成功加载每个示例且 `evaluate` 返回非空 Element 列表

## 8. time.rhai 归位

- [x] 8.1 检查默认配置与迁移逻辑是否引用 `builtin.time`
- [x] 8.2 若无引用：将 `crates/material/builtin/time.rhai` 移至 `examples/`，同步 `BUILTIN_MATERIALS` 列表与相关测试；若有引用：保留为内置并在 `material-scripting.md` 中说明其作为动态物料范例的地位

## 9. README 更新

- [x] 9.1 `README.md` 旧样式清单更新为四层架构描述（元素 / 物料 / 图层 / 配置 + 用户可编程物料），补一行隐私说明链接
- [x] 9.2 `README.zh-cn.md` 同步更新

## 10. 文档站集成

- [ ] 10.1 在 `docs/.vitepress/config.mts` 注册 `material-scripting`、`layers`、`privacy` 侧边栏条目（中英镜像）
- [ ] 10.2 `cd docs && npm run docs:build` 构建通过，无死链警告

## 11. 回归验证

- [ ] 11.1 `cargo test -p peregrine_material` 全部通过（含新增示例加载测试）
- [ ] 11.2 `cargo clippy -p peregrine_material -- -D warnings` 通过
- [ ] 11.3 文档站构建通过（同 10.2）
