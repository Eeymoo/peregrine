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
import options from "@/i18n/options.json";

/** 支持的语言，`auto` 表示跟随系统语言。 */
export type Locale = "auto" | "zh-CN" | "en";

const FALLBACK_LOCALE: Exclude<Locale, "auto"> = "zh-CN";
const LOCALE_EVENT = "peregrine:locale-changed";

const localeMap: Record<Exclude<Locale, "auto">, Record<string, string>> = {
  "zh-CN": flatten(zhCN),
  en: flatten(en),
};

/** 语言选项，供设置页下拉框使用。 */
export const LANGUAGE_OPTIONS = options.languages as { value: Locale; label: string }[];

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

/** 根据浏览器语言返回最匹配的受支持 locale。 */
export function detectLocale(): Exclude<Locale, "auto"> {
  const locale = navigator.language;
  if (locale.toLowerCase().startsWith("zh")) return "zh-CN";
  if (locale.toLowerCase().startsWith("en")) return "en";
  return FALLBACK_LOCALE;
}

/**
 * 将存储的 locale 解析为实际显示的语言。
 * `"auto"` 会根据系统语言实时解析。
 */
export function resolveLocale(locale: Locale): Exclude<Locale, "auto"> {
  if (locale === "auto") return detectLocale();
  return locale;
}

export function translate(locale: Locale, key: string): string {
  const resolved = resolveLocale(locale);
  return localeMap[resolved][key] ?? localeMap[FALLBACK_LOCALE][key] ?? key;
}

interface I18nContextValue {
  /** 当前选择的 locale（可能是 `auto`）。 */
  locale: Locale;
  /** 当前实际显示的语言（已解析 `auto`）。 */
  resolvedLocale: Exclude<Locale, "auto">;
  setLocale: (locale: Locale) => void;
  t: (key: string) => string;
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
        if (saved === "auto" || saved === "zh-CN" || saved === "en") {
          if (!cancelled) setLocaleState(saved);
        } else {
          if (!cancelled) setLocaleState("auto");
        }
      } catch {
        if (!cancelled) setLocaleState("auto");
      }
    })();
    return () => { cancelled = true; };
  }, []);

  const setLocale = useCallback(async (next: Locale) => {
    setLocaleState(next);
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
          if (next === "auto" || next === "zh-CN" || next === "en") {
            setLocaleState(next);
          }
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
