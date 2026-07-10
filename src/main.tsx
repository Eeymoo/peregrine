import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import ConfigApp from "./ConfigApp";
import SettingsApp from "./SettingsApp";
import "./index.css";

const label = getCurrentWebviewWindow().label;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {label === "settings" ? <SettingsApp /> : <ConfigApp />}
  </React.StrictMode>
);
