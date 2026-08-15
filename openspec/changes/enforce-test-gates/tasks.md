## 1. 测试门禁

- [x] 1.1 ci.yml `test-report`：删除 `Run all tests with JUnit report` 步骤的 `continue-on-error: true`，测试失败即步骤失败
- [x] 1.2 ci.yml `test-report`：Publish Test Report 的 `fail_on_failure: false` 改为 `true`（报告中发现失败同样拦截）

## 2. 测试矩阵对齐发布架构

- [x] 2.1 ci.yml `build` job matrix：`target` 扩展为 `i686-pc-windows-msvc / x86_64-pc-windows-msvc / aarch64-pc-windows-msvc`，库测试三架构全跑
- [x] 2.2 Tauri 二进制构建步骤仅在 x86_64 执行（matrix 内条件），避免三份前端构建重复

## 3. 验证

- [x] 3.1 YAML 解析通过；job/step 结构与原文件 diff 仅限预期改动
- [x] 3.2 本地检查全绿：`cargo fmt --check` ✓、`cargo clippy -D warnings` ✓、`cargo test`（config 85 + material 34 + peregrine 10 = 129 passed）✓、`npm run build` ✓
- [ ] 3.3 维护者开 PR（`gh pr create --base main --head feature/enforce-test-gates`）并将 PR 号回填 `.openspec.yaml`——本环境 gh 无认证凭据，无法由 agent 完成
- [ ] 3.4 PR 合并后执行 `/opsx:archive`（PR-merge gate 校验）
