# Peregrine 手动 UI 验证清单

> 对应 OpenSpec 变更 `multi-profile-config` 任务 29-31 和 `four-layer-customization` 任务 21.1-21.7。
> 需要在 Windows 环境运行应用后逐项验证。

## 前置准备

```powershell
# 1. 完成自动化验证（必须全部通过）
.\scripts\test-windows.ps1 -SkipBuild

# 2. 启动开发模式应用
npx tauri dev
```

或使用 release 构建产物：
```powershell
.\scripts\test-windows.ps1
.\src-tauri\target\release\peregrine-tauri.exe
```

---

## A. multi-profile-config 验证

### A1. 单图层模式下创建/切换/复制 profile（任务 29）

- [ ] **A1.1** 打开配置窗口（默认单图层模式），顶部可见 Profile 下拉框，显示当前 active profile
- [ ] **A1.2** 点击 `+` 按钮，输入名称（如 `测试A`），回车 → 新 profile 创建成功并自动切换为 active
- [ ] **A1.3** 新 profile 的样式/颜色/参数与默认值一致，预览正常渲染
- [ ] **A1.4** 点击铅笔图标，重命名为 `测试B` → 下拉框显示更新
- [ ] **A1.5** 点击复制图标 → 生成副本（如 `测试B (副本)`）并自动切换
- [ ] **A1.6** 修改副本的颜色 → 切换回原 profile，颜色未受影响（相互独立）
- [ ] **A1.7** 点击删除图标 → 当前 profile 被删除，自动切换到剩余第一个 profile
- [ ] **A1.8** 仅剩 1 个 profile 时，删除按钮禁用
- [ ] **A1.9** 创建同名 profile → 显示错误提示，不崩溃

### A2. 多图层模式下管理 profile 并编辑图层（任务 30）

- [ ] **A2.1** 点击「切换到图层模式」→ 图层编辑器打开，Profile 下拉框仍可见
- [ ] **A2.2** 在多图层模式下新建 profile → 创建成功，图层列表显示默认单图层
- [ ] **A2.3** 添加第二个图层（如 ring）→ 图层列表显示 2 项，预览叠加渲染
- [ ] **A2.4** 拖拽调整图层顺序 → 预览中叠加顺序同步变化
- [ ] **A2.5** 切换 profile → 图层列表和预览切换为新 profile 的内容
- [ ] **A2.6** 在多图层 profile 上点击「切换到单图层」→ 显示不兼容提示，编辑控件禁用

### A3. 切换 profile 后 overlay 渲染正确（任务 31）

- [ ] **A3.1** 创建 profile A（红色 cross）和 profile B（绿色 ring）
- [ ] **A3.2** 激活 profile A，点击「开始覆盖」→ overlay 显示红色十字
- [ ] **A3.3** overlay 运行中切换到 profile B → overlay **立即**变为绿色圆环
- [ ] **A3.4** 停止 overlay，切换 profile，重新启动 → 渲染与当前 active profile 一致
- [ ] **A3.5** 删除当前 active profile（自动切换）→ overlay 渲染新 active profile

---

## B. four-layer-customization 验证

### B1. 旧配置迁移（任务 21.1）

准备：备份 `%APPDATA%\Peregrine\config.json`，放入旧版（v0.1.x）配置文件。

- [ ] **B1.1** 旧配置（cross 样式）→ 启动后自动迁移，视觉效果与旧版一致
- [ ] **B1.2** 旧配置（ring / corner_dots / border_frame / edge_arrows / grid 各一份）→ 迁移后视觉零退化
- [ ] **B1.3** 迁移后原文件备份为 `config.json.legacy.bak`
- [ ] **B1.4** 放入损坏的 JSON → 应用正常启动（回退默认配置），不崩溃

### B2. 性能基准（任务 21.2-21.5）

- [ ] **B2.1** 1080p / 5 图层 / overlay 运行 1 小时 → 无明显掉帧（任务管理器观察 CPU 稳定）
- [ ] **B2.2** 对比 v0.1.15 内存占用 → 增量 < 10MB
- [ ] **B2.3** release 二进制体积对比 v0.1.15 → 增量 < 500KB

### B3. 用户物料错误场景（任务 21.6）

