# 工作示例：Peregrine（shadcn 暗色 → aukcraft）

升级情形（`references/upgrade.md`）的完整实例。Peregrine 设置窗口为 Tauri + React + Tailwind + shadcn/ui，960×560 固定暗色，`src/index.css` 使用 shadcn 默认 zinc 暗色主题。整个升级 = 变量重映射 + 少量基础组件层覆盖，**不做组件重构**。

## 「现状 → 目标」映射表

| 现状（shadcn `.dark`） | 目标 | 说明 |
|---|---|---|
| `--background: 240 10% 3.9%` | `#0B0E11`（≈ `213 24% 5.5%`） | 带蓝调的近黑，不是中性锌灰 |
| `--card / --popover: 240 10% 3.9%` | `#14181D`（≈ `213 18% 9.6%`） | 卡片必须浮在窗口之上 |
| `--border / --input: 240 3.7% 15.9%` | 发丝线（8% 白） | 1px 发丝线，消灭可见的灰框；用不透明近似值时按叠加表面计算（见生成脚本输出注释） |
| `--foreground: 0 0% 98%` | `#EDEDED`（`0 0% 93%`） | 绝不用近纯白 |
| `--muted-foreground: 240 5% 64.9%` | `#8A9199`（`212 7% 57%`） | |
| `--primary: 0 0% 98%`（白色按钮！） | accent 梯度（品牌蓝 `#2563EB` 家族） | 白色主按钮是最大的品牌断裂点，优先处理 |
| `--ring: 240 4.9% 83.9%` | accent-400（`#60A5FA`） | 焦点 = accent，恒成立 |
| `--radius: 0.5rem` | `4px`（md 3px，sm 2px） | 消灭 shadcn 的药丸感 |
| card/popover 上的 `shadow-*` | 移除 | 层次由发丝线承担 |
| `focus-visible:ring-2 ring-offset-2` | 1px accent 描边，3px 偏移 | 更细，与发丝线语言一致 |
| 系统字体栈 | 保留，另加 JetBrains Mono 用于 micro 标签 | 桌面 app 留在系统字体——零 webfont 下载成本 |

## 改动落点

- `src/index.css`：替换 `.dark` 变量块（用 `scripts/generate_theme.py --accent "#2563EB" --accent-soft "#60A5FA" --format shadcn` 生成——`--accent-soft` 显式对齐 peregrine.aukcraft.org 已在用的 Tailwind blue-400；生成产物见 `assets/examples/peregrine-shadcn.css`）
- `src/components/ui/*`：class 级覆盖（radius、shadow、focus ring）
- `src/components/settings/` 等功能组件：**零改动**。需要改它们才能修对颜色，说明上游 token 定错了

## 保留清单（Peregrine 实例）

这些现有决策本来就符合规范，升级时不要动：

- **自定义自动隐藏滚动条**（默认透明、悬停淡入、6px，见 `index.css`）——保留；滑块颜色对齐 `muted` 梯度
- **Tabs + Cards 密度**（`p-6` 内容区、`space-y-6` 分组）——正确；不膨胀
- **固定暗色主题**——游戏悬浮层工具的正确选择；不加亮色模式
- **自研 i18n + 运行时切换**（`src/lib/i18n.tsx`）——正确模式
- **原生标题栏与托盘**——属于 OS，不在改造范围

## 容易踩的坑

- **零阴影后弹层读不出层次**：用 `raised` 表面 + 发丝线 + 遮罩解决，不要把阴影加回来
- **发丝线用了半透明值但组件直接 `border-border`**：shadcn 的 `hsl(var(--border))` 语法不带 alpha 通道，要么用脚本生成的不透明近似值，要么把用法改成 `hsl(var(--border) / 0.08)`
- **accent 浅阶对比度**：`#2563EB`（600 阶）直接用在 `#0B0E11` 上对比度不足，暗色界面必须用 400 阶 `#60A5FA`；脚本会自动检查并警告
