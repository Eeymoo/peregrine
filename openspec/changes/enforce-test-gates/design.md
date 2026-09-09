## Context

aukcraft 组织 2026-08 审计确立"绕过显性"规则：任何跳过验证的开关必须有 change 记录。Peregrine 的 ci.yml 存在两处无记录绕过（`continue-on-error: true`、`fail_on_failure: false`），且测试矩阵（x86_64 单目标）与发布架构（i686/x86_64/aarch64）错位。上游决策：组织级制度由 aukcraft/aukcraft 仓库 `institutionalize-verification` change 承载（含 AGENTS.md 三条规则与 verification-governance 基线 spec），本 change 是 Peregrine 仓库侧的对应执行。

## Goals / Non-Goals

**Goals:**
- 测试失败自此阻塞合并（test-report job 真实把门）
- 库测试覆盖全部三个发布架构

**Non-Goals:**
- 不动 release.yml / snapshot.yml（它们的签名打包流程已正确覆盖三架构）
- 不修任何测试本身（若门禁生效后暴露失败测试，另开 change 修）
- 不加新的检查类型（lint/frontend-report/i18n-check 已存在且真实把门）

## Decisions

- **D1 删除绕过而非注释保留**：直接移除 `continue-on-error: true` 与 `fail_on_failure: false`，在步骤上方留注释说明门禁规则与"绕过须开 change"的约束——按组织规则，死代码形式的开关本身就是再次被翻开的诱惑。
- **D2 矩阵扩展但 Tauri 构建限 x86_64**：`cargo test` 三目标全跑（这是本 change 的核心）；`Setup Node / npm ci / Build frontend / Build Tauri binary` 四步加 `if: matrix.target == 'x86_64-pc-windows-msvc'`。理由：前端产物与 NSIS 打包在 release/snapshot 工作流按三架构分别构建，CI 此处只验"Rust 后端能否编译"一次即可；三份重复的前端构建每次 PR 白付 ~6 分钟 × 3。
- **D3 不动 quality-gate**：它已正确聚合 test-report 结果（`needs.*.result == 'failure'` 即 exit 1），门禁失效的根因只在 test-report 内部的两处绕过。
- **D4 nextest 报告链保持**：JUnit 报告、PR 评论、artifact 上传全部保留——报告设施是好的，问题只在不拦截。

## Risks / Trade-offs

- [main 可能存在未被发现的失败测试，门禁生效后首次变红] → 预期行为；修复测试而非恢复绕过（注释中明示）
- [i686 / aarch64 测试可能暴露平台特定问题（如 cfg(windows) 路径）] → 正是本 change 的目的；如三架构全绿成本过高，可退守"测试 x86_64 + aarch64，i686 仅 build"并开 change 记录
- [CI 时长增加（windows runner ×3）] → rust-cache 按 target 分 key，增量构建缓解；估算 +8–12 分钟/PR

## Migration Plan

1. 分支 `feature/enforce-test-gates`（已建）→ 本地检查 → 推送
2. 维护者开 PR（本环境 gh 无认证，`/opsx:apply` 的 PR 步骤由维护者补：`gh pr create --base main --head feature/enforce-test-gates`）
3. 合并后 `/opsx:archive`（PR-merge gate 会校验 pr: 字段——由维护者补录 PR 号）

## Open Questions

（无——决策均来自组织级审计结论）
