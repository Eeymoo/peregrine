#!/usr/bin/env node
/**
 * i18n locale JSON 校验脚本（翻译 PR CI 校验门）。
 *
 * 三重校验：
 *   (a) JSON 可解析：6 份目标 locale JSON 都能成功 parse。
 *   (b) 单语 key 集合不变：每份被 PR 修改的 locale JSON，其扁平化 key 集合
 *       与 PR base（默认从 git 拿 `origin/<base_branch>` 的版本）完全一致。
 *       即只允许改 value，不允许加 / 删 key（加新 key 必须先动 zh-CN + en
 *       并配套所有 6 语）。
 *   (c) 6 语 key 集合对齐：6 份 locale JSON 扁平化后 key 集合完全一致。
 *
 * 退出码：0 全部通过，1 校验失败（红灯阻塞 merge）。
 *
 * 用法：
 *   node scripts/check-i18n.mjs                # 默认对照 origin/main
 *   node scripts/check-i18n.mjs origin/dev     # 指定 base 分支
 */
import { readFileSync } from "node:fs";
import { execSync } from "node:child_process";

const LOCALES = ["zh-CN", "en", "ja-JP", "de-DE", "fr-FR", "ru-RU"];
const LOCALE_DIR = "src/i18n/locales";
const BASE_BRANCH = process.argv[2] || "origin/main";

let failed = false;
const fail = (msg) => {
  console.error(`✗ ${msg}`);
  failed = true;
};
const ok = (msg) => console.log(`✓ ${msg}`);

/** 复刻 src/lib/i18n.tsx 的 flatten 逻辑（点号路径 → 字符串）。 */
function flatten(obj, prefix = "", out = new Map()) {
  if (obj && typeof obj === "object" && !Array.isArray(obj)) {
    for (const [k, v] of Object.entries(obj)) {
      const path = prefix ? `${prefix}.${k}` : k;
      if (typeof v === "string") out.set(path, v);
      else if (v && typeof v === "object") flatten(v, path, out);
    }
  }
  return out;
}

/** 读取并 parse 一份 locale JSON，失败时返回 null。 */
function loadLocale(file) {
  try {
    const raw = readFileSync(file, "utf8");
    return JSON.parse(raw);
  } catch (e) {
    return { __error: String(e) };
  }
}

/** 通过 git 拿 base 分支上某文件的版本（失败时返回 null）。 */
function loadBaseLocale(file) {
  try {
    const raw = execSync(`git show ${BASE_BRANCH}:${file}`, { stdio: ["pipe", "pipe", "pipe"] }).toString();
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

// (a) JSON 可解析 + (c) 6 语 key 集合对齐。
console.log(`\n=== (a) JSON 可解析 + (c) 6 语 key 集合对齐 ===`);
const currentKeys = new Map(); // locale -> Set<key>
for (const l of LOCALES) {
  const file = `${LOCALE_DIR}/${l}.json`;
  const parsed = loadLocale(file);
  if (parsed && parsed.__error) {
    fail(`${file} 解析失败：${parsed.__error}`);
    currentKeys.set(l, new Set());
    continue;
  }
  if (!parsed) {
    fail(`${file} 不存在或读取失败`);
    currentKeys.set(l, new Set());
    continue;
  }
  const keys = new Set(flatten(parsed).keys());
  currentKeys.set(l, keys);
  ok(`${file}：${keys.size} keys`);
}

let sixLocaleAligned = false; // (c) 结果：6 语 key 集合是否对齐
const baseSet = currentKeys.get("zh-CN");
if (baseSet) {
  sixLocaleAligned = true;
  for (const l of LOCALES) {
    if (l === "zh-CN") continue;
    const s = currentKeys.get(l);
    const missing = [...baseSet].filter((k) => !s.has(k));
    const extra = [...s].filter((k) => !baseSet.has(k));
    if (missing.length || extra.length) {
      fail(`${l}.json 与 zh-CN.json key 集合不一致：missing=[${missing.join(",")}] extra=[${extra.join(",")}]`);
      sixLocaleAligned = false;
    }
  }
}

// (b) 单语 key 集合不变（对照 BASE_BRANCH）。
// 设计意图：挡住"翻译修正 PR（auto-translate 生成）误改 key"。
// 逃生阀：feature PR 合法地 6 语同步加 / 删 key 时不应被误伤——
// 只要 (c) 6 语 key 集合仍对齐，且各 locale 的 key 变更（added / removed）彼此一致，
// 即视为"有意同步变更"，(b) 只输出 info 不 fail。
console.log(`\n=== (b) 单语 key 集合不变（对照 ${BASE_BRANCH}）===`);
const perLocaleChanges = new Map(); // locale -> { added: Set, removed: Set }
for (const l of LOCALES) {
  const file = `${LOCALE_DIR}/${l}.json`;
  const baseParsed = loadBaseLocale(file);
  if (!baseParsed) {
    console.log(`  ${file}：base 版本不可读（可能是新增文件或 base 分支不存在），跳过`);
    continue;
  }
  const baseKeys = new Set(flatten(baseParsed).keys());
  const curKeys = currentKeys.get(l);
  const added = new Set([...curKeys].filter((k) => !baseKeys.has(k)));
  const removed = new Set([...baseKeys].filter((k) => !curKeys.has(k)));
  perLocaleChanges.set(l, { added, removed });
  if (added.size === 0 && removed.size === 0) {
    ok(`${file} key 集合不变（${curKeys.size} keys）`);
  } else {
    console.log(
      `  ℹ ${file} key 集合变更：added=[${[...added].join(",")}] removed=[${[...removed].join(",")}]`,
    );
  }
}

// (b) 最终判定：若任一 locale 单方面变更（即非 6 语同步），且 (c) 未对齐，才 fail；
// 反之（所有可对照 locale 的 added / removed 集合一致 + (c) 对齐）视为 feature PR 合法同步，放过。
if (perLocaleChanges.size > 0) {
  // 取所有可对照 locale 的变更快照，验证它们彼此一致。
  const refLocale = [...perLocaleChanges.keys()][0];
  const ref = perLocaleChanges.get(refLocale);
  const refAdded = [...ref.added].sort().join(",");
  const refRemoved = [...ref.removed].sort().join(",");
  let allConsistent = true;
  for (const [l, ch] of perLocaleChanges) {
    const a = [...ch.added].sort().join(",");
    const r = [...ch.removed].sort().join(",");
    if (a !== refAdded || r !== refRemoved) {
      fail(`${l}.json 的 key 变更与 ${refLocale}.json 不一致：added=[${a}] removed=[${r}]（参照 ${refLocale}: added=[${refAdded}] removed=[${refRemoved}]）`);
      allConsistent = false;
    }
  }
  if (allConsistent) {
    if (refAdded || refRemoved) {
      // 6 语同步变更：若 (c) 对齐则放过，否则 fail（说明同步不完整）。
      if (sixLocaleAligned) {
        console.log(
          `\n✓ 6 语同步 key 变更（added=${refAdded || "(none)"} removed=${refRemoved || "(none)"}），(c) 6 语对齐仍通过——视为 feature PR 合法同步，(b) 放过`,
        );
      } else {
        fail(`6 语 key 变更彼此一致但 (c) 6 语对齐失败——同步不完整，请检查所有 locale JSON`);
      }
    }
    // 否则所有 locale 都无变更，(b) 自然通过。
  }
  // allConsistent=false 时各不一致条目已逐条 fail。
}

console.log("");
if (failed) {
  console.error("❌ i18n 校验失败");
  process.exit(1);
} else {
  console.log("✅ i18n 校验通过");
  process.exit(0);
}
