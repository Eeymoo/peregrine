import { useEffect } from "react";
import { useI18n } from "@/lib/i18n";
import { getCurrentWebviewWindow } from "@/lib/api";

/**
 * 窗口标题实时本地化 hook。
 *
 * 窗口创建时后端已用 `translate(locale, key)` 设置过初始标题（见
 * `src-tauri/src/lib.rs` 的 `create_config_window` / `create_settings_window`，
 * key 为 `window.configTitle` / `window.settingsTitle`）。但创建只在启动时
 * 执行一次，语言切换后不会跟随。本 hook 在前端监听 locale 变化并调用
 * `setTitle` 覆盖，使标题与界面语言实时一致。
 *
 * 传入的 key 应与后端创建时使用的 key 相同，保证两端文案同源
 * （均来自各 locale JSON 的 `window.*` 命名空间）。
 *
 * 非 Tauri 环境（如截图管线 mock）下 `setTitle` 会失败，静默忽略。
 */
export function useWindowTitle(titleKey: string) {
  const { t, resolvedLocale } = useI18n();

  useEffect(() => {
    getCurrentWebviewWindow()
      .setTitle(t(titleKey))
      .catch(() => {});
    // 依赖 resolvedLocale 而非 t：t 的引用在 provider 每次渲染都会变化，
    // 而 resolvedLocale 只在实际语言切换时改变，避免无意义的重复 setTitle。
  }, [titleKey, resolvedLocale]);
}
