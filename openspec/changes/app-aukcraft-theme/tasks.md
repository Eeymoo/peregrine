# 任务清单：app-aukcraft-theme

> 说明：本变更为补录档案——代码已在分支 `feature/app-aukcraft-theme` 的 commit `3b7420b` 中完成，以下任务均已执行并验证。

## 1. 主题变量重映射

- [x] 1.1 用 `generate_theme.py --accent "#2563EB" --accent-soft "#60A5FA" --format shadcn` 生成目标变量块
- [x] 1.2 替换 `src/index.css`：中性色核心落地 `:root`（固定暗色），`--radius: 4px`
- [x] 1.3 新增 aukcraft 原始 token 变量（`--color-base` / `--color-raised` / `--color-hairline` / `--color-ink` / `--color-muted` / `--color-accent` / `--color-accent-soft` / `--ease-lock`）

## 2. 基础组件层覆盖（src/components/ui/）

- [x] 2.1 剥除全部 `shadow-*`（card / tabs / kbd / select / switch）
- [x] 2.2 焦点环改为 1px accent 描边 + 2–3px 偏移（大控件 3px，≤20px 小控件 2px）
- [x] 2.3 button 补 `:active` 缩放 0.98
- [x] 2.4 accent 消费场景矩阵落地：slider Range/Thumb 改 `foreground/60` 中性；switch checked 改 `bg-foreground`；checkbox/radio 未选描边改 `border-input`（选中标记保留 accent）

## 3. 功能组件减法

- [x] 3.1 删除 6 处散落的 `shadow-*`（UpdateProgress / UpdateDialog / AutoSwitchDialog / LayerPanel / LayersEditor / ConfigApp），其余零改动
- [x] 3.2 修复 3.1 引入的 class 粘连 bug（`roundedhover:bg-primary/90`，LayersEditor / ConfigApp 各 1 处）

## 4. 动效与降级

- [x] 4.1 `src/index.css` 新增交互元素 `ease-lock` 全局过渡
- [x] 4.2 新增 `prefers-reduced-motion` 降级（动画/过渡时长归零）

## 5. 新增工具

- [x] 5.1 新增 `.micro-label` 工具类（系统等宽栈、全大写、0.15em 字距、muted 色）——仅提供，不批量替换

## 6. 验证

- [x] 6.1 `npm run build` 通过（tsc 类型检查 + Vite 打包无错误）
- [x] 6.2 对照技能交付清单自查：中性色核心 / accent 配给 / 零阴影 / 圆角 ≤4px / 焦点可见 / 无环境动效 / 保留清单未动
- [x] 6.3 三级表面静态验证：`--card`（9.6% L）与 hover 系三槽（18.8% L）明度差 9.2% ≥ 3%

## 7. 技能侧修复（事故复盘 → aukcraft-app-design 规范修订）

- [x] 7.1 `generate_theme.py`：新增第三级表面 `hover`（#2C2F34）+ 槽位塌缩门禁（明度差 <3% 拒绝生成），四种输出格式同步
- [x] 7.2 `peregrine-mapping.md`：新增「accent 消费场景矩阵」+ 修订"白色主按钮 = 品牌断裂点"绝对判断 + 新增两条踩坑记录
- [x] 7.3 `upgrade.md`：插入步骤 2.5「槽位消费审计（不可跳过）」+ 焦点环小控件放宽 + 动态三态验证清单
- [x] 7.4 `SKILL.md`：中性色核心加 `hover` token + 三级表面硬约束说明 + 三道门禁描述
- [ ] 6.3 `npx tauri dev` 真实窗口（960×560）人工走查（需 Windows 环境，合并前由维护者确认）
