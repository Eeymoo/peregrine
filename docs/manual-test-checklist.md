# Peregrine 手动 UI 验证清单

> 覆盖以下 OpenSpec 变更的全部"代码已完成、待 Windows 实机验证"项：
> - `multi-profile-config`（A 区，任务 29-31）
> - `four-layer-customization`（B 区，任务 21.1-21.7）
> - `material-static-rendering`（D 区，任务 6.1-6.5 + 修复回归）
> - `merge-dev-into-four-layer`（C 区，dev v0.1.13 功能回归）
> - `settings-dev-mode`（E 区，任务 7.3-7.5）
> - `add-glitchtip-telemetry`（F 区，任务 10.1-10.16）
> - `unify-widget-fields`（G 区，全局 UI 重构复核）
> - `overlay-dynamic-text-fixes`（H 区，动态停用语义下复测）
>
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

> 2026-08-04 实测：除 A1.6 外全部通过 ✅

- [ ] **A1.1** 打开配置窗口（默认单图层模式），顶部可见 Profile 下拉框，显示当前 active profile
- [ ] **A1.2** 点击 `+` 按钮，输入名称（如 `测试A`），回车 → 新 profile 创建成功并自动切换为 active
- [ ] **A1.3** 新 profile 的样式/颜色/参数与默认值一致，预览正常渲染
- [ ] **A1.4** 点击铅笔图标，重命名为 `测试B` → 下拉框显示更新
- [ ] **A1.5** 点击复制图标 → 生成副本（如 `测试B (副本)`）并自动切换
- [ ] **A1.6** 修改副本的颜色 → 切换回原 profile，颜色未受影响（相互独立）
  - ⚠️ 实测问题：快捷颜色等操作**不立即生效**，下次操作才上屏（偶现，单图层）→ **已修复**（overlay UpdateConfig 主动 request_redraw，事件驱动下不再等后续事件），待回归
- [ ] **A1.7** 点击删除图标 → 当前 profile 被删除，自动切换到剩余第一个 profile
- [ ] **A1.8** 仅剩 1 个 profile 时，删除按钮禁用
- [ ] **A1.9** 创建同名 profile → 显示错误提示，不崩溃

### A2. 多图层模式下管理 profile 并编辑图层（任务 30）

> 2026-08-04 实测：除 A2.2 / A2.4 外符合预期 ✅；另发现图层删除后加入新图层偶现 `layer '...' not found`（未捕获 Promise）→ **已修复**（图层名输入改 400ms 防抖 + 在途请求错误捕获），待回归

- [ ] **A2.1** 点击「切换到图层模式」→ 图层编辑器打开，Profile 下拉框仍可见
- [x] **A2.2** 在多图层模式下新建 profile → 创建成功，图层列表显示默认单图层（实测：创建后默认为当前图层，合理 ✅）
- [ ] **A2.3** 添加第二个图层（如 ring）→ 图层列表显示 2 项，预览叠加渲染
- [ ] **A2.4** 拖拽调整图层顺序 → 预览中叠加顺序同步变化
  - ⚠️ 实测：暂不支持拖拽（暂定）；上下按钮**方向与实际操作相反**（列表反序显示但按原数组方向移动）→ **已修复**（上移=数组索引 +1），待回归
- [ ] **A2.5** 切换 profile → 图层列表和预览切换为新 profile 的内容
- [ ] **A2.6** 在多图层 profile 上点击「切换到单图层」→ 显示不兼容提示，编辑控件禁用
  - ⚠️ 实测：禁用态透明度与圆环半径等滑杆**视觉样式不一致** → **已修复**（统一为 pointer-events-none + opacity-60 wrapper），待回归

### A3. 切换 profile 后 overlay 渲染正确（任务 31）

> 2026-08-04 实测：全部正确 ✅

- [ ] **A3.1** 创建 profile A（红色 cross）和 profile B（绿色 ring）
- [ ] **A3.2** 激活 profile A，点击「开始覆盖」→ overlay 显示红色十字
- [ ] **A3.3** overlay 运行中切换到 profile B → overlay **立即**变为绿色圆环
- [ ] **A3.4** 停止 overlay，切换 profile，重新启动 → 渲染与当前 active profile 一致
- [ ] **A3.5** 删除当前 active profile（自动切换）→ overlay 渲染新 active profile

