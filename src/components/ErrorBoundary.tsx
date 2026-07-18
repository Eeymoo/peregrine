import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

/**
 * 全局错误边界：捕获子树渲染错误，避免白屏。
 *
 * 显示：
 * - 错误消息 + 堆栈
 * - 复制错误按钮（贴给我即可定位）
 * - 重新加载按钮
 * - 打开 DevTools 按钮（如果可用）
 *
 * 不捕获：事件回调、setTimeout、异步错误（用 window.onerror 兜底）。
 */
export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    this.setState({ errorInfo });
    console.error("ErrorBoundary caught:", error, errorInfo);
  }

  handleReload = (): void => {
    window.location.reload();
  };

  handleReset = (): void => {
    this.setState({ hasError: false, error: null, errorInfo: null });
  };

  handleCopy = async (): Promise<void> => {
    const { error, errorInfo } = this.state;
    const text = [
      "Peregrine 前端错误报告",
      "================",
      `时间: ${new Date().toISOString()}`,
      `URL: ${window.location.href}`,
      `UserAgent: ${navigator.userAgent}`,
      "",
      "Error:",
      error?.toString() ?? "(unknown)",
      "",
      "Stack:",
      error?.stack ?? "(no stack)",
      "",
      "Component Stack:",
      errorInfo?.componentStack ?? "(no component stack)",
    ].join("\n");
    try {
      await navigator.clipboard.writeText(text);
      alert("错误已复制到剪贴板，请贴给开发者。");
    } catch {
      // Fallback：打开新窗口让用户手动复制。
      const w = window.open("", "_blank");
      if (w) {
        w.document.write(`<pre>${text.replace(/</g, "&lt;")}</pre>`);
      } else {
        alert("无法访问剪贴板，请手动截图。\n\n" + text);
      }
    }
  };

  handleOpenDevTools = async (): Promise<void> => {
    try {
      const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
      const win = getCurrentWebviewWindow() as unknown as {
        openDevTools?: () => Promise<void>;
      };
      if (typeof win.openDevTools === "function") {
        await win.openDevTools();
      } else {
        alert("此构建未启用 DevTools。");
      }
    } catch (e) {
      alert(`打开 DevTools 失败：${String(e)}`);
    }
  };

  render(): ReactNode {
    if (!this.state.hasError) return this.props.children;

    const { error, errorInfo } = this.state;
    return (
      <div
        style={{
          padding: "32px",
          fontFamily:
            "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
          fontSize: "13px",
          color: "#1a1a1a",
          background: "#fef2f2",
          minHeight: "100vh",
          boxSizing: "border-box",
        }}
      >
        <h1
          style={{
            color: "#b91c1c",
            fontSize: "18px",
            marginTop: 0,
            marginBottom: "16px",
          }}
        >
          ⚠️ Peregrine 遇到了错误
        </h1>

        <p style={{ marginTop: 0, marginBottom: "20px", color: "#7f1d1d" }}>
          应用没有崩溃，但页面渲染失败。可以尝试下面的按钮恢复，或把错误信息复制贴给开发者。
        </p>

        <div style={{ marginBottom: "20px", display: "flex", gap: "8px", flexWrap: "wrap" }}>
          <button
            type="button"
            onClick={this.handleReload}
            style={btnStyle("#2563eb", "white")}
          >
            重新加载页面
          </button>
          <button
            type="button"
            onClick={this.handleReset}
            style={btnStyle("#71717a", "white")}
          >
            尝试恢复（不刷新）
          </button>
          <button
            type="button"
            onClick={this.handleCopy}
            style={btnStyle("#16a34a", "white")}
          >
            复制错误信息
          </button>
          <button
            type="button"
            onClick={this.handleOpenDevTools}
            style={btnStyle("#7c3aed", "white")}
          >
            打开 DevTools
          </button>
        </div>

        <details style={{ marginTop: "8px" }} open>
          <summary style={{ cursor: "pointer", color: "#7f1d1d" }}>
            错误详情（点击折叠）
          </summary>
          <pre
            style={{
              background: "white",
              padding: "12px",
              borderRadius: "4px",
              border: "1px solid #fecaca",
              overflow: "auto",
              maxHeight: "40vh",
              whiteSpace: "pre-wrap",
              wordBreak: "break-all",
            }}
          >
            {error?.toString() ?? "(unknown error)"}
            {"\n\n"}
            {error?.stack ?? "(no stack)"}
            {errorInfo?.componentStack ? `\n\nComponent Stack:${errorInfo.componentStack}` : ""}
          </pre>
        </details>
      </div>
    );
  }
}

function btnStyle(background: string, color: string): React.CSSProperties {
  return {
    background,
    color,
    border: "none",
    padding: "8px 14px",
    borderRadius: "4px",
    cursor: "pointer",
    fontSize: "13px",
    fontWeight: 500,
  };
}
