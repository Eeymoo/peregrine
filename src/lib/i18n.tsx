import {
  createContext,
  useContext,
  useEffect,
  useState,
  useCallback,
  type ReactNode,
} from "react";
import { listen } from "@tauri-apps/api/event";
import { getConfig, updatePreferences } from "@/lib/api";
import zhCN from "@/i18n/locales/zh-CN.json";
import en from "@/i18n/locales/en.json";
import jaJP from "@/i18n/locales/ja-JP.json";
import deDE from "@/i18n/locales/de-DE.json";
import frFR from "@/i18n/locales/fr-FR.json";
import ruRU from "@/i18n/locales/ru-RU.json";
import options from "@/i18n/options.json";

/**
 * 支持的语言。`auto` 表示跟随系统语言。
 *
 * 受支持的具体 locale id 集合与后端 `src-tauri/src/lib.rs::SUPPORTED_LOCALES`
 * 必须一字不差对齐（见 design.md 决策 3「前后端 locale 映射表对齐」）。
 */
export type Locale =
  | "auto"
  | "zh-CN"
  | "en"
  | "ja-JP"
  | "de-DE"
  | "fr-FR"
  | "ru-RU";

/** 受支持的具体 locale id（不含 `auto`）。 */
export type ResolvedLocale = Exclude<Locale, "auto">;

/** 缺失 key 时的回退语言。对齐后端 `FALLBACK_LOCALE = "en"`。 */
const FALLBACK_LOCALE: ResolvedLocale = "en";
const LOCALE_EVENT = "peregrine:locale-changed";

const localeMap: Record<ResolvedLocale, Record<string, string>> = {
  "zh-CN": flatten(zhCN),
  en: flatten(en),
  "ja-JP": flatten(jaJP),
  "de-DE": flatten(deDE),
  "fr-FR": flatten(frFR),
  "ru-RU": flatten(ruRU),
};

/** 受支持的显式 locale id 集合（与 localeMap 的键保持同步）。 */
const SUPPORTED_LOCALES: ResolvedLocale[] = [
  "zh-CN",
  "en",
  "ja-JP",
  "de-DE",
  "fr-FR",
  "ru-RU",
];

/**
 * 语言选项，供设置页下拉框使用。
 *
 * 字段约定：
 * - `value`: Locale id（`auto` / `zh-CN` / ...），写入配置文件。
 * - `label`: 直接展示的文案（语言自名 / endonym，跨语言恒定，如「日本語」「Deutsch」）。
 * - `labelKey`: 可选，i18n key；存在时渲染前走 `t()` 翻译（用于 `option.follow_system`，
 *   让"跟随系统"在不同语言下显示对应译文）。
 */
export interface LanguageOption {
  value: Locale;
  label: string;
  labelKey?: string;
}

export const LANGUAGE_OPTIONS: LanguageOption[] = options.languages as LanguageOption[];

/** 将嵌套 JSON 对象扁平化为点号路径的字典。 */
function flatten(obj: unknown, prefix = ""): Record<string, string> {
  const result: Record<string, string> = {};
  if (typeof obj === "object" && obj !== null && !Array.isArray(obj)) {
    for (const [key, value] of Object.entries(obj)) {
      const path = prefix ? `${prefix}.${key}` : key;
      if (typeof value === "string") {
        result[path] = value;
      } else if (typeof value === "object" && value !== null) {
        Object.assign(result, flatten(value, path));
      }
    }
  }
  return result;
}

/**
 * 把任意系统 locale 字符串映射到受支持的 locale id。
 *
 * 前缀映射表与后端 `src-tauri/src/lib.rs::map_locale_prefix` 必须一字不差对齐：
 * `zh`→`zh-CN`、`en`→`en`、`ja`→`ja-JP`、`de`→`de-DE`、`fr`→`fr-FR`、`ru`→`ru-RU`，
 * 其它前缀回退到 `FALLBACK_LOCALE = "en"`。
 */
function mapLocalePrefix(locale: string): ResolvedLocale {
  const lower = locale.toLowerCase();
  if (lower.startsWith("zh")) return "zh-CN";
  if (lower.startsWith("en")) return "en";
  if (lower.startsWith("ja")) return "ja-JP";
  if (lower.startsWith("de")) return "de-DE";
  if (lower.startsWith("fr")) return "fr-FR";
  if (lower.startsWith("ru")) return "ru-RU";
  return FALLBACK_LOCALE;
}

/** 根据浏览器语言返回最匹配的受支持 locale。 */
export function detectLocale(): ResolvedLocale {
  return mapLocalePrefix(navigator.language);
}

