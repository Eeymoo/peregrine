# i18n 审查结果（2026-08-03）

按 `.agent/skills/i18n-audit/SKILL.md` 流程执行首次全量审查。扫描范围：`src/`（tsx/ts），locale 文件 `src/i18n/locales/zh-CN.json` / `en.json`。

## 1. 硬编码 UI 文案（2 条确认 + 1 条灰区）

- `src/lib/globalErrorToast.ts:95` — 现状：`<summary>查看堆栈</summary>`（全局错误 toast 中用户可见）；建议：迁移为 `t("error.viewStack")`，双语条目需补充。
- `src/components/settings/UpdateTab.tsx:142` — 现状：`v4.gh-proxy.org（推荐）`；建议：迁移为 `{`v4.gh-proxy.org${t("settings.mirrorRecommended")}`}`，双语条目需补充。
- 灰区（建议保留）：`src/components/settings/UpdateTab.tsx:159` — `placeholder="https://your-mirror.example.com"` 为 URL 示例，语言中立，不迁移。

已剔除的命中：`AboutTab.tsx` 的 "Peregrine" / "Eeymoo"（专有名词）；注释、`console.*`、`logAction`、className 等按排除规则过滤。

## 2. 引用但缺失的 key（0 条）

无。`common.add`、`profile.selectPlaceholder` 等 235 个引用 key（含 `styles.*` / `hotkeyActions.*` / `anchors.*` / `ringStyles.*` / `borderStyles.*` / `gridAlignments.*` 动态拼接 key 按枚举展开）在两个 locale 文件中均存在。

## 3. 双语 key 不一致（0 条）

zh-CN 与 en 扁平化后 key 集合完全一致（各 262 条）。

## 4. 冗余 key（27 条，仅报告，不删除）

- `app.*`：`app.version`
- `layers.*`（物料运行时 / 变换区块软关闭相关，保留）：`layers.transformSection`、`layers.alpha`
- `common.*`：`common.ok`、`common.reload`、`common.reset`、`common.loading`、`common.error`
- `profile.*`：`profile.title`、`profile.copySuffix`、`profile.cannotDeleteLast`
- `overlay.*` / `overlaySettings.*`：`overlay.confirm`、`overlaySettings.fullscreenOverlay`、`overlaySettings.fullscreenOverlayHint`、`overlaySettings.windowModeHint`、`overlaySettings.fullscreenOnlyHint`
- `hotkeys.*`：`hotkeys.clear`、`hotkeys.pressCombo`
- `settings.*`：`settings.generalDesc`、`settings.overlayDesc`、`settings.updateDesc`、`settings.about.title`
- `fields.*`：`fields.gridCells`
- `tray.*`（后端 tray 菜单用，前端未引用）：`tray.config`、`tray.settings`、`tray.quit`、`tray.windowMode`

## 5. 需人工判断（1 条）

- `UpdateTab.tsx:159` 的 URL placeholder（见第 1 节灰区）。

## 统计

引用 key 235 / zh-CN 262 / en 262。
