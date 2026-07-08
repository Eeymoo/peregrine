---
name: feature
description: 新增功能开发流程规范。从理解需求、新建分支到实现、测试、合并。当需要开发新功能时使用。
---

## 核心原则

**先规划，再编码。** 在不了解需求全貌和代码结构前不要动手写代码，避免返工。

**分支隔离。** 每个功能在独立分支开发，不直接在 main 上提交。开发完成合并后删除分支。

**遵循架构。** 新代码必须符合项目分层原则与既有风格，不引入与周围代码格格不入的模式。

## 流程

### 1. 理解需求

动手前搞清楚：

| 要素 | 说明 |
| --- | --- |
| **目标** | 这个功能解决什么问题？用户期望什么效果？ |
| **范围** | 做到哪一步算完成？哪些不做？ |
| **验收标准** | 怎么判断功能正常工作？（用户能否操作、能否看到效果） |

需求模糊时**先问清楚再动手**。特别是 Peregrine 涉及覆盖层渲染时，要明确：
- 涉及哪些准心样式或配置项？
- 是否需要新增配置字段（持久化）？
- 预览（egui）与覆盖层（softbuffer）是否都要支持？

### 2. 新建分支

**始终在功能分支上开发，不直接动 main：**

```bash
# 从最新的 main 创建功能分支
git checkout main
git pull origin main
git checkout -b feat/<功能简述>

# 示例
git checkout -b feat/edge-arrows-style
git checkout -b feat/process-trigger
```

分支命名遵循 conventional commits 风格：`feat/` 前缀 + kebab-case 描述。

### 3. 理解现有代码

动手前通读相关代码，搞清楚要改哪里。参考 `AGENTS.md` 中的仓库结构与分层原则。

**Peregrine 新增可配置项时的标准改动点（四处同步）：**

| 改动点 | 文件 | 说明 |
| --- | --- | --- |
| ① schema | `crates/config/src/schema.rs` | 字段定义 + 默认值 + 校验 + `#[serde(default)]` |
| ② 几何 | `crates/peregrine/src/shapes.rs` | `build_shapes` 中新增图元定义 |
| ③ 设置面板 | `crates/peregrine/src/settings_ui.rs` | 编辑控件 + 预览绘制 |
| ④ 覆盖层渲染 | `crates/peregrine/src/overlay_renderer.rs` | 如引入新图元类型，补光栅化 |

`CustomImage` 与 `EdgeArrows` 是例外：前者由各渲染器单独 blit，后者只在 `shapes.rs` 生成。

### 4. 实现

- **读周围代码，匹配风格**。命名、注释密度、错误处理方式、序列化约定（`#[serde(rename_all = "snake_case")]`）都要与现有代码一致。
- **中文注释**。所有公开项写 `///` 中文文档注释，模块顶部用 `//!` 说明职责。
- **序列化兼容**。向已有结构新增字段时**必须**加 `#[serde(default)]` 或 `#[serde(default = "...")]`，保证旧配置文件仍可反序列化。
- **分层原则**。`peregrine_config` 不得依赖任何 UI / GPU / 窗口代码。平台相关逻辑只放在 `peregrine` 二进制 crate。
- **并发约定**。跨 tokio 与 winit 线程共享的配置快照用 `std::sync::Mutex`，不要替换为 `tokio::sync::Mutex`。
- **最小侵入**。不要为了新功能重构不相关的代码，不引入未确认可用的依赖。

### 5. 测试

| 检查项 | 命令 | 说明 |
| --- | --- | --- |
| 编译 | `cargo check` | 全 workspace |
| 格式 | `cargo fmt --all -- --check` | CI 严格检查 |
| Lint | `cargo clippy -p peregrine_config -- -D warnings` | config crate 警告视为错误 |
| 单元测试 | `cargo test -p peregrine_config` | 新增校验规则或默认值时必须同步更新测试 |

新增配置校验规则或默认值时，**同步新增或更新 `schema.rs` 中的 `#[cfg(test)] mod tests` 测试用例**。GUI crate 目前没有测试，但至少手动验证功能可用。

### 6. 提交

**开发过程中可多次提交，每次提交是一个完整的小步骤：**

```bash
git add -A
git commit -m "feat: 新增边框箭头准心样式

- schema 新增 EdgeArrows 枚举变体与相关参数
- shapes.rs 新增箭头图元几何定义
- settings_ui 新增样式选项与参数编辑控件
- overlay_renderer 新增箭头光栅化"
```

commit message 用中文，`feat:` 前缀。多个小提交优于一个巨型提交。

### 7. 合并

开发完成并验证通过后：

```bash
# 推送功能分支
git push origin feat/<功能简述>

# 切回 main 合并（使用 --no-ff 保留分支历史）
git checkout main
git merge --no-ff feat/<功能简述>

# 推送 main
git push origin main

# 删除本地和远程功能分支
git branch -d feat/<功能简述>
git push origin --delete feat/<功能简述>
```

如果项目使用 PR 工作流，则推送分支后在 GitHub 创建 PR，走 code review 后合并。

**合并到 main 前必须确认：**
1. 所有检查项通过（编译 / 格式 / lint / 测试）。
2. 功能在目标平台（Windows）实际可用——不能只靠 macOS 编译通过就声称完成。
3. `AGENTS.md` 中涉及的相关说明已同步更新（如新增了模块或改变了架构）。

## 常见陷阱

- **直接在 main 上开发**：出了问题难以回退，也污染主线历史。永远先建分支。
- **一个功能塞太多东西**：分支里夹带不相关的重构或修复。一个分支只做一件事。
- **忘记四处同步**：新增配置项只改了 schema，漏了 shapes / settings_ui / overlay_renderer，导致功能不完整。
- **破坏序列化兼容**：新增字段忘加 `#[serde(default)]`，用户旧配置文件加载失败。
- **引入未确认的依赖**：假设某个 crate 可用就直接加，没有检查 workspace 中是否已有、版本是否匹配。
- **谎报完成**：没编译就说"已完成"，或没在目标平台验证就说"已验证"。做不到的如实说明。
