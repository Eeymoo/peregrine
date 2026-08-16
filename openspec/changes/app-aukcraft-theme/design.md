# 设计：设置窗口 aukcraft 主题升级

## Context

Peregrine 设置窗口为 Tauri Webview（React 18 + Tailwind 3 + shadcn/ui），960×560 固定暗色。现状是 shadcn 默认锌灰暗色主题，与 aukcraft 家族视觉语言断裂（白色主按钮、锌灰底色、阴影层次、0.5rem 圆角）。`.agents/skills/aukcraft-app-design/` 技能已提供完整规范与 Peregrine 专用映射表（`references/peregrine-mapping.md`），以及主题生成脚本 `scripts/generate_theme.py`。

约束：升级情形规范要求**只改视觉层**——布局、tab、交互流程、组件库均为资产，零改动；功能组件应保持零改动（唯一例外：删除散落的 `shadow-*` 类，属于减法）。

## Goals / Non-Goals

**Goals:**

- 变量重映射承载几乎全部视觉变化（`src/index.css` 的 `:root` 块）
- 基础组件层（`src/components/ui/*`）做 class 级覆盖：剥阴影、收焦点环
- 固定暗色，直接写 `:root`，不提供 `.light`
- 每步可验证：`npm run build` 通过 + 真实窗口走查

**Non-Goals:**

- 不引入新依赖（不发 webfont，JetBrains Mono 走系统等宽栈回退）
- 不改功能组件结构（除删 `shadow-*`）
- 不做 Flight Line 描边（可选签名元素，留待后续单独评估）

## Decisions

### D1：主题变量由脚本生成，不手算 HSL

用 `generate_theme.py --accent "#2563EB" --accent-soft "#60A5FA" --format shadcn` 生成 `:root` 块。`--accent-soft` 显式传入 Tailwind blue-400，与产品站点 peregrine.aukcraft.org 已用浅阶对齐；脚本内置饱和度门禁与对比度检查，避免手算出错。

### D2：border/input 用不透明近似值而非半透明

技能规范的发丝线是 `rgba(255,255,255,0.08)`，但 shadcn 的 `hsl(var(--border))` 语法不带 alpha 通道。采用脚本输出的不透明近似值（8% 白叠加在 `#14181D` 上的计算结果 `218 9.3% 16.9%`），避免改全部组件的 border 用法。

### D3：shadcn 的 `--accent` 槽位保持中性

shadcn 的 `accent` 是悬停底色（hover:bg-accent），若放品牌蓝会造成大面积 accent 填充，违反配给纪律。品牌蓝只走 `--primary` 与 `--ring`。

**修订（D3 初版事故）**：D3 初版把 `--accent`/`--muted`/`--secondary` 全部拍成 raised（`#14181D`），导致 Card 上的 hover 底色与 Card 同色——悬停反馈全部死亡。修正为三级表面：`card/popover → raised`，`muted/secondary/accent → hover 级 #2C2F34`（10% 白叠加在 raised 上，明度差 9.2%），与 shadcn 暗色默认的三槽同值结构同构。

### D4：焦点环改为 1px accent 描边 + 2–3px 偏移

`ring-2 ring-offset-2` 是 shadcn 默认粗环，与发丝线语言不一致。统一改为 `ring-1 ring-ring ring-offset-background`：按钮/输入框等大控件偏移 3px，≤20px 小控件（checkbox/switch/radio/slider thumb）偏移 2px——3px 偏移对小控件比例失调。

### D5：零阴影后弹层层次靠 raised + 发丝线 + 遮罩

弹层（select 下拉、对话框）删除 `shadow-md/lg` 后仍可读：`--popover` 是 `#14181D`（比窗口底 `#0B0E11` 亮）+ 1px 发丝线边框 + 原有遮罩层，足以分层，不把阴影加回来。

### D6：`.micro-label` 只提供工具类，不批量替换

micro 标签（mono 全大写 0.15em 字距）是升级中唯一允许的新增元素，但当前设置面板的分组靠 Card 标题承担，批量替换属于布局改动、超出"只改视觉层"边界。本次只交付工具类，后续按需启用。

### D7：保留清单不动

自动隐藏滚动条（颜色已对 muted 梯度）、Tabs+Cards 密度、固定暗色、运行时 i18n、原生标题栏/托盘——这些本来就符合规范，零改动。

### D8：accent 消费场景矩阵（新增，修复 D1 的次生事故）

D1 把 `--primary` 换成品牌蓝后，`--primary` 的非按钮消费者被一并染色：41 根滑杆（Range + Thumb）、8 个开关（checked 态）、checkbox/radio 的未选中描边全部变蓝——违反配给纪律。按消费场景矩阵做组件级处置：

| 消费者 | 处置 |
|---|---|
| `slider` Range/Thumb | 中性化：`bg-foreground/60` / `border-foreground/60` |
| `switch` checked 态 | ink 填充：`data-[state=checked]:bg-foreground` |
| `checkbox`/`radio` 未选描边 | `border-primary` → `border-input`；checked 态保留 accent（选中标记） |
| `button` default 变体 | 保留 accent 蓝（各窗口默认按钮 ≤3 个，构成唯一视觉重心） |
| HotkeysTab 录音态 `bg-primary/10` | 保留（实时状态允许 accent） |

技能侧同步修复：`generate_theme.py` 加第三级表面与槽位塌缩门禁、`peregrine-mapping.md` 加消费场景矩阵并修订"白色主按钮"绝对判断、`upgrade.md` 插入槽位消费审计步骤（2.5）与动态三态验证清单、`SKILL.md` 中性色核心加 `hover` token。

## Risks / Trade-offs

- [发丝线近似值在 base 表面略偏亮] → 脚本按 raised 表面计算，base 上差异肉眼不可辨；若后续发现边界过亮，再按表面分别定义
- [零阴影后极暗环境下弹层可读性] → D5 的 raised + 发丝线 + 遮罩三重分层；已在 `#0B0E11` 底色上验证明度差 ≥4%
- [active:scale-[0.98] 在 Webview 中触发重排] → 只作用于 transform，合成层动画，无重排
- [`.dark` 类残留（`main.tsx` 若手动添加）] → 检查入口，主题直接写 `:root`，`.dark` 类加不加都生效

## Migration Plan

1. 替换 `src/index.css` 变量块（D1/D2/D3）
2. `src/components/ui/*` class 级覆盖（D4/D5）
3. 功能组件删 `shadow-*`（6 处）
4. `npm run build` 验证（tsc + Vite）
5. `npx tauri dev` 真实窗口走查交付清单

回滚策略：单 commit，直接 revert 即恢复 shadcn 默认主题。
