#!/usr/bin/env python3
"""aukcraft 桌面端主题生成器。

从中性色核心（aukcraft 家族共享，不可修改）+ 一个产品 accent 生成完整的
暗色主题，支持三种输出格式：

  shadcn   shadcn/ui 的 HSL 变量块（直接替换 index.css 里的 .dark）
  css      通用 CSS 自定义属性（任何技术栈可用）
  tailwind tailwind.config 的 theme.extend 片段

脚本自动推算暗色界面使用的 accent 浅阶（400 阶），并对照 base 背景做
WCAG 对比度检查，不足 4.5:1 时在 stderr 给出警告。

用法：
  python generate_theme.py --accent "#2563EB" --format shadcn
"""

import argparse
import colorsys
import re
import sys

# ── aukcraft 中性色核心（共享，勿改）─────────────────────────────
BASE = "#0B0E11"
RAISED = "#14181D"
INK = "#EDEDED"
MUTED = "#8A9199"
HAIRLINE_ALPHA = 0.08
# 第三级表面：悬停 / 轨道 / pill。10% 白叠加在 raised 上，
# 与 raised 保持 ≥3% 明度差——hover 反馈必须在浮层上可见。
HOVER_ALPHA = 0.10
# 槽位塌缩门禁：card 与 hover 系槽位（muted/secondary/accent）的明度差
# 下限。低于此值说明表面层级不够，hover 会"死"在浮层上。
MIN_SURFACE_L_GAP = 0.03

# shadcn 暗色主题的破坏性色（aukcraft 规范不覆盖语义色，沿用组件库默认）
DESTRUCTIVE_HSL = (0, 0.628, 0.506)

# 「不艳即舒适」饱和度门禁：accent 必须保留至少 ~10% 灰度。
# HSV 饱和度 > 0.90 且明度 ≥ 0.5 的颜色（霓虹区：纯色通道色、电光青/品红等）
# 直接拒绝。家族锚点均低于阈值：teal #14B8A6（S=0.89）、blue-600（S=0.84）。
MAX_HSV_SATURATION = 0.90
MIN_VIVID_VALUE = 0.5


def check_saturation(name: str, rgb: tuple[int, int, int]) -> None:
    """拒绝过艳的颜色——这是硬规则，不是警告。"""
    mx, mn = max(rgb), min(rgb)
    if mx == 0:
        return
    s = (mx - mn) / mx
    v = mx / 255
    if s > MAX_HSV_SATURATION and v >= MIN_VIVID_VALUE:
        sys.exit(
            f"错误：{name} {rgb_to_hex(rgb)} 饱和度过高"
            f"（HSV S={s:.2f}，阈值 {MAX_HSV_SATURATION}）——直接拒绝。\n"
            "aukcraft 的色彩纪律是「不艳即舒适」：霓虹感的颜色不进入家族。\n"
            "建议：选同色系更灰或更深的色阶（Tailwind palette 降一阶），"
            "或换一个带灰度的色相。"
        )


def hex_to_rgb(value: str) -> tuple[int, int, int]:
    """把 #RRGGBB 转成 (r, g, b)，非法输入直接报错退出。"""
    m = re.fullmatch(r"#?([0-9a-fA-F]{6})", value.strip())
    if not m:
        sys.exit(f"错误：无法解析颜色 {value!r}，需要 #RRGGBB 形式")
    h = m.group(1)
    return int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16)


def rgb_to_hex(rgb: tuple[int, int, int]) -> str:
    return "#{:02X}{:02X}{:02X}".format(*rgb)


def rgb_to_hls(rgb: tuple[int, int, int]) -> tuple[float, float, float]:
    """返回 (h 0-360, l 0-1, s 0-1)。"""
    r, g, b = (c / 255 for c in rgb)
    h, l, s = colorsys.rgb_to_hls(r, g, b)
    return h * 360, l, s


def hsl_triplet(rgb: tuple[int, int, int]) -> str:
    """shadcn 风格的 'H S% L%' 三元组。"""
    h, l, s = rgb_to_hls(rgb)
    return f"{h:.0f} {s * 100:.1f}% {l * 100:.1f}%"


def soft_accent(accent: tuple[int, int, int]) -> tuple[int, int, int]:
    """推算暗色界面用的 accent 浅阶（约 400 阶）。

    保持色相与饱和度，把亮度拉到 0.62–0.68 区间；accent 本身已经够亮
    （L ≥ 0.6）时原样保留。
    """
    r, g, b = (c / 255 for c in accent)
    h, l, s = colorsys.rgb_to_hls(r, g, b)
    if l < 0.6:
        l = min(max(0.62, l), 0.68)
    r2, g2, b2 = colorsys.hls_to_rgb(h, l, s)
    return round(r2 * 255), round(g2 * 255), round(b2 * 255)