---

## B. four-layer-customization 验证

### B1. 旧配置迁移（任务 21.1）

> 2026-08-04 决定：迁移暂不人工测，后续用脚本验证 + 备份用户原有配置即可

准备：备份 `%APPDATA%\Peregrine\config.json`，放入旧版（v0.1.x）配置文件。

- [ ] **B1.1** 旧配置（cross 样式）→ 启动后自动迁移，视觉效果与旧版一致
- [ ] **B1.2** 旧配置（ring / corner_dots / border_frame / edge_arrows / grid 各一份）→ 迁移后视觉零退化
- [ ] **B1.3** 迁移后原文件备份为 `config.json.legacy.bak`
- [ ] **B1.4** 放入损坏的 JSON → 应用正常启动（回退默认配置），不崩溃

### B2. 性能基准（任务 21.2-21.5）

> 2026-08-04 实测：暂不测，目测通过，待后续优化阶段补测

- [ ] **B2.1** 1080p / 5 图层 / overlay 运行 1 小时 → 无明显掉帧（任务管理器观察 CPU 稳定）
- [ ] **B2.2** 对比 v0.1.15 内存占用 → 增量 < 10MB
- [ ] **B2.3** release 二进制体积对比 v0.1.15 → 增量 < 500KB

### B3. 用户物料错误场景（任务 21.6）

> 2026-08-04 决定：用户物料不测，当前只支持官方内置物料

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
  - ℹ️ 实测偶现弹出「是否自动隐藏配置窗口并切换到游戏？」→ 定位为**设计行为**（启动 overlay 且 auto_switch=ask 时的确认对话框，只在点开始覆盖时出现），非 bug，继续观察复现条件

### D2. 12 种内置物料目检（任务 6.3）

> 2026-08-04 实测：除图片外没问题 ✅；文字不显示符合预期；custom_image 渲染不可用 → **已从选择器暂时隐藏**，待后续修复恢复

逐个切换物料渲染，确认无退化（重点：文本图元、自定义图片）：

- [ ] **D2.1** cross / large_cross / edge_rect / ring / corner_dots / border_frame
- [ ] **D2.2** edge_arrows / grid / custom_orb / random_orb / custom_image
- [ ] **D2.3** random_orb 随机序列与旧版一致（同参数同点位）

### D3. 动态物料停用（任务 6.4）

> 2026-08-04 实测：正确 ✅

- [ ] **D3.1** 打开「添加图层」物料选择器 → 列表中**没有** `builtin.time`（时钟）
- [ ] **D3.2** overlay 挂着时观察任务管理器 → CPU 无动态轮询空转（与纯静态一致）
- [ ] **D3.3**（可选）手工在 config.json 给某图层指定 `builtin.time` → 时钟按固定时间冻结渲染，不崩溃不刷新

### D4. 旧配置兼容（任务 6.5）

> 2026-08-04 实测：暂不测

- [ ] **D4.1** 纯旧 crosshair 配置（无 layers）→ 渲染外观与上个版本一致
- [ ] **D4.2** 含 layers 的配置编辑保存后 → config.json 中 layers 数据完整，不降级

### D5. 本轮修复回归

> 2026-08-04 实测：没问题 ✅

- [ ] **D5.1** 颜色不丢（fix `fe0b2f4`）：单图层 UI 改样式 → 快捷键/快捷色换色 → 切换 profile 再切回 → **颜色保持**
- [ ] **D5.2** 模式记忆（feat `62cff53`）：多图层模式关应用 → 重开仍是多图层；切单图层关应用 → 重开单图层
- [ ] **D5.3** 入口保留：不兼容 profile 提示区有「切换到图层编辑器」按钮，可正常进入；返回单图层出口可用
- [ ] **D5.4** 兼容判定恢复：不兼容 profile 在单图层模式下编辑控件**禁用**（防改坏多图层配置）；切到该 profile 时自动进入多图层模式

---

## C. merge-dev-into-four-layer 验证（dev v0.1.13 功能回归）

合并 origin/dev 后需确认 dev 引入的功能在四层架构代码库上正常工作。

