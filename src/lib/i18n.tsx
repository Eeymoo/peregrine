import {
  createContext,
  useContext,
  useEffect,
  useState,
  useCallback,
  type ReactNode,
} from "react";
import { emit, listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import zhCN from "@/i18n/locales/zh-CN.json";
import en from "@/i18n/locales/en.json";
import options from "@/i18n/options.json";

/** 支持的语言。 */
export type Locale = "zh-CN" | "en";

const STORAGE_KEY = "peregrine:locale";
const FALLBACK_LOCALE: Locale = "zh-CN";
const LOCALE_EVENT = "peregrine:locale-changed";

const localeMap: Record<Locale, Record<string, string>> = {
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
export function detectLocale(): Locale {
  const locale = navigator.language;
  if (locale.toLowerCase().startsWith("zh")) return "zh-CN";
  if (locale.toLowerCase().startsWith("en")) return "en";
  return FALLBACK_LOCALE;
}

function getStoredLocale(): Locale | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw === "zh-CN" || raw === "en") return raw;
  } catch {
    // localStorage 不可用（隐私模式等）时回退到检测。
  }
  return null;
}

function storeLocale(locale: Locale): void {
  try {
    localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    // 忽略写入失败。
  }
}

export function translate(locale: Locale, key: string): string {
  return localeMap[locale][key] ?? localeMap[FALLBACK_LOCALE][key] ?? key;
}

interface I18nContextValue {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: string) => string;
}

const I18nContext = createContext<I18nContextValue | null>(null);

interface I18nProviderProps {
  children: ReactNode;
}

/** 国际化上下文提供者，默认从 localStorage 读取，未设置则通过 navigator.language 检测。 */
export function I18nProvider({ children }: I18nProviderProps) {
  const [locale, setLocaleState] = useState<Locale>(() => {
    return getStoredLocale() ?? detectLocale();
  });

  const setLocale = useCallback(async (next: Locale) => {
    if (next === locale) return;
    storeLocale(next);
    setLocaleState(next);
    try {
      await emit(LOCALE_EVENT, { locale: next });
      await invoke("update_locale", {
        locale: next,
        tray: {
          config: translate(next, "tray.config"),
          settings: translate(next, "tray.settings"),
          quit: translate(next, "tray.quit"),
        },
      });
    } catch {
      // Tauri 事件或命令不可用（如浏览器环境）时静默回退。
    }
  }, [locale]);

  // 监听其他窗口触发的语言变更事件，保持多窗口同步。
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const setup = async () => {
      try {
        unlisten = await listen<{ locale: Locale }>(LOCALE_EVENT, (event) => {
          const next = event.payload.locale;
          if (next === "zh-CN" || next === "en") {
            storeLocale(next);
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

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  const value: I18nContextValue = {
    locale,
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
