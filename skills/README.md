# Peregrine 官方 Skills

面向 AI 编码代理（Claude Code / OpenCode / Codex / Cursor 等）的官方技能目录。每个子目录是一个独立 skill，核心是 `SKILL.md`（Agent Skills 规范：YAML frontmatter 的 `name` + `description` 决定触发时机）。

## 目录

| Skill | 用途 |
|---|---|
| [`material-creation/`](./material-creation/) | 为 Peregrine 创建自定义物料（`.rhai` 脚本）：从视觉需求选型图元、编写契约函数、写入用户物料目录并验证热加载 |

## 方式一：`npx skills` 安装（推荐）

本目录遵循 [skills.sh](https://skills.sh) 开放技能生态的仓库布局（`skills/` 根目录），可直接用 [`skills` CLI](https://github.com/vercel-labs/skills) 安装：

```bash
# 安装本仓库的全部 skills（交互式选择目标代理与技能）
npx skills add eeymoo/peregrine

# 只安装 material-creation 这一个 skill（指向仓库内路径）
npx skills add https://github.com/eeymoo/peregrine/tree/main/skills/material-creation

# 不安装、临时使用：生成该 skill 的提示词并直接喂给 claude
npx skills use eeymoo/peregrine@material-creation | claude
```

CLI 会自动识别你机器上已装的代理（Claude Code / OpenCode / Codex / Cursor 等七十餘种），把 skill 文件写入对应目录（如 Claude Code 的 `.claude/skills/`、OpenCode 的 `.opencode/skill/`）。私有仓库同样支持——复用你本地的 git 凭据。

已克隆本仓库的用户也可以从本地路径安装：

```bash
npx skills add ./skills/material-creation
```

## 方式二：手动安装（任意 LLM 代理）

一句话即可——把下面这句发给任意 LLM 代理（它会自行运行命令并按生成的指令安装执行）：

> Run `npx skills use "https://github.com/eeymoo/peregrine" --skill "material-creation"` and follow the generated skill instructions now. Read its complete output, redirecting it to a temporary file first if necessary. Resolve relative paths from the supporting-files directory it provides.

已克隆本仓库的用户更简短：

> Run `npx skills use "./skills" --skill "material-creation"` and follow the generated skill instructions now.

## 使用案例

安装后，在**任意**支持技能的编码代理里用自然语言触发即可。

**案例 1：静态物料**

> 帮我做一个中心带空心圆环的十字准星，圆环大小和十字臂长可调。

代理会触发 `material-creation`，生成一份含 `defaults()` / `schema()`（`ring_radius` / `arm_length` 两个 slider 参数）/ `build()`（`circle_stroke` + 4 个 `rect`）的 `.rhai` 脚本，写入 `%APPDATA%/Peregrine/materials/`（Windows），并从日志确认 `loaded user material`。

**案例 2：动态物料**

> 我想要一个跟着呼吸节奏缓缓胀缩的圆点，速度要能调。

代理会声明 `is_dynamic() = true`，按呼吸周期驱动 `time_ms()` 动画，声明 `refresh_interval_ms()` 节流，并对输出半径做 0.5px 量化以跳过无变化帧的光栅化。

**案例 3：SVG 路径准心**

> 把这个 SVG 路径 `M 0 -10 Q 10 0 0 10 Q -10 0 0 -10 Z` 做成准心，颜色跟图层走。

代理会用 `path` 图元 + `parse_svg_path(d)` 解析，省略颜色字段以继承图层基色（换色热键保持生效）。

物料写入后约 500ms 自动热加载，无需重启应用；在 **设置 → 图层** 添加 `user.<文件名>` 即可看到效果。完整人类向文档见[物料脚本创作](https://peregrine.aukcraft.org/zh-cn/guide/material-scripting/)。