### C1. 单例模式

> 2026-08-04 实测：没问题 ✅

- [ ] **C1.1** 应用已运行时再次双击启动 → 聚焦已有配置窗口，不启动第二个实例
- [ ] **C1.2** 配置窗口已关闭（托盘运行）时再次启动 → 从托盘恢复/重建配置窗口

### C2. Markdown 更新日志

> 2026-08-04 实测：没问题 ✅

- [ ] **C2.1** 设置窗口「更新」标签页检查更新 → 发布说明以 Markdown 排版渲染（标题/列表/加粗）
- [ ] **C2.2** 中文系统首次启动 → 自动启用中国大陆镜像（localStorage 只初始化一次）

### C3. 镜像下载修复

> 2026-08-04 实测：没问题 ✅

- [ ] **C3.1** 启用中国大陆镜像后下载更新 → 安装包下载链接同样套用镜像前缀（日志可见代理 URL）
- [ ] **C3.2** 关闭镜像后下载更新 → 下载链接直连 github.com

### C4. 拆分后的设置/配置窗口

> 2026-08-04 实测：除 C4.4 外没问题 ✅

- [ ] **C4.1** 设置窗口五个标签页（常规/覆盖/快捷键/更新/关于）均正常渲染与保存
- [ ] **C4.2** 关于标签页「复制版本信息」→ 剪贴板内容使用当前语言（中英文 i18n）
- [ ] **C4.3** 覆盖标签页快捷颜色「重置」按钮 → 五个色块恢复默认色值并持久化
- [ ] **C4.4** 配置窗口单/多图层模式切换、Profile 管理、开发者面板（版本号 5 击解锁）均正常
  - ⚠️ 实测问题：解锁流程不符合预期 → **已调整**：3 击改 5 击解锁、点击中实时显示剩余次数、解锁成功显示「已开启开发者模式」3 秒，待回归

---

## E. settings-dev-mode 验证

> 对应 OpenSpec 变更 `settings-dev-mode` 任务 §7.3 / 7.4 / 7.5。
> 重点：**release 构建的 DevTools 门禁**——门没关好等于给所有用户开了 devtools。

### E1. 开发构建（`npx tauri dev`）— 期望恒开放

- [ ] **E1.1** 启动后打开设置窗口 → Tab 栏直接有 6 个 Tab，第 6 个是「开发」（不需要解锁）
- [ ] **E1.2** 进入「开发」Tab → 看到「开启 DevTools」和「测试上报」两个按钮，**没有**日志/JSON/重置等其他控件
- [ ] **E1.3** 点「开启 DevTools」→ WebView DevTools 弹出
- [ ] **E1.4** 网页区域右键 → 有「检查」；Ctrl+Shift+I 也能打开 DevTools

### E2. release 构建（最关键，**门禁测试**）

构建：`npx tauri build` → 跑 `src-tauri/target/release/peregrine-tauri.exe`

- [ ] **E2.1** 首次启动后打开设置窗口 → **只有 5 个 Tab**，没有「开发」
- [ ] **E2.2** 网页区域右键 → **没有「检查」**；按 Ctrl+Shift+I → **不响应**
- [ ] **E2.3** 进入「关于」Tab，连点版本号 1、2 次 → 无任何提示（< 3 次不应有提示）
- [ ] **E2.4** 连点到第 3、4 次 → 出现「再点 N 次解锁」提示
- [ ] **E2.5** 中间停顿 > 1.5 秒 → 计数清零（下次再点从 0 开始，且无提示）
- [ ] **E2.6** 一气呵成点满 5 次 → 解锁成功提示（含「DevTools 需重新打开窗口后可用」），「开发」Tab 出现
- [ ] **E2.7** 检查 `%APPDATA%\Peregrine\config.json` → `settings.developer_mode = true`
- [ ] **E2.8** 关闭设置窗口再重新打开 → 右键「检查」可用、Ctrl+Shift+I 可用、「开启 DevTools」按钮可用
- [ ] **E2.9** 完全退出应用（托盘 Exit）后重新启动 → 设置窗口仍显示「开发」Tab（持久化）
- [ ] **E2.10** 把 config.json 中 `developer_mode` 改回 `false` 后重启 → 退回锁定态，Tab 消失，右键无「检查」