def blend_over_base(alpha: float, surface: tuple[int, int, int]) -> tuple[int, int, int]:
    """白色以 alpha 叠加在 surface 上的不透明近似值（发丝线的实心等效色）。"""
    return tuple(round(s + alpha * (255 - s)) for s in surface)  # type: ignore[return-value]


def relative_luminance(rgb: tuple[int, int, int]) -> float:
    def channel(c: int) -> float:
        c = c / 255
        return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4

    r, g, b = (channel(c) for c in rgb)
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def contrast(a: tuple[int, int, int], b: tuple[int, int, int]) -> float:
    la, lb = relative_luminance(a), relative_luminance(b)
    hi, lo = max(la, lb), min(la, lb)
    return (hi + 0.05) / (lo + 0.05)


def emit_shadcn(accent: tuple[int, int, int], soft: tuple[int, int, int]) -> str:
    base, raised, ink, muted = (hex_to_rgb(c) for c in (BASE, RAISED, INK, MUTED))
    border_on_raised = blend_over_base(HAIRLINE_ALPHA, raised)
    hover = blend_over_base(HOVER_ALPHA, raised)
    # 槽位塌缩门禁：hover 系槽位与 card 同值 = 悬停反馈死亡，拒绝生成
    _, card_l, _ = rgb_to_hls(raised)
    _, hover_l, _ = rgb_to_hls(hover)
    if hover_l - card_l < MIN_SURFACE_L_GAP:
        sys.exit(
            f"错误：hover 表面 {rgb_to_hex(hover)} 与 card {rgb_to_hex(raised)} "
            f"明度差 {hover_l - card_l:.3f} < {MIN_SURFACE_L_GAP}——"
            "槽位塌缩会导致悬停反馈不可见。请提高 HOVER_ALPHA。"
        )
    d_h, d_s, d_l = DESTRUCTIVE_HSL
    return f"""/* aukcraft 桌面端暗色主题 — 由 generate_theme.py 生成
 * 中性色核心原样落地；accent 为产品色，暗色界面统一使用 400 浅阶。
 * 三级表面：base（窗口底） < raised（浮层） < hover（悬停/轨道/pill）。
 * muted/secondary/accent 槽同属 hover 级（与 shadcn 暗色默认同构），
 * 保证浮层上的悬停反馈可见。accent 槽保持中性，不放品牌色。
 * 注意：border/input 是 8% 白发丝线叠加在 raised 上的不透明近似值，
 * 因为 shadcn 的 hsl(var(--border)) 语法不带 alpha 通道。
 * 固定暗色：直接使用 :root，不提供 .light。
 */
@layer base {{
  :root {{
    --background: {hsl_triplet(base)};
    --foreground: {hsl_triplet(ink)};
    --card: {hsl_triplet(raised)};
    --card-foreground: {hsl_triplet(ink)};
    --popover: {hsl_triplet(raised)};
    --popover-foreground: {hsl_triplet(ink)};
    --primary: {hsl_triplet(soft)};
    --primary-foreground: {hsl_triplet(base)};
    --secondary: {hsl_triplet(hover)};
    --secondary-foreground: {hsl_triplet(ink)};
    --muted: {hsl_triplet(hover)};
    --muted-foreground: {hsl_triplet(muted)};
    /* shadcn 的 accent 槽位是悬停底色，保持中性，不放品牌色 */
    --accent: {hsl_triplet(hover)};
    --accent-foreground: {hsl_triplet(ink)};
    --destructive: {d_h:.0f} {d_s * 100:.1f}% {d_l * 100:.1f}%;
    --destructive-foreground: {hsl_triplet(ink)};
    --border: {hsl_triplet(border_on_raised)};
    --input: {hsl_triplet(border_on_raised)};
    --ring: {hsl_triplet(soft)};
    --radius: 4px;

    /* aukcraft 原始 token，组件外直接使用 */
    --color-base: {BASE};
    --color-raised: {RAISED};
    --color-hover: {rgb_to_hex(hover)};
    --color-hairline: rgba(255, 255, 255, 0.08);
    --color-ink: {INK};
    --color-muted: {MUTED};
    --color-accent: {rgb_to_hex(accent)};
    --color-accent-soft: {rgb_to_hex(soft)};
    --ease-lock: cubic-bezier(0.16, 1, 0.3, 1);
  }}
}}"""


def hover_surface() -> tuple[int, int, int]:
    """第三级表面：悬停 / 轨道 / pill（10% 白叠加在 raised 上）。"""
    return blend_over_base(HOVER_ALPHA, hex_to_rgb(RAISED))


