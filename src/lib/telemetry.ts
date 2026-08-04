import * as Sentry from "@sentry/react";
import { invoke } from "@tauri-apps/api/core";

/**
 * 前端遥测模块：GlitchTip（sentry 协议）匿名崩溃/错误上报出口。
 *
 * - DSN 构建时经 Vite 注入：dev 用 `VITE_GLITCHTIP_DSN_TEST`，正式构建用
 *   `VITE_GLITCHTIP_DSN`；未注入时 SDK 不初始化、零网络请求，且遥测相关 UI 隐藏。
 * - `telemetry_enabled` 为 true 且 DSN 存在才初始化；关闭时错误经 Tauri
 *   command 落盘本地 pending 存储。
 * - beforeSend 脱敏规则与 Rust 侧一致：删 user/server_name/request，
 *   路径用户名替换为 `{user}`。
 */

/** 前端上报 Code 常量（与仓库根 REPORT_CODES.md 同步登记，禁止散落硬编码）。 */
export const REPORT_CODES = {
  /** React ErrorBoundary 捕获的组件渲染错误。 */
  REACT_BOUNDARY: "PGR-3001",
  /** window.onerror 全局未捕获错误。 */
  GLOBAL_ONERROR: "PGR-3002",
  /** unhandledrejection 未处理的 Promise rejection。 */
  UNHANDLED_REJECTION: "PGR-3003",
} as const;

/** 按构建 mode 选择 DSN（开发构建 → TEST 项目，正式构建 → 正式项目）。 */
const DSN: string | undefined = import.meta.env.DEV
  ? import.meta.env.VITE_GLITCHTIP_DSN_TEST
  : import.meta.env.VITE_GLITCHTIP_DSN;

/** DSN 是否已注入（编译期禁用构建不注入 → 遥测 UI 不可用/隐藏）。 */
export const TELEMETRY_DSN_AVAILABLE = typeof DSN === "string" && DSN.length > 0;

/** 遥测开关「稍后重启」待生效标记的 localStorage 键（重启后由 main.tsx 清除）。 */
export const TELEMETRY_PENDING_RESTART_KEY = "peregrine:telemetry-pending-restart";

let initialized = false;

/** 前端 SDK 是否已初始化。 */
export function telemetrySdkActive(): boolean {
  return initialized;
}

/**
 * 脱敏文本：绝对路径用户名替换为 `{user}`（与 Rust 侧 anonymize_text 同规则）。
 */
export function anonymizeText(input: string): string {
  return input
    .replace(/([A-Za-z]:[\\/]Users[\\/])[^\\/\s"']+/g, "$1{user}")
    .replace(/(\/Users\/)[^/\s"']+/g, "$1{user}")
    .replace(/(\/home\/)[^/\s"']+/g, "$1{user}");
}

/**
 * 初始化前端 Sentry SDK。
 *
 * 仅在 `telemetryEnabled` 为 true 且 DSN 已注入时生效；
 * 否则零网络请求（函数幂等，重复调用无副作用）。
 */
export function initTelemetry(telemetryEnabled: boolean): void {
  if (initialized || !telemetryEnabled || !TELEMETRY_DSN_AVAILABLE) return;
  Sentry.init({
    dsn: DSN,
    autoSessionTracking: false,
    beforeSend(event) {
      // 与 Rust before_send 一致的脱敏规则。
      delete event.user;
      delete event.server_name;
      delete event.request;
      if (event.message) event.message = anonymizeText(event.message);
      event.exception?.values?.forEach((ex) => {
        if (ex.value) ex.value = anonymizeText(ex.value);
        ex.stacktrace?.frames?.forEach((frame) => {
          if (frame.filename) frame.filename = anonymizeText(frame.filename);
          if (frame.abs_path) frame.abs_path = anonymizeText(frame.abs_path);
        });
      });
      return event;
    },
  });
  initialized = true;
}

/**
 * 前端错误统一上报出口：
 * - SDK 已初始化：携带 code/event_type/priority（+自定义）tag 上报；
 * - 未初始化（开关关闭/无 DSN）：经 Tauri command 落盘 pending 存储，零网络请求。
 */
export function captureFrontendError(
  code: string,
  error: unknown,
  tags?: Record<string, string>,
): void {
  if (initialized) {
    Sentry.withScope((scope) => {
      scope.setTag("code", code);
      scope.setTag("event_type", "error");
      scope.setTag("priority", "p2");
      for (const [k, v] of Object.entries(tags ?? {})) {
        scope.setTag(k, v);
      }
      if (error instanceof Error) {
        Sentry.captureException(error);
      } else {
        Sentry.captureMessage(anonymizeText(String(error)), "error");
      }
    });
  } else {
    const message =
      error instanceof Error
        ? `${error.message}\n${error.stack ?? ""}`
        : String(error);
    storePendingReport(code, message).catch(() => {
      // 落盘失败静默忽略（best-effort）。
    });
  }
}

/** 前端错误落盘 pending 存储（遥测关闭时）。 */
export async function storePendingReport(code: string, message: string): Promise<void> {
  return invoke("store_pending_report", { code, message });
}

/** 查询本地 pending 历史记录条数。 */
export async function listPendingReports(): Promise<number> {
  return invoke<number>("list_pending_reports");
}

/** 报错页面「匿名上传错误报告」一次性授权：上传当前错误 + 全部历史，返回上传条数。 */
export async function authorizeUploadAll(code: string, message: string): Promise<number> {
  return invoke<number>("authorize_upload_all", { code, message });
}

/** 开发者模式「测试上报」：发送一条 Error 级测试事件。 */
export async function testReport(): Promise<void> {
  return invoke("test_report");
}

/** 重启应用（遥测开关确认弹窗「立即重启」选项）。 */
export async function restartApp(): Promise<void> {
  return invoke("restart_app");
}
