---
name: codebase-audit
description: AI 在接手、审查或修改代码库前，先自动执行 git 历史审计，定位高变动热点、bug 聚集区、bus factor 风险与项目节奏，再决定阅读顺序与修改策略。
---

## 使用对象

本 skill 面向 **AI 编码代理**，不是给人看的命令合集。你应该直接调用 Bash 执行下面的命令，读取输出，基于输出做结构化分析，然后决定下一步行动。

## 核心原则

**先跑审计，再读代码。** AI 的优势是可以秒级跑完这些命令并交叉对比。不要靠目录结构猜测风险，让 git 历史告诉你哪里是高危区域。

**输出是决策输入，不是报告。** 你不是在生成一份“代码库健康报告”，而是在回答：
- 我该优先读哪些文件？
- 修改某文件前需要额外注意什么？
- 这个改动可能触发的回归风险在哪里？
- 是否需要询问用户关于团队、合并策略或历史背景？

**交叉验证才能定优先级。** 单一指标不可靠。只有“高变动 + 高 bug”重合的地方，才应列为最高风险。

## 自动审计流程

进入仓库根目录后，按顺序执行以下步骤。每一步都要读取 stdout，提炼关键信息。

### 步骤 1：高变动热点（Churn Hotspots）

执行：

```bash
git log --format=format: --name-only --since="1 year ago" | sort | uniq -c | sort -nr | head -20
```

AI 需要提取：
- 前 10 个高频文件的相对路径与修改次数。
- 这些文件在当前任务中是否涉及？如果涉及，风险等级上调一级。
- 是否出现配置文件、自动生成文件、测试快照等非业务文件霸榜？若是，提示数据可能失真。

### 步骤 2：Bug 聚集文件

执行：

```bash
git log -i -E --grep="fix|bug|broken|panic|crash|leak|race" --name-only --format='' | sort | uniq -c | sort -nr | head -20
```

AI 需要提取：
- 前 10 个与修复/崩溃/并发相关的文件。
- 与步骤 1 的 top 10 做交集。交集文件标记为 **最高风险**。
- 如果交集为空，降低“历史 bug 密度”权重，转而从代码结构判断风险。

### 步骤 3：贡献者分布（Bus Factor）

执行：

```bash
echo "=== all time ==="
git shortlog -sn --no-merges
echo "=== last 6 months ==="
git shortlog -sn --no-merges --since="6 months ago"
```

AI 需要提取：
- 全部历史 top 3 与近 6 个月 top 3 是否重合。
- 若历史主导贡献者近 6 个月未出现，标记为 **知识断层风险**。
- 若仅 1 人近期活跃，标记为 **单人维护**。

**注意**：squash-merge 会压缩作者信息。若输出里出现大量相同 committer 名字且 commit message 常为 `Merge pull request #...`，应提示“作者信息可能失真”。

### 步骤 4：项目节奏

执行：

```bash
git log --format='%ad' --date=format:'%Y-%m' | sort | uniq -c
```

AI 需要提取：
- 最近 6 个月的提交数量趋势。
- 是否有断崖式下跌或脉冲式尖峰。
- 结合 `CHANGELOG.md` / `CHANGELOG_ALPHA.md` / tag 历史判断是持续迭代还是发布驱动。

### 步骤 5：救火频率

执行：

```bash
git log --oneline --since="1 year ago" | grep -iE 'revert|hotfix|emergency|rollback|workaround|temporary|temp'
```

AI 需要提取：
- 近一年 revert/hotfix 数量与大致时间分布。
- 若近 3 个月频繁出现，标记为 **部署/测试流程不稳定**。
- 若结果为 0，不要直接得出“团队稳定”结论，可能是 commit message 不写这些词。

## 一键综合审计脚本

如需一次性收集上述所有指标，执行：

