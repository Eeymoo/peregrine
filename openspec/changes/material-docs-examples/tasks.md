# 物料创作文档与示例 - 实施任务清单

> 来源：`four-layer-customization` tasks.md §19（19.2–19.6 假勾选修正后移入）。

## 1. 物料创作指南文档

- [ ] 1.1 新增 `docs/guide/material-scripting.md`，按五步创作流程组织：选图元 → 定布局 → 抽参数 → 声明 defaults/schema → 验证
- [ ] 1.2 文档覆盖三函数约定（`build` / `defaults` / `schema`）+ 参数 widget 类型清单（number / slider / color / select / toggle / image_path / text）
- [ ] 1.3 文档覆盖动态输入 API：`time_ms()` / `mouse_pos()` / `key_down(code)` / `rand()`，含 `is_dynamic()` 声明与缓存行为说明
- [ ] 1.4 文档覆盖沙箱限制（max_operations / 无文件 IO / 无网络）与常见错误排查
- [ ] 1.5 文档包含至少一个完整可运行的物料示例（从 defaults/schema 到 build 全文）

## 2. 图层使用文档

- [ ] 2.1 新增 `docs/guide/layers.md`：图层层概念、叠加顺序（数组序 = 渲染序）、变换（offset / scale / rotation）、样式（颜色 / 不透明度 / 混合）、可见性与锁定
- [ ] 2.2 说明图层与 Profile 的关系，以及旧配置自动迁移后的图层形态（单图层）

## 3. 文档站集成

- [ ] 3.1 在 `docs/.vitepress/config.mts` 注册 `material-scripting` 与 `layers` 侧边栏条目
- [ ] 3.2 如 zh-cn 文档结构同步维护，在 `docs/zh-cn/` 下添加对应中文版本并注册
- [ ] 3.3 `cd docs && npm run docs:build` 构建通过，无死链警告

## 4. 示例物料库

- [ ] 4.1 新增 `crates/material/examples/` 目录，编写静态示例（如简易十字变体，仅 build/defaults/schema 三函数）
- [ ] 4.2 编写时间动态示例（时钟，`is_dynamic() == true`，使用 `time_ms()`）
- [ ] 4.3 编写输入动态示例（鼠标跟随点或键盘响应提示，使用 `mouse_pos()` / `key_down()`）
- [ ] 4.4 为每个示例添加顶部中文注释（用途、参数说明、对应创作流程步骤）
- [ ] 4.5 在 `crates/material` 新增单元测试：`Material::load` 成功加载每个示例且 `evaluate` 返回非空 Element 列表

## 5. time.rhai 归位

- [ ] 5.1 检查默认配置与迁移逻辑是否引用 `builtin.time`
- [ ] 5.2 若无引用：将 `crates/material/builtin/time.rhai` 移至 `examples/`，同步 `BUILTIN_MATERIALS` 列表与相关测试；若有引用：保留为内置并在 `material-scripting.md` 中说明其作为动态物料范例的地位

## 6. README 更新

- [ ] 6.1 `README.md` 第 59 行旧样式清单更新为四层架构描述（元素 / 物料 / 图层 / 配置 + 用户可编程物料）
- [ ] 6.2 `README.zh-cn.md` 同步更新

## 7. 回归验证

- [ ] 7.1 `cargo test -p peregrine_material` 全部通过（含新增示例加载测试）
- [ ] 7.2 `cargo clippy -p peregrine_material -- -D warnings` 通过
- [ ] 7.3 文档站构建通过（同 3.3）
