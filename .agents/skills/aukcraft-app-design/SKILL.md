---
name: aukcraft-app-design
description: Design rules for aukcraft desktop application UIs (Tauri, Electron, native, or any desktop toolkit). Use whenever building a new desktop app or restyling, upgrading, or auditing an existing one in the aukcraft family — settings windows, control panels, tray apps, overlay tools (e.g. Peregrine) — or when a desktop UI must visually belong to the aukcraft brand. Covers both scenarios — upgrading an existing UI (audit-first, token remap) and designing from scratch (token core first) — with framework-agnostic rules, a theme generator script (scripts/generate_theme.py) that produces shadcn/CSS/Tailwind themes from a product accent color, and a worked Peregrine example.
---

# aukcraft 桌面应用设计规范

aukcraft 品牌在桌面工具上的落地方式。网站是编辑级的作品；app 是安静待在用户手边的工具——用户已经熟悉的结构（tab、面板、设置行、操作流程）是资产，视觉服务于它，而不是反过来。

## 设计观

这套规则为什么长这样：

- **一致性来自共享内核，不来自共享界面。** app 与 aukcraft.org 读作一家人，靠的是同一套中性色 token 与纪律——而不是把网站的动态背景、板块节奏搬进窗口。载体不同，表达不同；内核相同，家族成立。
- **美观是纪律的副产品。** 零阴影、发丝线、≤4px 圆角、accent 配给——限制的目的不是限制，而是让窗口被添加任何功能之后仍然保持整体感。
- **accent 是产品身份，不是装饰。** 每个产品一个 accent，与产品站点对齐；只出现在引导视线的地方：焦点、主操作、链接、选中、实时状态。
- **每个窗口只有一个视觉重心。** 唯一 accent 主操作 + 明度/发丝线分层，视线落点是被设计出来的——小窗口尤其输不起第二个重心。
- **安静是工具的礼貌。** 网站可以表演（它是门面）；app 待在用户手边，无人触碰的窗口就是一块完全静止的表面。密度不等于堆叠：紧凑与拥挤的区别，在于每个元素是否挣得了自己的位置。
- **克制没有成本。** 这套规则里没有一条需要预算，它只需要事先做出决定。

## 情形选择：先读对应的 reference

| 情形 | 读这个 | 思路概要 |
|---|---|---|
| **升级现有项目**（如 Peregrine 换肤） | `references/upgrade.md` | 先审计现状 → 写「现状 → 目标」映射表 → 只改 token 与基础组件层，布局/组件库/流程不动 |
| **新建项目** | `references/new-project.md` | 从中性色核心直接生长 → 按工具形态选布局语法 → 第一天就按规则实现 |

完整的工作示例（Peregrine 的 shadcn 暗色主题 → aukcraft 的逐变量映射 + 保留清单）在 `references/peregrine-mapping.md`。

## 主题生成脚本

`scripts/generate_theme.py` 从中性色核心 + 一个产品 accent 自动生成完整主题，免去手算 HSL：

```bash
# shadcn/ui 暗色变量块（直接替换 index.css 里的 .dark）
python scripts/generate_theme.py --accent "#2563EB" --format shadcn

# 产品站点已有浅阶时显式传入以对齐（如 Tailwind blue-400）
python scripts/generate_theme.py --accent "#2563EB" --accent-soft "#60A5FA" --format shadcn

# 通用 CSS 自定义属性（任何栈可用） / Tailwind config 片段 / 平台无关 token JSON
python scripts/generate_theme.py --accent "#2563EB" --format css
python scripts/generate_theme.py --accent "#2563EB" --format tailwind
python scripts/generate_theme.py --accent "#2563EB" --format json
```

脚本内置三道门禁：**饱和度门禁**——过艳的颜色（HSV S > 0.90 且明度 ≥ 0.5 的霓虹区）直接拒绝生成，这是「不艳即舒适」的机器执行；**对比度检查**——自动推算（或被 `--accent-soft` 覆盖）暗色界面用的 accent 浅阶，对照 `base` 不足 4.5:1 时在 stderr 警告；**槽位塌缩门禁**——hover 级表面与 card 明度差 <3% 时拒绝生成。生成产物示例见 `assets/examples/peregrine-shadcn.css`。

## 共享核心（两种情形都不可协商）

### 中性色 token（原样落地）

| Token | 值 | 角色 |
|---|---|---|
| `base` | `#0B0E11` | 窗口背景 |
| `raised` | `#14181D` | 卡片、面板、弹层、侧栏 |
| `hover` | `#2C2F34` | 第三级表面：悬停底色、滑杆轨道、tab pill、kbd（10% 白叠加在 raised 上） |
| `hairline` | `rgba(255,255,255,0.08)` | 所有边框/分隔线，1px |
| `ink` | `#EDEDED` | 主文字、控件标签 |
| `muted` | `#8A9199` | 次要文字、占位符、禁用态 |
| `ease-lock` | `cubic-bezier(0.16, 1, 0.3, 1)` | 所有过渡 |