### E3. 配置窗口版本号（防回归）

> `settings-dev-mode` 已移除配置窗口的 DeveloperPanel；本节确认无残留。

- [ ] **E3.1** 打开配置窗口（不是设置窗口）→ 底部版本号是**纯文本**，点击无反应
- [ ] **E3.2** 配置窗口**没有**「开发」Tab、**没有** DeveloperPanel 残留

---

## F. add-glitchtip-telemetry 验证

> 对应 OpenSpec 变更 `add-glitchtip-telemetry` 任务 §10.1–10.16。
> **最复杂、最易出问题**：涉及隐私、网络、install_id 持久化、首次授权弹窗不能弹第二次。

### 前置准备

- GlitchTip 后台账号（**两个项目**：TEST 与正式），能查看 Events / Issues
- 抓包工具（Fiddler / Wireshark / `netstat` / `tasklist /v`），用于"零网络请求"验证
- 准备好让程序 panic 的方式（例如临时在代码里加 `panic!("test")` 跑一次 release，**别提交**）

### F1. 上报正确分流（10.1 / 10.7 / 10.14 / 10.15）

| 构建 | DSN 环境变量 | 期望 GlitchTip 项目 | event_type tag |
|---|---|---|---|
| `npx tauri dev` | `VITE_GLITCHTIP_DSN_TEST` / `GLITCHTIP_DSN_TEST` | TEST | startup=Info |
| `npx tauri build` | `GLITCHTIP_DSN` | 正式 | startup=Info |

- [ ] **F1.1** dev 启动一次 → TEST 项目收到一条 `app_startup` 事件，level=**Info**，**不进 issue 列表**（Info 不算 issue）
- [ ] **F1.2** 该事件的 tags 包含：`code=PGR-0001`、`event_type=startup`、`priority=p3`、`install_id`、`version`、`os`、`arch` — **逐个核对**
- [ ] **F1.3** release 启动一次 → 正式项目同样收到，tags 完整
- [ ] **F1.4** 在 GlitchTip 后台按 `event_type=crash` / `error` / `startup` 三个 tag 筛选 → 各自只能筛出对应类别

### F2. 开关关闭 / 无 DSN → 零网络（10.2 / 10.13）

- [ ] **F2.1** 设置页关掉遥测开关 → 重启 → 抓包确认**整个启动过程零出站网络请求**到 GlitchTip 域名
- [ ] **F2.2** 删除本地 `.env*` / 不注入 DSN 跑一次 → SDK 不初始化、零网络、应用功能完全正常
- [ ] **F2.3** 用 `PEREGRINE_DISABLE_TELEMETRY=1` 编译一次（`PEREGRINE_DISABLE_TELEMETRY=1 npx tauri build`）→ 设置页**遥测 UI 不可见或隐藏**、零网络（这是编译期禁用，最严格）

### F3. panic 落盘 + 静默回传（10.3 / 10.4）

> 这一组是最容易出 bug 的，重点测。

