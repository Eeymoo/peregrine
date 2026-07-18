/**
 * 全局错误兜底：捕获 window.onerror 和 unhandledrejection。
 *
 * 把异步错误（事件回调、setTimeout、Promise reject）也显示给用户，
 * 而不是静默吞掉。
 *
 * 用 toast 风格的右上角提示，不打断操作，可点击展开详情。
 */
export function installGlobalErrorHandler(): void {
  if (typeof window === "undefined") return;

  window.addEventListener("error", (event) => {
    console.error("[global error]", event.error ?? event.message);
    showToast(event.message || String(event.error), event.error?.stack);
  });

  window.addEventListener("unhandledrejection", (event) => {
    const reason = event.reason;
    const message =
      reason instanceof Error ? reason.message : typeof reason === "string" ? reason : "(promise rejection)";
    const stack = reason instanceof Error ? reason.stack : undefined;
    console.error("[unhandled rejection]", reason);
    showToast(`Promise 未捕获: ${message}`, stack);
  });
}

interface ToastEntry {
  id: number;
  message: string;
  stack?: string;
}

const toasts: ToastEntry[] = [];
let nextId = 1;

function showToast(message: string, stack?: string): void {
  const id = nextId++;
  toasts.push({ id, message, stack });
  renderToasts();

  // 10 秒后自动消失
  setTimeout(() => {
    const idx = toasts.findIndex((t) => t.id === id);
    if (idx >= 0) {
      toasts.splice(idx, 1);
      renderToasts();
    }
  }, 10000);
}

function renderToasts(): void {
  let container = document.getElementById("__peregrine_error_toasts__");
  if (!container) {
    container = document.createElement("div");
    container.id = "__peregrine_error_toasts__";
    document.body.appendChild(container);
  }

  container.setAttribute(
    "style",
    "position: fixed; top: 12px; right: 12px; z-index: 99999; pointer-events: none; max-width: 420px;",
  );

  container.innerHTML = toasts
    .map(
      (t, i) => `
    <div data-toast-id="${t.id}" style="
      position: relative;
      pointer-events: auto;
      background: #7f1d1d;
      color: white;
      padding: 10px 28px 10px 14px;
      margin-bottom: 8px;
      border-radius: 4px;
      box-shadow: 0 2px 8px rgba(0,0,0,0.2);
      font-family: ui-monospace, monospace;
      font-size: 12px;
      max-width: 400px;
      cursor: pointer;
      opacity: ${1 - i * 0.1};
    ">
      <button data-close-id="${t.id}" style="
        position: absolute; top: 6px; right: 8px;
        background: transparent; color: white; border: none;
        cursor: pointer; font-size: 14px; line-height: 1;
      ">×</button>
      <div style="font-weight: bold; margin-bottom: 4px;">⚠️ ${escapeHtml(t.message).slice(0, 100)}</div>
      ${t.stack ? `<details><summary style="cursor:pointer;opacity:0.8">查看堆栈</summary><pre style="white-space:pre-wrap;font-size:11px;margin-top:4px;max-height:150px;overflow:auto">${escapeHtml(t.stack)}</pre></details>` : ""}
    </div>
  `,
    )
    .join("");

  // 关闭按钮
  container.querySelectorAll("[data-close-id]").forEach((btn) => {
    btn.addEventListener("click", (e) => {
      e.stopPropagation();
      const id = Number((btn as HTMLElement).dataset.closeId);
      const idx = toasts.findIndex((t) => t.id === id);
      if (idx >= 0) toasts.splice(idx, 1);
      renderToasts();
    });
  });
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
