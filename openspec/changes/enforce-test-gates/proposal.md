> 跟踪 issue：待创建（本环境 gh 无认证；维护者执行 `gh issue create --label feature,openspec` 后将编号回填于此与 `.openspec.yaml`）

## Why

2026-08 对组织四仓库审计发现：ci.yml 的完整测试套件（test-report job）存在 `continue-on-error: true` 与 `fail_on_failure: false`，测试失败不会阻塞合并——"流程即产品"的门禁形同虚设。同时 build job 仅在 x86_64 上运行测试，而 release 发布 i686 / x86_64 / aarch64 三架构，另两架构的二进制"编译通过但从未跑过测试"。按组织新立的"绕过显性"规则，任何跳过验证的开关必须有 change 记录原因与到期条件——本 change 即该记录，并以移除绕过为最终修复。

## What Changes

- ci.yml `test-report` job：删除 `Run all tests` 步骤的 `continue-on-error: true` 与 Publish Test Report 的 `fail_on_failure: false`，测试失败即 job 失败，quality-gate 随之拦截
- ci.yml `build` job：测试矩阵从单一 `x86_64-pc-windows-msvc` 扩展为 `i686 / x86_64 / aarch64` 三目标，与 release 发布架构对齐（Tauri 二进制构建保留 x86_64 单架构——前端产物与 NSIS 打包由 release/snapshot 工作流覆盖，CI 只验 Rust 侧）

## Capabilities

### Modified Capabilities

（Peregrine 基线 specs 无 CI 相关能力条目；本 change 属流程设施，不修改产品行为 spec）

## Impact

- **修改**：仅 `.github/workflows/ci.yml`
- **风险**：三架构测试增加 CI 时长（windows runner ×3 估算 +8–12 分钟/次）；若当前 main 存在失败测试，下次 push 将首次变红——这是门禁生效的预期行为，需当场修复而非恢复绕过
- **依据**：组织级规则（aukcraft AGENTS.md "Process rules"：绕过显性、勾选失效）
