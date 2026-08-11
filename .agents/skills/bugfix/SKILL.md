---
name: bugfix
description: Bug 修复流程规范。从理解问题到定位根因、最小修复、同类排查、验证闭环。当需要排查或修复 bug 时使用。
---

## 核心原则

**先理解，再动手。** 不要在未定位根因前盲目改代码——猜测式修复会掩盖真正的问题，引入新 bug。

**最小改动。** 只改与 bug 直接相关的代码，不顺手重构、不扩大范围。

**同类排查。** 修完一个点后，检查代码库中是否有相同模式的隐患——同类 bug 往往不止一处。

**分支隔离。** 每个修复在独立分支开发，不直接在 main 上提交。开发完成合并后删除分支。

## 流程

### 1. 理解问题

搞清楚三件事，再动代码：

| 要素 | 说明 |
| --- | --- |
| **触发条件** | 用户做了什么操作？在什么平台/配置下？ |
| **预期行为** | 正确应该是什么样？ |
| **实际行为** | 出了什么问题（崩溃 / 卡死 / 渲染异常 / 数据错误）？ |

如果用户描述模糊，**先问清楚再动手**，不要猜。

### 2. 新建分支

**始终在修复分支上开发，不直接动 main：**

```bash
# 从最新的 main 创建修复分支
git checkout main
git pull origin main
git checkout -b fix/<问题简述>

# 示例
git checkout -b fix/minimize-crash
git checkout -b fix/overlay-flicker
```

分支命名遵循 conventional commits 风格：`fix/` 前缀 + kebab-case 描述。

### 3. 收集信息

按顺序收集线索，从代价最低的开始：

```bash
# 1. 查日志（如果是崩溃，日志通常有最后的报错栈）
#    Peregrine 日志在 %APPDATA%/Peregrine/peregrine.log

# 2. 查最近的代码改动
git log --oneline -20

# 3. 查相关文件的改动历史
git log --oneline -10 -- <可疑文件路径>
```

### 4. 定位根因

通读**调用链上的所有相关代码**，不要只看用户提到的那个点。Peregrine 是事件驱动的双窗口架构，bug 往往出在事件流转的衔接处。

**方法：**

1. 从用户操作出发，沿事件链追代码：
   - 用户操作 → `WindowEvent` / `UserEvent` → `window_event()` / `user_event()` / `about_to_wait()` → 渲染器
2. 重点审查这些高危模式：

| 高危模式 | 说明 | 示例 |
| --- | --- | --- |
| **状态不对称** | 一处检查了某条件，另一处没检查 | `about_to_wait` 检查了 `hidden` 但没检查最小化 |
| **零尺寸/空值** | 窗口最小化、文件为空、buffer 长度为 0 | 向 0×0 surface 提交渲染 |
| **跨线程竞态** | 异步操作（`set_visible` / tokio task）与渲染的时序 | 窗口恢复显示的竞态窗口期 |
| **unwrap/expect** | 在可能失败的外部输入上直接 panic | `NonZeroU32::new(0).unwrap()` |
| **平台差异** | Windows 能跑但 macOS 崩，或反之 | alpha mode capabilities 差异 |

3. **找到根因后，用一句话向自己解释清楚**：什么条件下、哪行代码、为什么会导致观察到的现象。解释不通就还没找对。

### 5. 最小修复

- **只改与 bug 直接相关的代码**。发现的"顺便可以改"的问题记下来，不要在这个 commit 里一起改。
- **补守卫而非删逻辑**。优先在入口处加前置条件检查（如尺寸为 0 时提前返回），而不是改动核心渲染逻辑。
- **与现有代码风格一致**。看周围代码怎么做错误处理的（log + return / Result），照着来。
- **加注释说明为什么**。特别是非直觉的守卫，注释要解释"不加会怎样"，而非"加了什么"。

```rust
// ✅ 好注释：解释原因
// 最小化时 inner_size 为 0×0，向最小化窗口的 wgpu surface 持续提交
// 渲染会触发 GPU 设备丢失甚至 panic。
if size.width == 0 || size.height == 0 {
    return;
}

// ❌ 差注释：只复述代码
// 检查尺寸是否为 0
```

### 6. 同类排查

修完一个 bug 后，**主动搜索代码库中是否有相同模式的未保护路径**：

```bash
# 搜索所有调用同一接口的路径
rg "render_settings|render_overlay|request_redraw" crates/

# 搜索所有 unwrap/expect，审查是否有不安全的
rg "\.unwrap\(\)|\.expect\(" crates/ | rg -v "test"
```

对每个搜索结果逐一判断：**这个调用点有没有和我刚修的 bug 相同的隐患？**

- 有 → 一并修复，记在同一个 commit 里（因为属于同一类问题）。
- 无 → 确认安全后跳过。

### 7. 验证

| 检查项 | 命令 | 必须通过 |
| --- | --- | --- |
| 编译 | `cargo check` | ✅ |
| 格式 | `cargo fmt --all -- --check` | ✅ |
| 测试 | `cargo test -p peregrine_config` | ✅ |
| Lint | `cargo clippy -p peregrine_config -- -D warnings` | ✅ |

**不要跳过验证就报告"已修复"。** 如果无法在目标平台（Windows）实际复现验证，要如实说明"已在 macOS 编译通过，未在 Windows 实测"，不要谎称已验证。

### 8. 记录与提交

- **commit message 用中文**，`fix:` 前缀，简述修复的问题。
- 涉及用户可感知行为的修复，更新 `CHANGELOG_ALPHA.md`。
- 如果 bug 影响已发布的正式版，提醒用户是否需要发布补丁版本（走 release skill）。

```bash
git add -A
git commit -m "fix: 修复设置窗口最小化后闪退

最小化时 inner_size 为 0×0，about_to_wait 仍持续 request_redraw，
向最小化的 wgpu surface 提交渲染触发崩溃。补尺寸守卫。"
```

### 9. 合并

验证通过后合并到 main 并删除分支：

```bash
# 推送修复分支
git push origin fix/<问题简述>

# 切回 main 合并（使用 --no-ff 保留分支历史）
git checkout main
git merge --no-ff fix/<问题简述>

# 推送 main
git push origin main

# 删除本地和远程修复分支
git branch -d fix/<问题简述>
git push origin --delete fix/<问题简述>
```

如果项目使用 PR 工作流，则推送分支后在 GitHub 创建 PR，走 code review 后合并。

**合并到 main 前必须确认：**
1. 所有检查项通过（编译 / 格式 / lint / 测试）。
2. 修复在目标平台（Windows）实际有效——不能只靠 macOS 编译通过就声称已修复。

## 常见陷阱

- **修症状不修根因**：加个 `catch_unwind` 吞掉 panic，根因还在。永远先找到为什么 panic。
- **过度修复**：用户报一个最小化崩溃，顺手把整个渲染循环重构了。review 成本暴增，回退困难。
- **忽略平台差异**：在 macOS 上验证通过不等于 Windows 没问题，Peregrine 的 overlay 是 Windows 专有实现。
- **忘记用户意图**：修着修着偏离了用户最初的诉求，改了一堆无关的东西。每改一处前问自己"这和用户报的 bug 有关系吗？"。
- **直接在 main 上开发**：出了问题难以回退，也污染主线历史。永远先建分支。
- **谎报完成**：没编译就说"已修复"，或没在目标平台验证就说"已验证"。做不到的如实说明。
