---
name: i18n-audit
description: 审查前端国际化（i18n）覆盖情况。当需要检查 src/ 下是否存在硬编码 UI 文案、t() 引用的 key 是否都在 locale 文件中存在、zh-CN 与 en 双语 key 是否对齐、是否存在未被引用的冗余 key 时使用。涉及"国际化审查""i18n 检查""文案补齐""双语对齐"等诉求均应触发。
---

## 使用对象

本 skill 面向 **AI 编码代理**。直接按下面的流程调用 Bash 执行命令、读取输出，再基于语义判断产出分类审查结果。

项目 i18n 结构（以本仓库为准，其他项目按实际调整）：

- locale 文件：`src/i18n/locales/zh-CN.json`、`src/i18n/locales/en.json`（嵌套 JSON，运行时由 `src/lib/i18n.tsx` 的 `flatten()` 扁平化为点号 key）。
- 取文案方式：组件内 `const { t } = useI18n();` 后调用 `t("namespace.key")`。
- 约定：**用户可见文案一律走 `t()`**；注释、日志、错误内部信息不强制国际化。

## 审查维度

按以下四个维度执行，产出四张清单：

1. **硬编码 UI 文案**：`src/` 下 JSX/TSX 中直接向用户展示、但未走 `t()` 的文本。
2. **缺失 key**：代码中 `t("...")` 引用、但在 zh-CN.json 或 en.json 中不存在的 key。
3. **双语对齐**：zh-CN.json 与 en.json 扁平化后 key 集合不一致的条目。
4. **冗余 key**：locale 文件存在但代码中无任何引用的 key（**仅报告，不强制删除**）。

## 排除规则（硬编码文案扫描）

以下内容**不算**硬编码 UI 文案，扫描结果中应剔除：

- 注释（`//`、`/* */`、`{/* */}`）。
- `console.*`、`logAction(...)`、`throw new Error(...)` 等日志 / 调试 / 内部错误信息。
- `className` 中的纯样式字符串、`key={...}`、`id`、data 属性。
- 纯符号 / 单字符（如 `"→"`、`"-"`、`"…"`），但 **图标 + 文字** 组合（如 `"▶ 开始"`）属于用户可见文案，需要国际化。
- 类型定义、枚举值、序列化字段名（如 `kind === "builtin"`）。
- 灰区（如错误提示 `setError(String(e))`）：标注为「需人工判断」，由代理结合上下文给出建议。

## 执行流程

在仓库根目录按顺序执行。

### 步骤 1：提取全部 `t()` 引用 key

```bash
rg -o 't\("[^"]+"\)' src/ --no-filename | sed 's/^t("//; s/")$//' | sort -u > /tmp/i18n-used-keys.txt
wc -l /tmp/i18n-used-keys.txt
```

注意：若代码中存在 `t(` 单引号或模板字符串用法（如 `` t(`key.${x}`) ``），需另行 `rg -o "t\\('[^']+'\\)" src/` 补充，动态拼接 key 单独记录为「动态 key，需人工核对」。

### 步骤 2：扁平化两个 locale 文件并对比

用一个 Node 单行脚本（复刻 `src/lib/i18n.tsx` 的 flatten 逻辑）输出三张清单：

