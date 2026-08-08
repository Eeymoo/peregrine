/**
 * 全局操作日志收集器。
 *
 * 实时记录所有 actions 和 log，不依赖开发者功能。
 * 开发者面板的日志查看区域从此处读取。
 *
 * 日志保存在内存中，最多保留 500 条（超出后丢弃最早的）。
 */

export interface LogEntry {
  /** 自增 id。 */
  id: number;
  /** 时间戳（ISO 字符串，精确到毫秒）。 */
  ts: string;
  /** 日志级别。 */
  level: "info" | "warn" | "error";
  /** 来源分类：action（用户操作）/ system（系统事件）/ log（通用日志）。 */
  category: "action" | "system" | "log";
  /** 消息文本。 */
  message: string;
  /** 可选详情（任意可序列化值）。 */
  detail?: unknown;
}

const MAX_ENTRIES = 500;
const entries: LogEntry[] = [];
let nextId = 1;

/** 订阅回调列表。 */
const subscribers: Set<() => void> = new Set();

/** 通知所有订阅者日志已更新。 */
function notify() {
  for (const cb of subscribers) {
    try {
      cb();
    } catch {
      /* 忽略订阅者错误 */
    }
  }
}

/** 记录一条日志。 */
function push(
  level: LogEntry["level"],
  category: LogEntry["category"],
  message: string,
  detail?: unknown,
) {
  const entry: LogEntry = {
    id: nextId++,
    ts: new Date().toISOString().slice(11, 23),
    level,
    category,
    message,
    detail,
  };
  entries.push(entry);
  if (entries.length > MAX_ENTRIES) {
    entries.splice(0, entries.length - MAX_ENTRIES);
  }
  notify();
}

/** 记录一条用户操作日志。 */
export function logAction(message: string, detail?: unknown): void {
  push("info", "action", message, detail);
}

/** 记录一条系统事件日志。 */
export function logSystem(message: string, detail?: unknown): void {
  push("info", "system", message, detail);
}

/** 记录一条通用日志。 */
export function logMessage(message: string, detail?: unknown): void {
  push("info", "log", message, detail);
}

/** 记录一条警告日志。 */
export function logWarn(message: string, detail?: unknown): void {
  push("warn", "log", message, detail);
}

/** 记录一条错误日志。 */
export function logError(message: string, detail?: unknown): void {
  push("error", "log", message, detail);
}

/** 获取当前所有日志条目（只读引用，不要修改）。 */
export function getEntries(): readonly LogEntry[] {
  return entries;
}

/** 清空所有日志。 */
export function clearEntries(): void {
  entries.length = 0;
  notify();
}

/**
 * 订阅日志更新。返回取消订阅函数。
 * 立即触发一次回调，方便订阅者初始化。
 */
export function subscribeEntries(cb: () => void): () => void {
  subscribers.add(cb);
  return () => {
    subscribers.delete(cb);
  };
}

/**
 * 安装全局日志收集器：
 * - 拦截 console.log / console.warn / console.error
 * - 拦截 window.onerror / unhandledrejection
 *
 * 调用一次即可，重复调用无副作用。
 */
let installed = false;
export function installLogger(): void {
  if (installed) return;
  installed = true;

  const origLog = console.log.bind(console);
  const origWarn = console.warn.bind(console);
  const origError = console.error.bind(console);

  console.log = (...args: unknown[]) => {
    origLog(...args);
    const msg = args.map(formatArg).join(" ");
    logMessage(msg);
  };
  console.warn = (...args: unknown[]) => {
    origWarn(...args);
    const msg = args.map(formatArg).join(" ");
    logWarn(msg);
  };
  console.error = (...args: unknown[]) => {
    origError(...args);
    const msg = args.map(formatArg).join(" ");
    logError(msg);
  };

  window.addEventListener("error", (event) => {
    logError(event.message || String(event.error), event.error?.stack);
  });
  window.addEventListener("unhandledrejection", (event) => {
    const reason = event.reason;
    const msg = reason instanceof Error ? reason.message : String(reason);
    logError(`unhandledrejection: ${msg}`, reason instanceof Error ? reason.stack : reason);
  });
}

/** 格式化单个 console 参数为字符串。 */
function formatArg(arg: unknown): string {
  if (arg === null) return "null";
  if (arg === undefined) return "undefined";
  if (typeof arg === "string") return arg;
  if (typeof arg === "number" || typeof arg === "boolean") return String(arg);
  if (arg instanceof Error) return `${arg.message}\n${arg.stack ?? ""}`;
  try {
    return JSON.stringify(arg);
  } catch {
    return String(arg);
  }
}