- [ ] **F3.1** 开关开启状态下，临时加 `panic!("test")` 跑 release → 程序崩溃
- [ ] **F3.2** 检查 `%APPDATA%\Peregrine\pending_reports\`（或类似路径）→ 有 JSON 文件，含 ts/version/install_id/code/message
- [ ] **F3.3** 删除 GlitchTip 后台对应 issue，再次启动应用 → **无任何弹窗**地静默上传了该 pending 记录，后台又收到
- [ ] **F3.4** 检查 pending 目录 → 已上传记录被清除
- [ ] **F3.5** 开关关闭状态下崩溃 → pending 落盘但**不上传**；累积多次崩溃 → pending 目录有多条
- [ ] **F3.6** pending 累积到超过 5MB → 最旧的被删除（验证容量上限）
- [ ] **F3.7** 报错页面（ErrorBoundary）点「匿名上传错误报告」按钮 → 一次性上传当前 + 全部历史；上传完成后**SDK 关闭**，后续不再上报（再崩溃又只能落盘）

### F4. 首次授权唯一性（10.5）— 隐私敏感

- [ ] **F4.1** 删掉 config.json 中 `telemetry_enabled` 字段（模拟首次启动）→ 启动时弹出**唯一一次**授权弹窗（"是否允许匿名上报崩溃信息与使用情况？"），默认勾选
- [ ] **F4.2** 选「允许」→ 字段写入 `true`，再启动**不再弹**
- [ ] **F4.3** 把字段改成 `false` 再启动 → **不再弹任何授权**（即使关了又开）
- [ ] **F4.4** 把字段重新删掉 → ⚠️ 行为待确认：是否再弹？（任务 4.1 说"字段缺失=未授权"，请与产品意图对齐）

### F5. install_id 稳定性（10.8）

- [ ] **F5.1** 启动一次 → 查看 install_id 文件（路径通常为 `%APPDATA%\Peregrine\install_id` 或独立文件），记下 UUID
- [ ] **F5.2** 多次重启 → UUID 不变
- [ ] **F5.3** 删除整个 config.json 重置 → install_id **不受影响**（独立文件）
- [ ] **F5.4** 手动把 install_id 文件内容改坏 → 启动时**自动重建**一个新 UUID，不崩溃
- [ ] **F5.5** 卸载重装（删除整个 Peregrine 目录再装） → UUID 变化（不同安装）

### F6. 脱敏（10.9）— 隐私敏感

- [ ] **F6.1** 用户名为 `C:\Users\张三\...` 的 Windows 账户下触发一次崩溃 → 在 GlitchTip 后台**抽样查看事件原文 JSON**
- [ ] **F6.2** 确认事件中：**无 IP**、**无 user 字段**、**无 server_name**、**无 machine_name**、所有路径中 `张三` 被替换为 `{user}`
- [ ] **F6.3** 前端 ErrorBoundary 上报的事件同样脱敏（前后端规则一致）

### F7. DSN 不入库（10.10）

```bash
git log -p --all | grep -E "GLITCHTIP_DSN|VITE_GLITCHTIP"
```

- [ ] **F7.1** 上述命令在仓库历史中**只出现 env var 名（如 `option_env!("GLITCHTIP_DSN")`），无真实 DSN 字符串**

### F8. 前端错误上报（10.6）

- [ ] **F8.1** 在 dev 模式下临时让某 React 组件抛错 → ErrorBoundary 接住并显示降级 UI
- [ ] **F8.2** GlitchTip TEST 项目收到事件，tag 含**组件名**和对应 `PGR-3xxx` code

### F9. 测试上报按钮（10.11）— 与 settings-dev-mode 联动

- [ ] **F9.1** 未解锁的 release 构建 → 设置窗口**没有**「开发」Tab，普通用户**看不到**测试上报按钮
- [ ] **F9.2** 解锁后（或 dev 构建）→ 「开发」Tab 里点「测试上报」→ GlitchTip 收到一条 **Error 级**事件，**进 issue 列表**

### F10. 开关修改重启弹窗（10.12）

- [ ] **F10.1** 在设置页切换遥测开关 → 弹「修改将在重启后生效」对话框
- [ ] **F10.2** 选「立即重启」→ 应用重启后新状态生效
- [ ] **F10.3** 选「稍后重启」→ 设置页保留「待重启生效」标记；手动重启后生效

---

## G. unify-widget-fields 复核（在拆分窗口布局下）

> 对应 OpenSpec 变更 `unify-widget-fields`。
> 任务已全打勾，但代码改动是**全局 UI 重构**，必须在当前**两窗口（配置 + 设置）+ 12 种样式**下复核一遍。
> 重点：双向同步、ColorField 非法值兜底、单位后缀显示。

### G1. 单图层模式（配置窗口）— 12 种样式逐个

切到每种 CrosshairStyle，重点验证 widget 渲染正确、双向同步：

- [ ] **G1.1** `cross`：size/thickness/gap 用 SliderField，**拖滑块→数字框同步**、**改数字框→滑块同步**
- [ ] **G1.2** `large_cross`：同上
- [ ] **G1.3** `edge_rect`：edge position 是 SelectField（两行布局）
- [ ] **G1.4** `ring`：anchor 用 SelectField
- [ ] **G1.5** `corner_dots_4/6/8`：count 用 NumberField（**无滑块**）
- [ ] **G1.6** `custom_orb`：位掩码组合控件保持独立（OrbPositionCheck）
- [ ] **G1.7** `random_orb`：seed/positions 等
- [ ] **G1.8** `border_frame`：thickness/margin
- [ ] **G1.9** `custom_image`：path 是 ImagePathField（输入框 + 「浏览」按钮），点「浏览」→ 弹文件选择对话框
- [ ] **G1.10** `edge_arrows`：position + size
- [ ] **G1.11** `grid`：rows/cols/spacing，alignment 是 SelectField

### G2. ColorField 重点（任务 1.4 + 5.7）

- [ ] **G2.1** hex 输入框输入合法值 `#ff0000` → 色块同步变红
- [ ] **G2.2** hex 输入框输入**非法值** `#zzz` / `xyz` → **保持当前值不变**（不能崩、不能清空）
- [ ] **G2.3** 点色块打开系统取色器 → 选色后同步到 hex 框
- [ ] **G2.4** 快捷色块（如多图层侧 LayerStyleEditor）→ 点击立即应用
- [ ] **G2.5** RGBA 透明度往返（hex ↔ tuple）无损

