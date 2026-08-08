import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import ConfigApp from "./ConfigApp";
import SettingsApp from "./SettingsApp";
import { I18nProvider } from "./lib/i18n";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { installGlobalErrorHandler } from "./lib/globalErrorToast";
import { installLogger } from "./lib/actionLog";
import { getConfig } from "./lib/api";
import { initTelemetry, TELEMETRY_PENDING_RESTART_KEY } from "./lib/telemetry";
import "./index.css";

// 安装全局错误兜底：异步错误显示右上角 toast，不白屏。
installGlobalErrorHandler();
// 安装全局日志收集器：实时记录 actions / console / error，供开发者面板查看。
installLogger();

// 遥测开关「稍后重启」标记：应用重启后清除（标记生命周期跨一次重启）。
localStorage.removeItem(TELEMETRY_PENDING_RESTART_KEY);

const label = getCurrentWebviewWindow().label;

async function bootstrap(): Promise<void> {
  // 读取配置决定前端 SDK 是否初始化：
  // telemetry_enabled 为 true 且 DSN 已注入才 init，否则零网络请求。
  // 首次启动（字段缺失）按关闭处理，授权弹窗在 ConfigApp 内处理并即时初始化。
  let telemetryEnabled = false;
  try {
    const config = await getConfig();
    telemetryEnabled = config.settings.telemetry_enabled === true;
  } catch (e) {
    console.error("[telemetry] failed to load config:", e);
  }
  initTelemetry(telemetryEnabled);

  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <ErrorBoundary name={label === "settings" ? "SettingsApp" : "ConfigApp"}>
        <I18nProvider>
          {label === "settings" ? <SettingsApp /> : <ConfigApp />}
        </I18nProvider>
      </ErrorBoundary>
    </React.StrictMode>,
  );
}

void bootstrap();
