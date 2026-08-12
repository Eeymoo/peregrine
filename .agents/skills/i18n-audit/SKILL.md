---
name: i18n-audit
description: 审查前端国际化（i18n）覆盖情况。当需要检查 src/ 下是否存在硬编码 UI 文案、t() 引用的 key 是否都在 locale 文件中存在、6 语（zh-CN / en / ja-JP / de-DE / fr-FR / ru-RU）key 集合是否对齐、目标 locale JSON 文件是否齐全、是否存在未被引用的冗余 key 时使用。涉及"国际化审查""i18n 检查""文案补齐""6 语对齐"等诉求均应触发。
---

## 使用对象

本 skill 面向 **AI 编码代理**。直接按下面的流程调用 Bash 执行命令、读取输出，再基于语义判断产出分类审查结果。

项目 i18n 结构（以本仓库为准，其他项目按实际调整）：

- locale 文件：`src/i18n/locales/zh-CN.json`、`en.json`、`ja-JP.json`、`de-DE.json`、`fr-FR.json`、`ru-RU.json`（共 **6 份**，嵌套 JSON，运行时由 `src/lib/i18n.tsx` 的 `flatten()` 扁平化为点号 key）。
- 取文案方式：组件内 `const { t } = useI18n();` 后调用 `t("namespace.key")`。
- 约定：**用户可见文案一律走 `t()`**；注释、日志、错误内部信息不强制国际化。

## 审查维度

按以下维度执行，产出分类清单：

1. **目标 locale 文件齐全**：`src/i18n/locales/` 下 6 份目标 JSON（zh-CN / en / ja-JP / de-DE / fr-FR / ru-RU）必须全部存在；任一缺失则在报告顶部明确指出（缺失文件时无法做后续 key 对齐，必须先补齐）。
2. **硬编码 UI 文案**：`src/` 下 JSX/TSX 中直接向用户展示、但未走 `t()` 的文本。
3. **缺失 key**：代码中 `t("...")` 引用、但在某份 locale JSON 中不存在的 key（per-locale 报告）。
4. **6 语对齐**：6 份 locale JSON 扁平化后 key 集合不一致的条目（per-locale 缺失 + per-locale 冗余）。
5. **冗余 key**：locale 文件存在但代码中无任何引用的 key（**仅报告，不强制删除**）。

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

### 步骤 1：检查目标 locale 文件齐全

```bash
ls src/i18n/locales/{zh-CN,en,ja-JP,de-DE,fr-FR,ru-RU}.json
```

任一文件不存在时，**报告顶部明确指出缺失文件**，要求补齐后再次审查，后续步骤仍执行（已存在文件之间）但明确标注「不完整」。

### 步骤 2：提取全部 `t()` 引用 key

```bash
rg -o 't\("[^"]+"\)' src/ --no-filename | sed 's/^t("//; s/")$//' | sort -u > /tmp/i18n-used-keys.txt
wc -l /tmp/i18n-used-keys.txt
```

注意：若代码中存在 `t(` 单引号或模板字符串用法（如 `` t(`key.${x}`) ``），需另行 `rg -o "t\\('[^']+'\\)" src/` 补充，动态拼接 key 单独记录为「动态 key，需人工核对」。

### 步骤 3：扁平化 6 份 locale 文件并对比（含结构化 JSON 输出）

用一个 Node 单行脚本（复刻 `src/lib/i18n.tsx` 的 flatten 逻辑）输出清单，
**同时生成结构化缺失清单**到 `.agents/skills/i18n-audit/output/missing-keys.json`
（该 JSON 是 AI agent 批量补齐缺失翻译的直接输入）：

