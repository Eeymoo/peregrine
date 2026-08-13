/**
 * 当前页面的 locale URL base 获取工具。
 *
 * `Astro.locals.starlightRoute` 是 Starlight 定义的「未就绪即抛错」getter，
 * 在 llms.txt 等插件于路由中间件之外渲染组件时访问会抛 AstroError，
 * 因此必须 try/catch（Starlight 官方建议），并兜底从 URL 路径判断
 *（本站仅 en（root）/ zh-cn 两个 locale）。
 */
import type { AstroGlobal } from 'astro';

/** 返回 'zh-cn' 或 undefined（root locale / 无法判定）。 */
export function localeBase(Astro: AstroGlobal): string | undefined {
  try {
    return Astro.locals.starlightRoute.locale;
  } catch {
    return Astro.url.pathname.startsWith('/zh-cn') ? 'zh-cn' : undefined;
  }
}