固定暗色主题；绝不使用纯 `#000` / `#FFF`。圆角一律 ≤ 4px（行内代码/kbd 可用 2px）。**app 自有界面上零阴影**——层次只靠明度差 + 1px 发丝线。（OS 自有的原生菜单、托盘、系统工具提示豁免。）

**三级表面是硬约束**：`base < raised < hover` 两两明度差 ≥ 3%。把悬停槽塌缩到 `raised` 会让浮层上的 hover/focus 反馈彻底消失——"界面死了"是最典型的不高级感来源。映射到 shadcn 槽位时：`card/popover → raised`，`muted/secondary/accent → hover`（与 shadcn 暗色默认的三槽同值结构同构）。

### 产品 accent 与组织 accent

aukcraft.org 使用 Auk Teal `#14B8A6`。**app 不自动继承它**——每个产品声明自己的一个 accent，并与自己的站点保持一致（Peregrine 用品牌蓝 `#2563EB`，暗色界面上用 400 阶 `#60A5FA` 以保证 ≥ 4.5:1 对比度）。家族一致性由中性色核心 + 下述规则表达，而不是共享 accent 颜色。

accent 是配给制的，只能出现在：

1. **焦点指示**——每个可交互元素的可见焦点环。
2. **每个窗口/对话框的唯一主操作**——一个 accent 文字色按钮；次要操作为 `ink` 文字 + 发丝线边框；同一目标的重复入口降级为普通下划线链接。
3. **链接**与**选中态标记**——选中 tab 的下划线、单选框的圆点、选中行的 1px accent 边缘。绝不用 accent 填充背景。
4. **实时状态**——采集/悬浮层/进程正在运行时的小 accent 圆点或 mono 标签。静态的"已开启"状态（如打开的开关）保持 `ink`，不用 accent。

accent 绝不用于：背景填充、徽标、装饰线、默认图标。

### 不要从网站移植的东西

- 不要 DotField / HeroCanvas / 任何动态背景。app 表面是实色——设置面板后面放一块活画布，既是视觉噪音又白烧 GPU/电量。
- 不要玻璃拟态 / backdrop-blur——`.glass` 的存在意义是让画布透出来；没有画布就没有它。
- 不要网站的板块节奏（`py-40`、`01 ─ LABEL` 板块开场、scroll-snap、滚动渐现）。app 的分组靠一条发丝线 + 一个 micro 标签，不靠编辑级留白。
- app chrome 里不要衬线斜体 / 宋体点缀。唯一的宽松例外：About 页可以放一句衬线点缀的标语，与产品站语气一致。
- 不要胶片噪点叠加层。

### 窗口内的字体与标签

- 正文/控件文字 13–14px；密集的次要文字 12–13px；窗口内部标题很少超过 16–18px semibold。桌面 app 留在系统字体栈——零 webfont 下载成本。
- **Micro 标签**（JetBrains Mono、全大写、`letter-spacing: 0.15em`、`muted` 色）用于分组标题与状态：`TARGET WINDOW`、`OVERLAY`、`RUNNING`。它们取代重量级标题，在工具尺度上给出家族的编辑感指纹。
- 控件高度：28px 紧凑行 / 32px 默认输入框与按钮 / 36px 主操作。8px 间距网格。

### 动效：比网站更少，且只由交互触发

- 控件过渡统一到 `ease-lock`；按钮 `:active` 缩放 0.98。
- 可选的 Flight Line（1px accent 周长描边，按钮 700ms / 输入框 500ms，hover/focus 期间整圈常亮）**只给主操作**——小窗口里一个描边元素是签名，五个就是赌场。
- 没有任何环境动效、待机动效、循环动效。网站 HeroCanvas 波浪的例外条款在 app 里不存在。无人触碰的窗口是一块完全静止的表面。
- `prefers-reduced-motion`：描边降级为直接变色，缩放禁用，行为零变化。只动画 `opacity` / `transform` / `stroke-dashoffset`。

## 交付前检查清单（共用）

情形特有的检查项在各自的 reference 里；以下是两种情形都要过的：

- [ ] 中性色核心原样落地（`#0B0E11` / `#14181D` / 发丝线 / `#EDEDED` / `#8A9199`）；只声明了一个产品 accent
- [ ] 圆角 ≤ 4px；app 自有界面零阴影；层次读作明度 + 发丝线
- [ ] accent 审计：仅焦点 / 唯一主操作 / 链接 / 选中 / 实时状态；无 accent 填充
- [ ] 每个可交互元素焦点可见；完整键盘可达；Escape 关闭对话框
- [ ] 无动态背景、无玻璃、无环境动效；reduced-motion 降级为直接变色
- [ ] 分组标题/状态用 micro 标签；chrome 无衬线；emoji 克制
