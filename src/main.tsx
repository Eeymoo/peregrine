import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import ConfigApp from "./ConfigApp";
import SettingsApp from "./SettingsApp";
import { I18nProvider } from "./lib/i18n";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { installGlobalErrorHandler } from "./lib/globalErrorToast";
import "./index.css";

// 安装全局错误兜底：异步错误显示右上角 toast，不白屏。
installGlobalErrorHandler();

const label = getCurrentWebviewWindow().label;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <I18nProvider>
        {label === "settings" ? <SettingsApp /> : <ConfigApp />}
      </I18nProvider>
    </ErrorBoundary>
  </React.StrictMode>,
);