def emit_css(accent: tuple[int, int, int], soft: tuple[int, int, int]) -> str:
    return f"""/* aukcraft 桌面端暗色主题 — 由 generate_theme.py 生成（通用 CSS 变量） */
:root {{
  --color-base: {BASE};
  --color-raised: {RAISED};
  --color-hover: {rgb_to_hex(hover_surface())};
  --color-hairline: rgba(255, 255, 255, 0.08);
  --color-ink: {INK};
  --color-muted: {MUTED};
  /* 产品 accent：600 阶用于站点/亮色语境，400 浅阶用于暗色界面 */
  --color-accent: {rgb_to_hex(accent)};
  --color-accent-soft: {rgb_to_hex(soft)};
  --ease-lock: cubic-bezier(0.16, 1, 0.3, 1);
  --radius: 4px;
}}

body {{
  background-color: var(--color-base);
  color: var(--color-ink);
}}

/* 焦点 = accent 浅阶，恒成立 */
:focus-visible {{
  outline: 1px solid var(--color-accent-soft);
  outline-offset: 3px;
}}

::selection {{
  background: color-mix(in srgb, var(--color-accent) 22%, transparent);
}}"""


def emit_tailwind(accent: tuple[int, int, int], soft: tuple[int, int, int]) -> str:
    return f"""// aukcraft 桌面端主题 — 由 generate_theme.py 生成
// 并入 tailwind.config 的 theme.extend
{{
  colors: {{
    base: '{BASE}',
    raised: '{RAISED}',
    hover: '{rgb_to_hex(hover_surface())}',
    ink: '{INK}',
    muted: '{MUTED}',
    hairline: 'rgba(255,255,255,0.08)',
    accent: {{
      DEFAULT: '{rgb_to_hex(soft)}', // 暗色界面统一用 400 浅阶
      deep: '{rgb_to_hex(accent)}',  // 600 阶，仅站点/亮色语境
    }},
  }},
  borderRadius: {{
    DEFAULT: '4px',
    sm: '2px',
    md: '3px',
  }},
  transitionTimingFunction: {{
    lock: 'cubic-bezier(0.16, 1, 0.3, 1)',
  }},
  fontFamily: {{
    mono: ['"JetBrains Mono"', 'ui-monospace', 'monospace'],
  }},
}}"""


def emit_json(accent: tuple[int, int, int], soft: tuple[int, int, int]) -> str:
    """平台无关的 token JSON（供 React Native / Flutter / SwiftUI 等非 CSS 栈消费）。"""
    import json
    return json.dumps({
        "base": BASE,
        "raised": RAISED,
        "hover": rgb_to_hex(hover_surface()),
        "hairline": "rgba(255,255,255,0.08)",
        "ink": INK,
        "muted": MUTED,
        "accent": rgb_to_hex(accent),
        "accentSoft": rgb_to_hex(soft),
        "easeLock": "cubic-bezier(0.16, 1, 0.3, 1)",
        "radius": 4,
    }, indent=2, ensure_ascii=False)


def main() -> None:
    parser = argparse.ArgumentParser(description="生成 aukcraft 桌面端暗色主题")
    parser.add_argument("--accent", required=True,
                        help="产品 accent 色，#RRGGBB（与产品站点一致）")
    parser.add_argument("--accent-soft", default=None,
                        help="暗色界面用的 accent 浅阶，#RRGGBB。产品站点已有浅阶时"
                             "（如 Tailwind blue-400 #60A5FA）应显式传入以对齐；"
                             "省略时按 HSL 自动推算")
    parser.add_argument("--format", choices=["shadcn", "css", "tailwind", "json"],
                        default="css", help="输出格式（默认 css）")
    args = parser.parse_args()

    accent = hex_to_rgb(args.accent)
    soft = hex_to_rgb(args.accent_soft) if args.accent_soft else soft_accent(accent)

    # 饱和度门禁：过艳的颜色没有生成主题的资格
    check_saturation("--accent", accent)
    if args.accent_soft:
        check_saturation("--accent-soft", soft)

    # WCAG 对比度检查：暗色界面上的 accent 浅阶 vs base 背景
    ratio = contrast(soft, hex_to_rgb(BASE))
    if ratio < 4.5:
        print(f"警告：accent 浅阶 {rgb_to_hex(soft)} 在 {BASE} 上对比度 "
              f"{ratio:.2f}:1，不足 4.5:1，请选用更亮的色阶", file=sys.stderr)

    emit = {"shadcn": emit_shadcn, "css": emit_css,
            "tailwind": emit_tailwind, "json": emit_json}[args.format]
    print(emit(accent, soft))


if __name__ == "__main__":
    main()
