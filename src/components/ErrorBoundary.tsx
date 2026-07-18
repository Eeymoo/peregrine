import { Component, type ErrorInfo, type ReactNode } from "react";
import { useI18n } from "@/lib/i18n";

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

  render(): ReactNode {
    if (!this.state.hasError) return this.props.children;

    return <ErrorBoundaryFallback error={this.state.error} errorInfo={this.state.errorInfo} onReload={this.handleReload} onReset={this.handleReset} />;
  }
}

interface FallbackProps {
  error: Error | null;
  errorInfo: ErrorInfo | null;
  onReload: () => void;
  onReset: () => void;
}

function ErrorBoundaryFallback({ error, errorInfo, onReload, onReset }: FallbackProps) {
  const { t } = useI18n();

  const handleCopy = async (): Promise<void> => {
    const text = [
      "Peregrine 前端错误报告",
      "================",
      `时间: ${new Date().toISOString()}`,
      `URL: ${window.location.href}`,
      `UserAgent: ${navigator.userAgent}`,
      "",
      "Error:",
      error?.toString() ?? t("error.unknown"),
      "",
      "Stack:",
      error?.stack ?? t("error.noStack"),
      "",
      t("error.componentStack"),
      errorInfo?.componentStack ?? "",
    ].join("\n");
    try {
      await navigator.clipboard.writeText(text);
      alert(t("error.copied"));
    } catch {
      // Fallback：打开新窗口让用户手动复制。
      const w = window.open("", "_blank");
      if (w) {
        w.document.write(`<pre>${text.replace(/</g, "&lt;")}</pre>`);
      } else {
        alert(t("error.copyFallback") + text);
      }
    }
  };

  const handleOpenDevTools = async (): Promise<void> => {
    try {
      const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
      const win = getCurrentWebviewWindow() as unknown as {
        openDevTools?: () => Promise<void>;
      };
      if (typeof win.openDevTools === "function") {
        await win.openDevTools();
      } else {
        alert(t("error.devToolsDisabled"));
      }
    } catch (e) {
      alert(`${t("error.openDevToolsFailed")}: ${String(e)}`);
    }
  };

  return (
    <div className="min-h-screen p-8 bg-destructive/10 text-destructive-foreground dark:text-foreground font-mono text-sm">
      <h1 className="text-lg font-semibold text-destructive mb-4">
        ⚠️ {t("error.title")}
      </h1>

      <p className="mb-5 text-muted-foreground">
        {t("error.description")}
      </p>

      <div className="flex flex-wrap gap-2 mb-5">
        <button type="button" onClick={onReload} className="px-3.5 py-2 rounded bg-primary text-primary-foreground text-xs font-medium">
          {t("error.reload")}
        </button>
        <button type="button" onClick={onReset} className="px-3.5 py-2 rounded bg-muted text-muted-foreground text-xs font-medium">
          {t("error.recover")}
        </button>
        <button type="button" onClick={handleCopy} className="px-3.5 py-2 rounded bg-green-600 text-white text-xs font-medium">
          {t("error.copy")}
        </button>
        <button type="button" onClick={handleOpenDevTools} className="px-3.5 py-2 rounded bg-violet-600 text-white text-xs font-medium">
          {t("error.openDevTools")}
        </button>
      </div>

      <details open>
        <summary className="cursor-pointer text-muted-foreground">
          {t("error.details")}
        </summary>
        <pre className="mt-2 p-3 rounded bg-background border border-border overflow-auto max-h-[40vh] whitespace-pre-wrap break-all text-xs">
          {error?.toString() ?? t("error.unknown")}
          {"\n\n"}
          {error?.stack ?? t("error.noStack")}
          {errorInfo?.componentStack ? `\n\n${t("error.componentStack")}:${errorInfo.componentStack}` : ""}
        </pre>
      </details>
    </div>
  );
}