### G3. 多图层模式 LayerEditors（任务 5.6）

> ⚠️ 注意：当前物料运行时**静态渲染启用**，多图层模式可达（`material-static-rendering` §D5.2 已验证模式记忆），但**动态物料已停用**。

- [ ] **G3.1** opacity 显示为 `0.85` + `%` 后缀（原始值 0..1，不是 85）
- [ ] **G3.2** scale 显示为 `1.20` + `x` 后缀
- [ ] **G3.3** rotation 显示为 `45` + `°` 后缀
- [ ] **G3.4** 拖 opacity 到 0 → 图层完全透明

---

## H. overlay-dynamic-text-fixes 在动态停用语义下的复测

> 对应 OpenSpec 变更 `overlay-dynamic-text-fixes`。
> ⚠️ 任务原文 5.5–5.10 描述的是"动态物料应工作"，但**当前动态输入已软禁用**（`MATERIAL_DYNAMIC_INPUT_ENABLED = false`）。
> 所以这些条目**语义反转**，下表为对照：

| 原任务 | 当前期望 |
|---|---|
| 5.5 时钟每秒更新 | ❌ **不应**更新（冻结在固定时间），不崩溃 |
| 5.6 纯静态无空转 | ✅ CPU 与修复前一致（任务管理器观察） |
| 5.7 静态↔动态切换重绘 | ⚠️ 动态物料在选择器里被过滤，**无法构造动态 profile**，跳过 |
| 5.8 时间物料「加粗」 | ⚠️ time.rhai 已从选择器隐藏，**只有手工改 config.json 才能验证** |
| 5.9 物料热重载改 is_dynamic | ⚠️ 同上，需要手工写物料文件 |
| 5.10 鼠标/键盘物料 | ❌ 已停用，跳过 |

实际需要测的简化为：

- [ ] **H1** 纯静态 profile overlay 长跑 30 分钟 → 任务管理器 CPU 稳定，**无空转**（动态刷新被关掉的直接证据）
- [ ] **H2** （可选）手编 config.json 用 `builtin.time` + `bold: true` → overlay 文本**比 bold:false 明显加粗**、Preview 与 overlay 一致
- [ ] **H3** 字重支持的 serde 兼容：旧 Element JSON（无 `font_weight`）→ 渲染为常规字重（400），不报错

---

## 验证结果记录

| 日期 | 系统 | 版本 | 验证人 | A1-A3 | B1-B4 | C1-C4 | D1-D5 | E1-E3 | F1-F10 | G1-G3 | H1-H3 | 备注 |
|------|------|------|--------|-------|-------|-------|-------|-------|--------|-------|-------|------|
|      |      |      |        |       |       |       |       |       |        |       |       |      |

全部通过后：
```bash
# 更新 OpenSpec 任务状态，然后归档变更
openspec archive multi-profile-config
openspec archive material-static-rendering
openspec archive four-layer-customization
openspec archive merge-dev-into-four-layer
openspec archive settings-dev-mode
openspec archive add-glitchtip-telemetry
openspec archive unify-widget-fields
openspec archive overlay-dynamic-text-fixes
```
