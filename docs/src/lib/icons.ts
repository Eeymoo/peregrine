/**
 * lucide-static 图标取用工具（构建时内联，零运行时依赖）。
 *
 * 与主应用的 lucide-react 共享同一套 path 数据，视觉同源。
 * 返回完整 <svg> 字符串，统一把线宽改为 1.5px（与落地页既有线稿风格一致），
 * 尺寸由调用方 CSS 控制（svg 上的 width/height 属性会被 CSS 覆盖）。
 */
import * as lucide from 'lucide-static';

/** 按 PascalCase 名称取 lucide 图标 SVG 字符串；名称不存在时构建期直接抛错。 */
export function lucideIcon(name: keyof typeof lucide | string): string {
  const svg = (lucide as Record<string, string>)[name];
  if (!svg) {
    throw new Error(`lucide-static 中不存在图标：${name}`);
  }
  return svg.replace('stroke-width="2"', 'stroke-width="1.5"');
}
