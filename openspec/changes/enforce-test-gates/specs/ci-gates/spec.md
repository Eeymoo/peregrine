## ADDED Requirements

### Requirement: CI 测试门禁

CI SHALL 在测试失败时阻止合并：完整测试套件（test-report job）MUST NOT 使用 `continue-on-error` 或等效跳过机制；测试报告发布 MUST 在发现失败时使 job 失败。任何此类绕过开关的引入 MUST 经 change 记录原因与到期条件。

#### Scenario: 测试失败阻塞合并

- **WHEN** 任意 workspace 测试失败
- **THEN** test-report job 失败，quality-gate 汇总失败，PR 无法合并

### Requirement: 测试架构矩阵对齐发布架构

CI 库测试 SHALL 覆盖全部发布架构（i686 / x86_64 / aarch64 msvc）：build job 的测试步骤 MUST 在三目标上运行；发布产物与测试覆盖的目标集合 MUST NOT 出现"发布但未测试"的架构。

#### Scenario: 三架构测试执行

- **WHEN** CI build job 运行
- **THEN** matrix 包含 i686-pc-windows-msvc、x86_64-pc-windows-msvc、aarch64-pc-windows-msvc 三目标且各自执行 `cargo test`
