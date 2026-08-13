# 任务清单：Overlay 动态物料刷新修复与文本加粗

## 1. 动态物料持续重绘（overlay.rs）

- [x] 1.1 `OverlayApp` 新增字段：`dynamic_dirty: bool`（初始 `true`）+ 缓存的动态性判定结果
- [x] 1.2 实现 `compute_is_animated(&self) -> bool`：遍历 active profile 可见图层，经 `material_registry` 查询任一 `is_dynamic == true`
- [x] 1.3 改造 `about_to_wait` 新格式分支：`dynamic_dirty` 时调用 1.2 并缓存，之后复用缓存值；删除写死 `None` 的"首期简化"分支
- [x] 1.4 `OverlayCommand::UpdateConfig` / `create_overlay` 路径置位 `dynamic_dirty = true`
- [x] 1.5 新增 `OverlayCommand::RefreshMaterials` 变体：更新 registry 句柄 + `dynamic_dirty = true` + `needs_redraw = true`

## 2. 物料热重载联动（src-tauri/src/lib.rs）

- [x] 2.1 物料 watcher 重建 registry 并广播 `peregrine:materials-changed` 后，向 overlay 线程发送 `OverlayCommand::RefreshMaterials`

## 3. 文本字重（schema + 渲染 + 物料）

- [x] 3.1 `schema.rs`：`Element::Text` 新增 `font_weight: Option<u16>`（`#[serde(default)]` + 中文 doc comment）
- [x] 3.2 ~~`AppConfig::validate()` 扩展~~（**实现调整**：Element 为求值输出、不持久化，校验移至 `material.rs` Rhai→Element 转换层，非法字重返回 `MaterialError::ElementField`，该图层求值失败被跳过）
- [x] 3.3 单元测试：serde 往返（含缺失字段旧 JSON 兼容）、非法字重校验失败用例
- [x] 3.4 `svg_renderer.rs`：`<text>` 输出 `font-weight` 属性（`None` 时不输出）
- [x] 3.5 `material.rs` Rhai→Element 转换：`font_weight` 缺失 / null / 700 三种输入的单元测试
- [x] 3.6 `time.rhai`：`defaults()` 加 `bold: false`，`schema()` 加 toggle 参数「加粗」，`build()` 按 `bold` 输出 `font_weight: 700`

## 4. 前端同步

- [x] 4.1 `src/types/config.ts`：`Element` text 变体新增 `fontWeight?: number`
- [x] 4.2 `src/components/Preview.tsx`：Canvas `fillText` 按 `fontWeight >= 600` 设置 `bold` 字体串

## 5. 回归验证

- [x] 5.1 `cargo test -p peregrine_config -p peregrine_material -p peregrine` 全部通过
- [x] 5.2 `cargo clippy`（3 crate）+ `cargo fmt --check` 通过
- [x] 5.3 `npx tsc --noEmit` + `npm run build` 通过
- [x] 5.4 `cargo check --target x86_64-pc-windows-msvc` 通过【**CI 覆盖**：ci.yml 每次 push 跑 release 编译，release.yml 打 v* tag 时三架构（i686/x86_64/aarch64）完整构建；3 个 lib crate 的 windows-msvc check 已本机通过】

待验证（动态物料实机，**暂缓**——`MATERIAL_DYNAMIC_INPUT_ENABLED = false` 软关闭期间不可达，待重新启用后补测）：
- [ ] ~~5.5 时钟物料每秒自动更新，无窗口交互（checklist B4.1）~~【暂缓：动态物料软关闭】
- [ ] ~~5.6 纯静态 profile overlay 无空转（任务管理器 CPU 与修复前一致）~~【暂缓：动态物料软关闭】
- [ ] ~~5.7 切换静态 ↔ 动态 profile 后重绘调度即时切换~~【暂缓：动态物料软关闭】
- [ ] ~~5.8 时间物料「加粗」开关：overlay 与预览同步变粗，目测可读性改善~~【暂缓：动态物料软关闭】
- [ ] ~~5.9 物料热重载改 `is_dynamic` 后无需重启 overlay 即时生效~~【暂缓：动态物料软关闭】
- [ ] ~~5.10 鼠标跟随 / 键盘响应物料实效复测（依赖 `material-docs-examples` §4 示例物料就位，checklist B4.2/B4.3）~~【暂缓：动态物料软关闭】
