<!--
本 PR 由 /opsx:apply 流程自动创建。
合并前请确认：
  1. 关联的 OpenSpec change 所有任务已完成（tasks.md 全部 [x]）
  2. CI（build / test / lint）全部通过
  3. 至少一名维护者 review 通过
合并后请运行 /opsx:archive 归档对应 change（archive 前置门禁：本 PR 必须处于 merged 状态）。
-->

## 关联 OpenSpec change

- Change: `<change-name>`（`openspec/changes/<change-name>/`）
- Issue: #<issue-number>
- 提案 / 设计 / 任务：proposal.md / design.md / tasks.md

## 变更摘要

<!-- 简述本 PR 做了什么，以及为什么。 -->

## 变更类型

- [ ] 新功能（feature）
- [ ] bug 修复（bugfix）
- [ ] 重构（refactor）
- [ ] 文档（docs）
- [ ] 构建 / CI（build）
- [ ] 其它

## 自检清单

- [ ] 对应 `tasks.md` 中所有任务已勾选 `[x]`
- [ ] 新增 / 修改的配置项在 `schema.rs`（字段 + 默认值 + 校验）与前端 `types/config.ts` 同步
- [ ] 新增文案已补齐到全部已支持语言 locale JSON（`src/i18n/locales/*.json`）
- [ ] `cargo test` / `cargo clippy` / `cargo fmt --check` 本地通过
- [ ] `npm run build`（含 `tsc` 类型检查）本地通过
- [ ] 公开 API / 结构体变更带 `#[serde(default)]`，旧配置文件可向后兼容加载
- [ ] 如引入新 Report Code，已在 `docs/guide/report-codes.md` 与 `report_code` / `REPORT_CODES` 同步登记

## 截图 / 录屏（可选）

<!-- 如果涉及 UI / overlay 效果，请附上前后对比。 -->
