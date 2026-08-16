/**
 * 内置物料展示文案的 locale 映射覆盖（change `i18n-builtin-materials` 设计 D2/D3）。
 *
 * 内置物料脚本（`crates/material/builtin/*.rhai`）内的 `// Name:` 与 schema `label`
 * 统一为 zh-CN 源文案，同时充当 locale 映射未命中时的回退兜底。本模块在消费
 * `MaterialInfo` 时按当前界面语言查表覆盖：
 *
 * - `materials.<name>.name`                      → 覆盖 `display_name`
 * - `materials.<name>.params.<paramKey>`         → 覆盖 `schema[].label`
 * - `materials.<name>.options.<paramKey>.<value>` → 覆盖 `schema[].options[].label`
 *   （`<value>` 为脚本原始值转字符串，数字 `4` → `"4"`）
 *
 * 仅对 `builtin === true` 的物料生效；用户物料原样展示脚本文案。
 * 查表未命中时保留脚本原文，MUST NOT 显示原始 key 字符串（依赖 `has` 存在性判断）。
 */

import type { MaterialInfo, MaterialSchemaEntry } from "@/types/config";

/** 翻译上下文最小接口：与 `useI18n()` 返回值的 `t` / `has` 对齐。 */
export interface MaterialTranslate {
  t: (key: string) => string;
  has: (key: string) => boolean;
}

/** 内置物料 id 前缀（`builtin.cross` → 映射 key 使用 `cross` 段）。 */
const BUILTIN_ID_PREFIX = "builtin.";

/** 覆盖单个 schema 条目的 label 与 options label（原地返回新对象，不改入参）。 */
function localizeSchemaEntry(
  name: string,
  entry: MaterialSchemaEntry,
  i18n: MaterialTranslate,
): MaterialSchemaEntry {
  const labelKey = `materials.${name}.params.${entry.key}`;
  const label = i18n.has(labelKey) ? i18n.t(labelKey) : entry.label;
  let options = entry.options;
  if (options) {
    options = options.map((opt) => {
      const optKey = `materials.${name}.options.${entry.key}.${String(opt.value)}`;
      return i18n.has(optKey) ? { ...opt, label: i18n.t(optKey) } : opt;
    });
  }
  return { ...entry, label, options };
}

/**
 * 按当前界面语言覆盖内置物料的展示文案。
 *
 * 用户物料或未命中映射的内置物料原样返回（未命中字段保留脚本原文）。
 */
export function localizeMaterial(
  material: MaterialInfo,
  i18n: MaterialTranslate,
): MaterialInfo {
  if (!material.builtin) return material;
  // id 形如 `builtin.cross`；稳妥起见兼容无前缀的裸 id。
  const name = material.id.startsWith(BUILTIN_ID_PREFIX)
    ? material.id.slice(BUILTIN_ID_PREFIX.length)
    : material.id;
  const nameKey = `materials.${name}.name`;
  return {
    ...material,
    display_name: i18n.has(nameKey) ? i18n.t(nameKey) : material.display_name,
    schema: material.schema.map((entry) => localizeSchemaEntry(name, entry, i18n)),
  };
}