```bash
cat <<'EOF' > /tmp/peregrine_git_audit.sh
#!/usr/bin/env bash
set -euo pipefail

echo "===== CHURN HOTSPOTS (last 1 year) ====="
git log --format=format: --name-only --since="1 year ago" | sort | uniq -c | sort -nr | head -20

echo ""
echo "===== BUG FIX HOTSPOTS ====="
git log -i -E --grep="fix|bug|broken|panic|crash|leak|race" --name-only --format='' | sort | uniq -c | sort -nr | head -20

echo ""
echo "===== CONTRIBUTORS ALL TIME ====="
git shortlog -sn --no-merges | head -10

echo ""
echo "===== CONTRIBUTORS LAST 6 MONTHS ====="
git shortlog -sn --no-merges --since="6 months ago" | head -10

echo ""
echo "===== MONTHLY COMMIT VELOCITY ====="
git log --format='%ad' --date=format:'%Y-%m' | sort | uniq -c | tail -12

echo ""
echo "===== FIREFIGHTING (last 1 year) ====="
git log --oneline --since="1 year ago" | grep -iE 'revert|hotfix|emergency|rollback|workaround|temporary|temp' || echo "(none found)"
EOF
bash /tmp/peregrine_git_audit.sh
```

## 针对 Peregrine 的模块风险映射

拿到审计结果后，按以下映射判断风险等级：

| 模块/文件 | 典型高变动原因 | 修改前需额外关注 |
| --- | --- | --- |
| `crates/config/src/schema.rs` | 新增准心样式、配置字段、校验规则 | 序列化兼容（`#[serde(default)]`）、单元测试、默认值 |
| `crates/peregrine/src/shapes.rs` | 新增/调整几何图元 | 前端预览与覆盖层渲染一致性 |
| `crates/peregrine/src/overlay_renderer.rs` | 新图元光栅化、渲染性能调优 | Windows 平台验证、透明/穿透窗口行为 |
| `src-tauri/src/overlay.rs` | 窗口生命周期、事件循环、跟随逻辑 | 跨线程状态同步、目标窗口跟随竞态 |
| `src-tauri/src/platform/windows.rs` | Win32 API 调整 | 仅 Windows 生效，macOS/Linux 只能编译验证 |
| `src/App.tsx` / `src/components/` | UI 控件、预览 Canvas、设置面板 | 前端类型与后端 schema 一致性 |

若审计结果显示上述文件同时处于 **churn top 10** 和 **bug top 10**，在执行相关改动前应：
1. 优先阅读该文件近 6 个月的 `git log -p -- <file>`，理解历次修复的上下文。
2. 检查是否有未覆盖的边界条件或平台差异。
3. 必要时向用户确认测试/验证环境（尤其是 Windows overlay 行为）。

## AI 决策树

完成审计后，按以下逻辑决定下一步：

```
当前任务涉及文件是否在 churn top 10？
├─ 是 → 是否在 bug top 10？
│       ├─ 是 → 最高风险：先读该文件近 6 个月改动历史，再动手；修改后优先跑相关测试/验证。
│       └─ 否 → 中高风险：该文件活跃但 bug 密度不高，注意兼容性与回归测试。
└─ 否 → 是否在 bug top 10？
        ├─ 是 → 中风险：该文件虽不常改，但历史易出 bug，重点审查边界条件与错误处理。
        └─ 否 → 低风险：按常规流程阅读与修改。
```

如果审计发现以下信号，应主动向用户提问：
- 历史主要贡献者近 6 个月未出现，且你需要修改其留下的核心模块。
- 近 3 个月 revert/hotfix 频繁，而你正要改动相关文件。
- 文件作者信息因 squash-merge 完全失真，无法判断 bus factor。

## 局限与校准

- **提交消息质量决定 bug/救火指标的可靠性。** 若团队 commit message 不写 `fix:`、`hotfix:`、不写 revert 原因，命令 2 和 5 会失效。
- **squash-merge 会压缩作者信息。** 命令 3 的输出可能反映合并者而非实际开发者。
- **时间窗口可调整。** 对于年轻项目，可把 `--since="1 year ago"` 改为 `--since="6 months ago"` 或更短。
- **不能替代静态分析、类型检查与测试。** 这些指标只决定“从哪里开始”，不能替代逐行阅读。

## 参考

本 skill 原理来自 [Five git commands that tell you where a codebase hurts before you open a single file](https://piechowski.io/post/git-commands-before-reading-code/)，作者 Ally Piechowski。本文将其改写为 AI 可直接执行的审计流程与决策框架。
