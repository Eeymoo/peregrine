# contributor-guide 规格增量

## ADDED Requirements

### Requirement: 根目录贡献指南入口文件

仓库根目录 SHALL 存在 `CONTRIBUTING.md`，作为贡献者引导的入口文件，在 GitHub 新建 issue / PR 场景下可被自动提示阅读。该文件 MUST 使用英文撰写，且 MUST 为「瘦指针」形态：仅包含欢迎语、指向文档站完整贡献指南的链接与常用贡献入口，MUST NOT 复制文档站贡献指南的正文内容（分支命名、commit 规范、开发流程、PR 流程等）。

#### Scenario: GitHub 场景引导贡献者

- **WHEN** 贡献者在 GitHub 上新建 issue 或 Pull Request
- **THEN** GitHub 提示阅读的 `CONTRIBUTING.md` 存在，且其中包含指向文档站完整贡献指南的链接

#### Scenario: 完整指南单一维护点

- **WHEN** 贡献流程（如分支命名、commit 规范、PR 流程）发生变更
- **THEN** 只需修改文档站贡献指南页面，根目录 `CONTRIBUTING.md` 无需同步修改

### Requirement: 双语完整指南可达性

根目录 `CONTRIBUTING.md` MUST 同时提供文档站英文页（`/guide/contributing.html`）与简体中文页（`/zh-cn/guide/contributing.html`）的链接，使中文读者可以一键跳转到中文完整指南。链接 MUST 使用文档站线上地址，与 `README.md` 中既有链接保持同源。

#### Scenario: 中文读者跳转

- **WHEN** 中文贡献者打开根目录 `CONTRIBUTING.md`
- **THEN** 文件内存在指向简体中文完整贡献指南的链接

#### Scenario: 英文读者跳转

- **WHEN** 英文贡献者打开根目录 `CONTRIBUTING.md`
- **THEN** 文件内存在指向英文完整贡献指南的链接

### Requirement: 注释与提交信息语言约定一致

文档站贡献指南（英文页与简体中文页）中的语言约定 MUST 与 `AGENTS.md` 保持一致：代码注释（`///`、`//!`）、文档与 commit message body 使用简体中文。文档站页面 MUST NOT 出现要求上述内容使用英文的表述。同时页面 MUST 澄清该约定不适用于 Issue 与 PR 描述——后者使用英文仍然被接受。

#### Scenario: 文档间约定无冲突

- **WHEN** 贡献者对照阅读 `AGENTS.md` 与文档站贡献指南
- **THEN** 两者对代码注释 / 文档 / commit message body 的语言要求一致（均为简体中文）

#### Scenario: 非中文贡献者不被误伤

- **WHEN** 非中文贡献者阅读文档站贡献指南的语言约定章节
- **THEN** 页面明确说明 Issue 与 PR 描述可使用英文
