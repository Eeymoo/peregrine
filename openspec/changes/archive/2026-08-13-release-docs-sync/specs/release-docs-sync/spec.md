## ADDED Requirements

### Requirement: 发版-文档同步强制时序

发版流程 MUST 遵循「先合 changelog、后打 tag」时序，并通过 CI 强制：打纯版本号 tag 前，`docs/src/content/docs/guide/changelog.md` 的最新版本条目 MUST 与 tag 版本一致，否则发布流程失败。

#### Scenario: release 前置闸门

- **WHEN** 推送纯版本号 tag（如 `v0.2.5`，不含 `-`）
- **THEN** release.yml 在构建前校验 changelog 最新 `## [vX.Y.Z]` 条目；若与 tag 版本不一致，job 失败且不创建 GitHub Release

#### Scenario: 预发布 tag 跳过闸门

- **WHEN** 推送带预发布后缀的 tag（如 `v0.2.3-alpha.0`，含 `-`）
- **THEN** 闸门跳过，不强制 changelog 对应条目（与 release.yml 通道判定一致）

#### Scenario: CI 一致性提前暴露

- **WHEN** 存在修改版本号（`package.json` / `Cargo.toml`）的 PR
- **THEN** ci.yml 的文档一致性 job 比对版本号与 changelog 最新条目，不一致时该 job 失败，在 merge 前提前暴露

### Requirement: changelog 版本条目格式约定

闸门校验依赖的 changelog 版本条目 MUST 遵循统一格式：行首为 `## [v<版本号>] — <日期>`，其中版本号段与 tag 版本可直接比对。

#### Scenario: 条目格式解析

- **WHEN** 解析 `docs/src/content/docs/guide/changelog.md`
- **THEN** 取首个匹配 `^## \[(v[\d.]+[^\]]*)\]` 的版本号段，作为「最新版本」与 tag 版本比对

#### Scenario: 无条目时失败

- **WHEN** changelog 中不存在任何 `## [v...]` 条目而推送纯版本号 tag
- **THEN** release 闸门失败，提示先补充 changelog 条目