```bash
node -e '
const fs = require("fs");
const flatten = (o, p = "", r = {}) => {
  for (const [k, v] of Object.entries(o)) {
    const path = p ? `${p}.${k}` : k;
    if (typeof v === "string") r[path] = v;
    else if (v && typeof v === "object" && !Array.isArray(v)) flatten(v, path, r);
  }
  return r;
};
const zh = flatten(JSON.parse(fs.readFileSync("src/i18n/locales/zh-CN.json", "utf8")));
const en = flatten(JSON.parse(fs.readFileSync("src/i18n/locales/en.json", "utf8")));
const used = new Set(fs.readFileSync("/tmp/i18n-used-keys.txt", "utf8").trim().split("\n").filter(Boolean));
const zhKeys = new Set(Object.keys(zh)), enKeys = new Set(Object.keys(en));
const missingZh = [...used].filter(k => !zhKeys.has(k));
const missingEn = [...used].filter(k => !enKeys.has(k));
const onlyZh = [...zhKeys].filter(k => !enKeys.has(k));
const onlyEn = [...enKeys].filter(k => !zhKeys.has(k));
const unused = [...zhKeys].filter(k => !used.has(k));
console.log("== 引用但 zh-CN 缺失 =="); missingZh.forEach(k => console.log("  " + k));
console.log("== 引用但 en 缺失 =="); missingEn.forEach(k => console.log("  " + k));
console.log("== 仅 zh-CN 有 =="); onlyZh.forEach(k => console.log("  " + k));
console.log("== 仅 en 有 =="); onlyEn.forEach(k => console.log("  " + k));
console.log("== 存在但未引用（冗余，仅报告）=="); unused.forEach(k => console.log("  " + k));
console.log(`== 统计 == 引用 ${used.size} / zh ${zhKeys.size} / en ${enKeys.size}`);
'
```

### 步骤 3：扫描疑似硬编码用户可见文案

```bash
# JSX 文本节点中的中文（>中文< 形式）
rg -n '[>"][^<>{}"]*[一-龥]+[^<>{}"]*[<"]' src/ --glob '*.tsx'

# JSX 文本节点 / 属性中的英文词组（两个及以上单词）
rg -n '>[^<>{}]*[A-Za-z]+\s+[A-Za-z]+[^<>{}]*<' src/ --glob '*.tsx'

# title= / placeholder= / aria-label= 属性中的硬编码字符串
rg -n '(title|placeholder|aria-label)="[^"]*[一-龥A-Za-z][^"]*"' src/ --glob '*.tsx'
```

对每条命中**逐条人工判断**：

- 命中排除规则（注释 / `console.*` / `logAction` / className / 类型值）→ 剔除。
- 已走 `t()`（如 `placeholder={t("...")}`）→ 剔除。
- 确认为用户可见文案 → 记入硬编码清单，给出建议 key（按既有命名空间习惯，如 `common.add`、`profile.new`）。
- 灰区 → 单独标注「需人工判断」。

## 输出格式

审查结果按分类清单输出，每条包含**文件路径 + 行号 + 现状 + 建议**：

```markdown
## i18n 审查结果（YYYY-MM-DD）

### 1. 硬编码 UI 文案（N 条）
- `src/components/Foo.tsx:42` — 现状：`>添加<`；建议：`{t("common.add")}`，双语条目已存在 / 需补充
- ...

### 2. 引用但缺失的 key（N 条）
- `profile.selectPlaceholder` — zh-CN 缺失 / en 缺失；引用位置：`src/components/ProfileManager.tsx:166`

### 3. 双语 key 不一致（N 条）
- 仅 zh-CN：`xxx.yyy`；仅 en：`aaa.bbb`

### 4. 冗余 key（N 条，仅报告）
- 按命名空间分组列出，如 `layers.*`（物料运行时软关闭相关，保留）

### 5. 需人工判断（N 条）
- ...
```

## 修复原则

- 缺失 key：**双语同时补**，文案风格与既有条目保持一致（中文简洁、英文首字母大写惯例参照同类 key）。
- 硬编码文案：迁移为 `t()` 调用并补充双语条目；优先复用已有 key（如 `common.add`）。
- 双语不齐：以实际使用方为准补齐另一侧，使两个 locale 扁平化后 key 集合完全一致。
- 冗余 key：**仅报告，不删除**（尤其是软关闭功能相关 key，如 `layers.transformSection`）。
- 修复完成后必须重新执行本流程复查：缺失 key 清单为空、双语 key 集合一致、无未修复的硬编码用户可见文案。
