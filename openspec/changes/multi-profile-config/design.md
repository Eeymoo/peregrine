# 多配置管理设计文档（方案 A：单图层编辑当前 active profile）

## 数据模型

`AppConfig` 保持现有结构：

```rust
pub struct AppConfig {
    pub active_profile: String,
    pub profiles: HashMap<String, Profile>,
    pub settings: AppSettings,
}
```

### 单图层兼容判定

```rust
impl Profile {
    /// 是否可在单图层 UI 中编辑。
    /// 条件：只有 1 个图层，且该图层是默认/空图层。
    pub fn is_legacy_compatible(&self) -> bool {
        if self.layers.len() != 1 {
            return false;
        }
        let layer = &self.layers[0];
        layer.is_legacy_compatible()
    }
}

impl Layer {
    /// 图层是否未经过深度自定义，可以在单图层 UI 中编辑。
    pub fn is_legacy_compatible(&self) -> bool {
        // 使用基础物料（cross / edge_rect / ring / corner_dots4 / large_cross 等）
        // 且 transform 为默认、style 为默认、params 与物料 defaults 一致。
        if self.transform != Transform2D::default() {
            return false;
        }
        if self.style.opacity != LayerStyle::default().opacity || self.style.blend_mode != BlendMode::Normal {
            return false;
        }
        let material_id = self.material.material_id();
        let legacy_materials = ["builtin.cross", "builtin.edge_rect", "builtin.ring", "builtin.corner_dots4", "builtin.large_cross"];
        if !legacy_materials.contains(&material_id.as_str()) {
            return false;
        }
        // params 必须等于物料 defaults
        true
    }
}
```

## 后端命令

新增 `ProfileCommands` 模块：

```rust
#[tauri::command]
fn list_profiles(state: State<AppState>) -> Vec<String> { ... }

#[tauri::command]
async fn create_profile(state: State<'_, AppState>, name: String) -> Result<Profile, String> { ... }

#[tauri::command]
async fn rename_profile(state: State<'_, AppState>, old_name: String, new_name: String) -> Result<(), String> { ... }

#[tauri::command]
async fn delete_profile(state: State<'_, AppState>, name: String) -> Result<(), String> { ... }

#[tauri::command]
async fn set_active_profile(state: State<'_, AppState>, name: String) -> Result<(), String> { ... }

#[tauri::command]
fn get_profile(state: State<AppState>, name: String) -> Result<Profile, String> { ... }

#[tauri::command]
fn is_profile_legacy_compatible(profile: Profile) -> bool { profile.is_legacy_compatible() }

#[tauri::command]
fn get_active_profile_name(state: State<AppState>) -> Result<String, String> { ... }
```

## 前端状态

```ts
interface AppState {
  config: AppConfig;
  activeProfile: Profile;
  layersMode: boolean; // true=多图层模式，false=单图层模式
}
```

### 单图层模式

- 编辑 `config.profiles[config.active_profile]`。
- 如果该 profile 不是 `is_legacy_compatible`，显示提示并强制切换到多图层模式。
- 样式变更只修改 `layers[0]`。
- "应用到多图层"不需要，因为单图层模式就是直接编辑当前 profile。

### 多图层模式

- 编辑 `config.profiles[config.active_profile]`。
- 完整图层管理。

## ProfileManager 组件

位置：单图层/多图层模式都显示在顶部栏。

功能：
- 下拉选择 active profile
- 新建 profile（输入名称，默认单图层兼容）
- 重命名 profile（不能重命名为已存在名称）
- 删除 profile（至少保留一个）
- 复制 profile（复制所有图层）

## 配置持久化

所有 profile 变更通过 `save_config` 保存整个 `AppConfig`，并广播给 overlay。

## 渲染

- `OverlayRenderer` 始终渲染当前 `active_profile` 对应的 `Profile`。

## 任务拆分

1. 数据层：`Profile::is_legacy_compatible()` 和 `Layer::is_legacy_compatible()`。
2. 后端命令：profile CRUD + active profile 切换。
3. 前端 API：新增 profile 相关 invoke 封装。
4. 前端组件：新增 `ProfileManager`。
5. 前端状态：单图层模式绑定 active profile，非兼容时切换到多图层模式。
6. 测试：`cargo test -p peregrine_config`，`npx tsc --noEmit`，`npm run build`。
