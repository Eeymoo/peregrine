- [x] 在 `Profile` 上实现 `is_legacy_compatible()`
- [x] 在 `Layer` 上实现 `is_legacy_compatible()`（默认图层、默认 transform、默认 opacity、基础物料）
- [x] 添加单元测试

- [x] 新增 `list_profiles` command
- [x] 新增 `create_profile` command（默认生成单图层兼容 profile）
- [x] 新增 `rename_profile` command
- [x] 新增 `delete_profile` command
- [x] 新增 `set_active_profile` command
- [x] 新增 `get_profile` command
- [x] 新增 `get_active_profile_name` command
- [x] 新增 `copy_profile` command
- [x] 在 `invoke_handler` 中注册新 commands

- [x] 在 `src/lib/api.ts` 中新增 profile 相关 invoke 封装

- [x] 新建 `src/components/ProfileManager.tsx` 组件（下拉选择 + 图标按钮：新建/重命名/复制/删除）
- [x] 在 `ConfigApp` 顶部栏集成 `ProfileManager`
- [x] 在 `LayersEditor` 顶部栏集成 `ProfileManager`
- [x] 单图层模式下，如果当前 profile 不兼容，显示提示并禁用编辑控件
- [x] 初始加载时根据 active profile 兼容性自动选择单图层/多图层模式
- [x] 使用过程中不强制切换模式，保持用户主动控制
- [x] 切换 active profile 后自动刷新配置和 profile 列表
- [x] 给 `StyleFields` 添加 `disabled` prop，支持整体禁用

- [x] 添加 `profile.*` i18n key（zh-CN / en）
- [x] 添加 `common.delete` / `common.save` i18n key

- [x] `cargo test -p peregrine_config`
- [x] `npx tsc --noEmit`
- [x] `npm run build`
- [ ] `cargo build --manifest-path src-tauri/Cargo.toml --bins --release`（Linux 环境缺少 pkg-config，无法本地验证）

待验证：
- [ ] Windows 上完整 Tauri 构建
- [ ] 单图层模式下创建/切换/复制 profile 的 UI 交互
- [ ] 多图层模式下管理 profile 并编辑图层
- [ ] 切换 profile 后 overlay 渲染正确
