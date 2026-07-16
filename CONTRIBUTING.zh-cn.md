# 贡献指南

感谢你考虑为 Peregrine 贡献代码！本文档说明了参与项目的流程与规范。

---

## 快速开始

1. Fork 本仓库。
2. 创建功能分支：`git checkout -b feat/your-feature-name`。
3. 在本地开发和测试。
4. 确保所有测试通过：`cargo test`。
5. 提交 Pull Request。

---

## 分支命名规范

| 前缀 | 用途 |
| --- | --- |
| `feat/` | 新功能（如 `feat/add-linux-support`） |
| `fix/` | 修复 bug（如 `fix/overlay-crash`） |
| `refactor/` | 代码重构（如 `refactor/render-pipeline`） |
| `docs/` | 文档更新（如 `docs/contributing-guide`） |
| `ci/` | CI/CD 配置变更（如 `ci/add-macos-builder`） |
| `chore/` | 杂项（依赖更新、构建脚本等） |

请保持分支名简短且有描述性。

---

## 提交信息规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
<type>(<scope>): <简短描述>

<详细说明（可选）>
```

### type（必填）

| 类型 | 说明 |
| --- | --- |
| `feat` | 新功能 |
| `fix` | 修复 bug |
| `refactor` | 重构，不涉及功能变更 |
| `docs` | 文档 |
| `ci` | CI/CD |
| `chore` | 杂项 |
| `test` | 测试 |

### scope（可选）

指明影响范围，例如 `config`、`renderer`、`overlay`、`settings-ui`、`ci` 等。

### 示例

```
feat(overlay): add softbuffer pixel buffer approach
fix(renderer): fix opacity failure under sRGB
docs: add contributing guide and startup instructions
ci(release): only build and release Windows x86/x86_64/ARM64
```

随着项目国际化，提交信息主体建议使用**英文**描述内容，但 type/scope 保持英文。

---

## 开发流程

1. 从 `main` 分支创建功能分支。
2. 在分支上完成开发，**保持提交粒度合理**（不要一个提交塞太多改动）。
3. 提交前运行：
   ```bash
   cargo test          # 确保测试通过
   cargo fmt           # 保持代码格式统一
   cargo clippy        # 检查常见问题
   ```
4. 推送分支并创建 Pull Request 到 `main`。
5. PR 标题使用与提交信息相同的风格（如 `feat(overlay): add PNG image support`）。
6. 在 PR 描述中写明改动内容和测试方式。
7. 等待 Code Review，根据反馈修改。
8. 合并后分支会被删除。

---

## 代码风格

- 遵循标准 Rust 风格（`cargo fmt` 默认配置）。
- 随着项目国际化，公开项的文档注释建议改用**英文**（`///`），模块顶部用 `//!` 说明职责。
- 错误处理：库层统一用 `thiserror` 定义的 `ConfigError`，不要在库中 `panic`/`unwrap`。
- 日志使用 `tracing`，不要新增 `println!`/`eprintln!`。
- 新增字段时务必加 `#[serde(default)]` 保持向后兼容。
- 枚举序列化统一 `#[serde(rename_all = "snake_case")]`。

---

## 测试要求

- 新功能应附带对应的单元测试。
- `schema.rs` 中的校验规则变更需同步更新测试。
- 涉及配置结构改动后至少运行 `cargo test -p peregrine_config`。
- 涉及 tokio 的测试使用 `#[tokio::test]`；涉及文件系统事件的用 `#[tokio::test(flavor = "multi_thread")]`。

---

## Pull Request 流程

1. **创建 PR**：提交到本仓库的 `main` 分支。
2. **CI 检查**：提交后 GitHub Actions 会自动运行测试与 lint，必须全部通过。
3. **Code Review**：至少一位维护者审核后方可合并。
4. **合并方式**：采用 **Squash & Merge**，将分支上的多个提交压缩为一个提交合并到 `main`。

---

## 报告问题

提交 Issue 时请尽量包含以下信息：

- Peregrine 版本（标签或 commit hash）
- Windows 版本（如 Windows 10 / 11）
- 复现步骤
- 期望行为与实际行为
- 日志或截图（如有）

---

再次感谢你的贡献！
