# 多配置（Profile）管理

## 目标

将 Peregrine 升级为**多配置、多图层**模型，同时保留原有单图层 UI 作为简化编辑器：

- 顶层是**配置（Profile）**，每个配置对应一个使用场景（游戏、应用、全局）。
- 每个配置包含多个**图层（Layer）**，图层之间叠加渲染。
- 单图层 UI 是**当前 active profile 的简化编辑器**，只支持编辑单图层兼容的配置。
- 多图层 UI 是**当前 active profile 的完整编辑器**，支持完整图层管理。
- 支持多个配置的增、删、改、查，以及激活配置切换。
- 切换 active profile 后，若新配置不是单图层兼容的，前端自动切换到多图层模式。

## 当前问题

1. 单图层 UI 和多图层 UI 同时共享同一个 active profile，但单图层 UI 不应直接修改多图层配置。
2. 没有配置级别的 UI，用户无法创建、切换、重命名、删除多个 profile。

## 变更范围

### 数据层（`crates/config`）

- `AppConfig` 结构不变（已有 `active_profile` + `profiles`）。
- 在 `Profile` 上实现 `is_legacy_compatible()`：
  - 必须只有 `layers.len() == 1`
  - `layers[0]` 必须是单图层兼容的默认图层（使用 `builtin.cross`、`builtin.edge_rect` 等基础物料，且无额外参数）
- 提供 `AppConfig::default_legacy_profile()` 生成一个默认单图层 profile。
- 序列化兼容：不新增字段，无需迁移。

### 后端接口（`src-tauri`）

新增 Tauri command：

- `list_profiles() -> Result<Vec<String>>`
- `create_profile(name: String) -> Result<Profile>`
- `rename_profile(old_name: String, new_name: String) -> Result<()>`
- `delete_profile(name: String) -> Result<()>`
- `set_active_profile(name: String) -> Result<()>`
- `get_profile(name: String) -> Result<Profile>`
- `is_profile_legacy_compatible(profile: Profile) -> Result<bool>`
- `get_active_profile_name() -> Result<String>`
- 修改 `save_config`：保存后确保 `active_profile` 存在，并广播给 overlay。

### 前端（`src`）

- `ConfigApp` 在单图层模式下绑定当前 `active_profile`。
- 如果当前 `active_profile` 不是单图层兼容的，则禁用单图层 UI，显示提示并强制切换到多图层模式。
- 新增 `ProfileManager` 组件：
  - 下拉选择当前 active profile
  - 新建配置（输入名称，默认创建一个单图层兼容的默认配置）
  - 重命名配置
  - 删除配置（至少保留一个配置）
  - 复制配置（复制所有图层）
- 配置管理器在单图层模式和多图层模式都可见。
- 切换 active profile 后，如果新配置不是单图层兼容的，自动切换到多图层模式；如果是单图层兼容的，保持当前模式。

### 渲染层

- `OverlayRenderer` 始终渲染当前 `active_profile` 对应的 `Profile`。
- 单图层兼容配置和多图层配置都按图层列表渲染。

## 验收标准

- [ ] 单图层 UI 只编辑单图层兼容的 active profile；多图层配置无法在单图层 UI 编辑。
- [ ] 支持创建、重命名、删除、复制、切换多个配置。
- [ ] 切换 active profile 后，overlay 立即按新配置渲染。
- [ ] 旧配置文件升级后保持可用。
- [ ] `cargo test -p peregrine_config` 全部通过。
- [ ] 前端 TypeScript 检查和构建通过。

## 影响范围

- `crates/config/src/schema.rs`（`Profile::is_legacy_compatible`）
- `src-tauri/src/lib.rs`（新增 commands）
- `src/lib/api.ts`（新增 API 封装）
- `src/ConfigApp.tsx`（单图层模式绑定 active profile）
- `src/components/ProfileManager.tsx`（新增）
- `src/components/LayersEditor.tsx`（集成配置管理器）
- `src/i18n/locales/*.json`（新增 profile 相关翻译）
- `src/types/config.ts`（无需修改）