/**
 * 将存储的 locale 解析为实际显示的语言。
 * `"auto"` 会根据系统语言实时解析；显式受支持 locale 直接 resolve；
 * 未识别值回退到 `detectLocale()`（与后端 `current_locale` 行为对齐）。
 */
export function resolveLocale(locale: Locale): ResolvedLocale {
  if (locale === "auto") return detectLocale();
  if ((SUPPORTED_LOCALES as string[]).includes(locale)) return locale;
  return detectLocale();
}

/** 根据当前选择的 locale 翻译 key（不依赖 React）。 */
export function translate(locale: Locale, key: string): string {
  const resolved = resolveLocale(locale);
  return localeMap[resolved][key] ?? localeMap[FALLBACK_LOCALE][key] ?? key;
}

/**
 * 判断 key 在当前语言或回退语言（en）中是否存在译文（不依赖 React）。
 *
 * 用于"命中才覆盖、未命中保留原文"的场景（如内置物料文案映射），
 * 避免依赖 `t()` 缺 key 返回 key 字符串的比对 hack。
 */
export function hasTranslation(locale: Locale, key: string): boolean {
  const resolved = resolveLocale(locale);
  return key in localeMap[resolved] || key in localeMap[FALLBACK_LOCALE];
}

interface I18nContextValue {
  /** 当前选择的 locale（可能是 `auto`）。 */
  locale: Locale;
  /** 当前实际显示的语言（已解析 `auto`）。 */
  resolvedLocale: Exclude<Locale, "auto">;
  setLocale: (locale: Locale) => void;
  t: (key: string) => string;
  /** 判断 key 在当前语言或回退语言（en）中是否存在译文。 */
  has: (key: string) => boolean;
}

const I18nContext = createContext<I18nContextValue | null>(null);

interface I18nProviderProps {
  children: ReactNode;
}

/**
 * 国际化上下文提供者。
 *
 * locale 从后端配置（config.json）读取，支持 `"auto"` 跟随系统语言。
 * 写入时通过 `update_preferences` 命令持久化到配置文件并广播给所有窗口。
 */
export function I18nProvider({ children }: I18nProviderProps) {
  const [locale, setLocaleState] = useState<Locale>("auto");

  // 初始化：从后端配置读取 locale，未设置则回退到 auto。
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const config = await getConfig();
        const saved = config.settings?.locale;
        // 接受 "auto" 或任何受支持 locale id；其它值回退到 "auto"。
        const valid: Locale =
          saved === "auto" || (SUPPORTED_LOCALES as string[]).includes(saved)
            ? (saved as Locale)
            : "auto";
        if (!cancelled) setLocaleState(valid);
        localStorage.setItem("peregrine:locale", valid);
      } catch {
        if (!cancelled) setLocaleState("auto");
        localStorage.setItem("peregrine:locale", "auto");
      }
    })();
    return () => { cancelled = true; };
  }, []);

  const setLocale = useCallback(async (next: Locale) => {
    setLocaleState(next);
    // 同步写入 localStorage，让全局错误 toast 等非 React 模块也能读取最新语言。
    localStorage.setItem("peregrine:locale", next);
    try {
      // 通过 update_preferences 写入配置，后端会广播 locale-changed 事件给所有窗口。
      await updatePreferences({ locale: next });
    } catch {
      // 非 Tauri 环境静默失败。
    }
  }, []);

  // 监听后端广播的语言变更事件，统一更新所有窗口的 React 状态。
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const setup = async () => {
      try {
        unlisten = await listen<string>(LOCALE_EVENT, (event) => {
          const next = event.payload;
          const valid: Locale =
            next === "auto" || (SUPPORTED_LOCALES as string[]).includes(next)
              ? (next as Locale)
              : "auto";
          setLocaleState(valid);
          localStorage.setItem("peregrine:locale", valid);
        });
      } catch {
        // 非 Tauri 环境下忽略监听失败。
      }
    };
    setup();
    return () => unlisten?.();
  }, []);

  const resolvedLocale = resolveLocale(locale);

  useEffect(() => {
    document.documentElement.lang = resolvedLocale;
  }, [resolvedLocale]);

  const value: I18nContextValue = {
    locale,
    resolvedLocale,
    setLocale,
    t: (key: string) => translate(locale, key),
    has: (key: string) => hasTranslation(locale, key),
  };

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

/** 在 React 组件中使用国际化。 */
export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext);
  if (!ctx) {
    throw new Error("useI18n must be used within <I18nProvider>");
  }
  return ctx;
}