在 `%APPDATA%\Peregrine\materials\` 放入以下物料，逐个验证应用不崩溃：

- [ ] **B3.1** 语法错误物料（如 `fn build( {`）→ 启动正常，日志有 warn，该物料不可用
- [ ] **B3.2** 运行时异常物料（如 `1 / 0`）→ overlay 不崩溃，跳过该图层
- [ ] **B3.3** 死循环物料（`loop {}`）→ 达到 max_operations 后终止，不卡死
- [ ] **B3.4** 调用未注册函数 → 报错但不崩溃

### B4. 动态物料效果（任务 21.7）

> ⚠️ **2026-08-03 起预期反转**：`material-static-rendering` 已停用动态输入（`MATERIAL_DYNAMIC_INPUT_ENABLED = false`）。
> 时钟物料不再每秒更新（冻结渲染），鼠标跟随/键盘响应不生效，物料选择器中不出现 `builtin.time`。
> 以下条目仅在未来翻回动态开关后适用，当前跳过。

- [ ] **B4.1** 添加 `builtin.time`（时钟物料）图层 → overlay 每秒更新
- [ ] **B4.2** 添加鼠标跟随物料（examples 目录）→ 移动鼠标，overlay 延迟 < 50ms
- [ ] **B4.3** 添加键盘响应物料 → 按键即时响应
- [ ] **B4.4** 前端预览动态物料 → 显示「动态物料 - 预览为快照」提示

---

## D. material-static-rendering 验证（2026-08-03 快照）

> 对应 OpenSpec 变更 `material-static-rendering` 任务 6.1-6.5 及本轮 bug 修复回归。
> 大行为变更：overlay 从「永远旧准星」变为「按 layers 多图层渲染」。

### D1. 静态多图层渲染恢复（任务 6.1-6.2）

- [ ] **D1.1** 切到多图层模式，添加两个图层（如 cross + ring）→ overlay 按图层**叠加渲染**，不再是旧版默认准星
- [ ] **D1.2** 修改某图层颜色/参数并保存 → overlay 与预览**即时更新**（WYSIWYG），无需重启应用
- [ ] **D1.3** 调整图层顺序/可见性 → overlay 渲染同步变化
- [ ] **D1.4** 单图层模式下改准星样式 → overlay 同步更新（旧路径不回归）

### D2. 12 种内置物料目检（任务 6.3）

逐个切换物料渲染，确认无退化（重点：文本图元、自定义图片）：

- [ ] **D2.1** cross / large_cross / edge_rect / ring / corner_dots / border_frame
- [ ] **D2.2** edge_arrows / grid / custom_orb / random_orb / custom_image
- [ ] **D2.3** random_orb 随机序列与旧版一致（同参数同点位）

### D3. 动态物料停用（任务 6.4）

- [ ] **D3.1** 打开「添加图层」物料选择器 → 列表中**没有** `builtin.time`（时钟）
- [ ] **D3.2** overlay 挂着时观察任务管理器 → CPU 无动态轮询空转（与纯静态一致）
- [ ] **D3.3**（可选）手工在 config.json 给某图层指定 `builtin.time` → 时钟按固定时间冻结渲染，不崩溃不刷新

### D4. 旧配置兼容（任务 6.5）

- [ ] **D4.1** 纯旧 crosshair 配置（无 layers）→ 渲染外观与上个版本一致
- [ ] **D4.2** 含 layers 的配置编辑保存后 → config.json 中 layers 数据完整，不降级

### D5. 本轮修复回归

- [ ] **D5.1** 颜色不丢（fix `fe0b2f4`）：单图层 UI 改样式 → 快捷键/快捷色换色 → 切换 profile 再切回 → **颜色保持**
- [ ] **D5.2** 模式记忆（feat `62cff53`）：多图层模式关应用 → 重开仍是多图层；切单图层关应用 → 重开单图层
- [ ] **D5.3** 入口保留：不兼容 profile 提示区有「切换到图层编辑器」按钮，可正常进入；返回单图层出口可用
- [ ] **D5.4** 兼容判定恢复：不兼容 profile 在单图层模式下编辑控件**禁用**（防改坏多图层配置）；切到该 profile 时自动进入多图层模式

---

## C. merge-dev-into-four-layer 验证（dev v0.1.13 功能回归）

合并 origin/dev 后需确认 dev 引入的功能在四层架构代码库上正常工作。

### C1. 单例模式

- [ ] **C1.1** 应用已运行时再次双击启动 → 聚焦已有配置窗口，不启动第二个实例
- [ ] **C1.2** 配置窗口已关闭（托盘运行）时再次启动 → 从托盘恢复/重建配置窗口

### C2. Markdown 更新日志

- [ ] **C2.1** 设置窗口「更新」标签页检查更新 → 发布说明以 Markdown 排版渲染（标题/列表/加粗）
- [ ] **C2.2** 中文系统首次启动 → 自动启用中国大陆镜像（localStorage 只初始化一次）

### C3. 镜像下载修复

- [ ] **C3.1** 启用中国大陆镜像后下载更新 → 安装包下载链接同样套用镜像前缀（日志可见代理 URL）
- [ ] **C3.2** 关闭镜像后下载更新 → 下载链接直连 github.com

### C4. 拆分后的设置/配置窗口

- [ ] **C4.1** 设置窗口五个标签页（常规/覆盖/快捷键/更新/关于）均正常渲染与保存
- [ ] **C4.2** 关于标签页「复制版本信息」→ 剪贴板内容使用当前语言（中英文 i18n）
- [ ] **C4.3** 覆盖标签页快捷颜色「重置」按钮 → 五个色块恢复默认色值并持久化
- [ ] **C4.4** 配置窗口单/多图层模式切换、Profile 管理、开发者面板（版本号 3 击解锁）均正常

---

## 验证结果记录

| 日期 | 系统 | 版本 | 验证人 | A1 | A2 | A3 | B1 | B2 | B3 | B4 | C1-C4 | D1-D5 | 备注 |
|------|------|------|--------|----|----|----|----|----|----|----|-------|-------|------|
|      |      |      |        |    |    |    |    |    |    |    |       |       |      |

全部通过后：
```bash
# 更新 OpenSpec 任务状态，然后归档变更
openspec archive multi-profile-config
openspec archive material-static-rendering
openspec archive four-layer-customization
openspec archive merge-dev-into-four-layer
```
