# 实施任务清单

## 1. 根目录 CONTRIBUTING.md

- [x] 1.1 在仓库根目录新建 `CONTRIBUTING.md`（英文瘦指针）：欢迎语 + 文档站完整指南双语链接（`https://peregrine.aukcraft.org/guide/contributing.html` 与 `https://peregrine.aukcraft.org/zh-cn/guide/contributing.html`）+ 快捷入口（issue 模板选择页、translation-improvement 模板、GitHub Discussions）
- [x] 1.2 校验文件中不含文档站贡献指南的正文内容（分支命名、commit 规范、PR 流程等不得复制），链接与 `README.md` 第 12 行同源

## 2. 文档站语言约定修正

- [x] 2.1 修改 `docs/guide/contributing.md`（英文页）：commit message body 语言约定由「English」改为「Simplified Chinese」（约 L68），代码文档注释语言约定由「English」改为「Simplified Chinese」（约 L93）
- [x] 2.2 在 `docs/guide/contributing.md` 语言约定相关章节补充澄清：中文约定仅针对代码注释 / 文档 / commit body，Issue 与 PR 描述使用英文仍然欢迎
- [x] 2.3 同步修改 `docs/zh-cn/guide/contributing.md`（中文页）中的对应表述与澄清说明
- [x] 2.4 复查两份页面，确认不再存在「注释 / commit body 用英文」的残留表述，且与 `AGENTS.md` 约定一致

## 3. 验证

- [x] 3.1 运行 `openspec validate add-contributing-readme`（或等效命令）确认变更产物合法
- [x] 3.2 本地构建文档站（`cd docs && npm ci && npm run docs:build`）确认两份修改后的页面无 VitePress 构建错误
- [x] 3.3 对照 spec 场景逐项自查：GitHub 入口存在、双语链接可达（链接地址正确）、语言约定无冲突、非中文贡献者澄清已写明
