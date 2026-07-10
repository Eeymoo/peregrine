import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import ConfigApp from "./ConfigApp";
import SettingsApp from "./SettingsApp";
import { I18nProvider } from "./lib/i18n";
import "./index.css";

const label = getCurrentWebviewWindow().label;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <I18nProvider>
      {label === "settings" ? <SettingsApp /> : <ConfigApp />}
    </I18nProvider>
  </React.StrictMode>
);