```bash
mkdir -p .agents/skills/i18n-audit/output
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
const LOCALES = ["zh-CN", "en", "ja-JP", "de-DE", "fr-FR", "ru-RU"];
const maps = {};
for (const l of LOCALES) {
  try {
    maps[l] = flatten(JSON.parse(fs.readFileSync(`src/i18n/locales/${l}.json`, "utf8")));
  } catch (e) {
    maps[l] = {}; // 文件缺失时记为空，后续对比会报告
  }
}
const used = new Set(fs.readFileSync("/tmp/i18n-used-keys.txt", "utf8").trim().split("\n").filter(Boolean));

// 维度 3：per-locale 缺失 key（代码引用但 locale 文件没有）。
console.log("== 维度 3：引用但 locale 缺失 ==");
for (const l of LOCALES) {
  const keys = new Set(Object.keys(maps[l]));
  const missing = [...used].filter(k => !keys.has(k));
  if (missing.length) {
    console.log(`  ${l}：`);
    missing.forEach(k => console.log(`    ${k}`));
  }
}

// 维度 4：6 语对齐（per-locale 缺失 + per-locale 冗余）。
const allKeys = new Set();
for (const l of LOCALES) Object.keys(maps[l]).forEach(k => allKeys.add(k));
const missingReport = []; // 结构化缺失清单
const extraReport = [];   // 结构化冗余清单
for (const key of allKeys) {
  const missingIn = LOCALES.filter(l => !(key in maps[l]));
  if (missingIn.length > 0 && missingIn.length < LOCALES.length) {
    const presentIn = {};
    for (const l of LOCALES) if (key in maps[l]) presentIn[l] = maps[l][key];
    missingReport.push({ key, missing_in: missingIn, present_in: presentIn });
  }
}
// 6 语全缺（极端）由维度 3 兜底，这里只关心部分缺。

// 以 zh-CN 为基准（项目原始语义源）对齐时，把 zh-CN 有而其它没有的 key 视为冗余在其它语；
// 反之 zh-CN 没有而其它有的视为冗余在其它语（按 per-locale 报告）。
for (const l of LOCALES) {
  if (l === "zh-CN") continue;
  const onlyInL = Object.keys(maps[l]).filter(k => !(k in maps["zh-CN"]));
  for (const key of onlyInL) {
    extraReport.push({ key, only_in: [l] });
  }
}

console.log("== 维度 4：6 语对齐 ==");
console.log(`  per-locale 缺失（部分 locale 缺该 key）：${missingReport.length} 条`);
missingReport.forEach(r => console.log(`    ${r.key} — 缺于：${r.missing_in.join(", ")}`));
console.log(`  per-locale 冗余（该 locale 有，zh-CN 没有）：${extraReport.length} 条`);
extraReport.forEach(r => console.log(`    ${r.key} — 仅 ${r.only_in.join(", ")}`));

// 维度 5：冗余 key（locale 有但代码无引用）。
console.log("== 维度 5：存在但未引用（冗余，仅报告）==");
for (const l of LOCALES) {
  const keys = new Set(Object.keys(maps[l]));
  const unused = [...keys].filter(k => !used.has(k));
  if (unused.length) {
    console.log(`  ${l}：${unused.length} 条`);
    unused.forEach(k => console.log(`    ${k}`));
  }
}

// 统计
console.log("== 统计 ==");
console.log(`  引用 ${used.size}`);
for (const l of LOCALES) console.log(`  ${l}: ${Object.keys(maps[l]).length}`);

// 结构化 JSON 缺失清单：供 AI agent 批量补齐翻译直接消费。
const structured = {
  generated_at: new Date().toISOString(),
  locales: LOCALES,
  missing: missingReport,   // [{key, missing_in: [locale...], present_in: {locale: value}}]
  extra: extraReport,        // [{key, only_in: [locale...]}]
};
fs.writeFileSync(".agents/skills/i18n-audit/output/missing-keys.json", JSON.stringify(structured, null, 2));
console.log(`  结构化缺失清单：.agents/skills/i18n-audit/output/missing-keys.json`);
'
```

### 步骤 4：扫描疑似硬编码用户可见文案

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

### 0. 目标 locale 文件齐全（必须先检查）
- 6 份齐全 ✓ / 缺失：fr-FR.json（请先补齐后再次审查）

### 1. 硬编码 UI 文案（N 条）
- `src/components/Foo.tsx:42` — 现状：`>添加<`；建议：`{t("common.add")}`，6 语条目已存在 / 需补充
- ...

### 2. 引用但缺失的 key（N 条，per-locale）
- `profile.selectPlaceholder` — ja-JP / fr-FR 缺失；引用位置：`src/components/ProfileManager.tsx:166`

### 3. 6 语 key 不一致（N 条）
- per-locale 缺失：`xxx.yyy` 缺于 ja-JP, fr-FR
- per-locale 冗余：`aaa.bbb` 仅 ru-RU 有

### 4. 冗余 key（N 条，仅报告）
- 按命名空间分组列出，如 `layers.*`（物料运行时软关闭相关，保留）

### 5. 需人工判断（N 条）
- ...

### 结构化缺失清单
- `.agents/skills/i18n-audit/output/missing-keys.json`（含 `missing[]` / `extra[]`，供 AI agent 批量补齐翻译直接消费）
```

## 修复原则

- 缺失 key：**6 语同时补**，文案风格与既有条目保持一致（中文简洁、英文首字母大写、其它语言沿用相应习惯）。
- 硬编码文案：迁移为 `t()` 调用并补充 6 语条目；优先复用已有 key（如 `common.add`）。
- 6 语不齐：以 `zh-CN.json`（项目原始语义源）为基准补齐其它 5 份，使 6 份 locale 扁平化后 key 集合完全一致。
- 冗余 key：**仅报告，不删除**（尤其是软关闭功能相关 key，如 `layers.transformSection`）。
- 修复完成后必须重新执行本流程复查：缺失 key 清单为空、6 语 key 集合一致、无未修复的硬编码用户可见文案。
- AI agent 批量补齐翻译时，直接读取 `.agents/skills/i18n-audit/output/missing-keys.json`，按 `missing[]` 中每条 key 的 `present_in`（已有译文）翻译为 `missing_in` 列出的语言，遵循"以 en 为翻译源 + 以 zh-CN 为校对源"的双源策略。
