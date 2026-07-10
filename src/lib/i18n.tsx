import {
  createContext,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";

/** 支持的语言。 */
export type Locale = "zh-CN" | "en";

const STORAGE_KEY = "peregrine:locale";
const FALLBACK_LOCALE: Locale = "zh-CN";

/** 根据浏览器语言返回最匹配的受支持 locale。 */
export function detectLocale(): Locale {
  const locale = navigator.language;
  if (locale.toLowerCase().startsWith("zh")) return "zh-CN";
  if (locale.toLowerCase().startsWith("en")) return "en";
  return FALLBACK_LOCALE;
}

function getStoredLocale(): Locale | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw === "zh-CN" || raw === "en") return raw;
  } catch {
    // localStorage 不可用（隐私模式等）时回退到检测。
  }
  return null;
}

function storeLocale(locale: Locale): void {
  try {
    localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    // 忽略写入失败。
  }
}

/** 平铺翻译字典，键支持点号路径。 */
const translations: Record<Locale, Record<string, string>> = {
  "zh-CN": {
    "app.title": "Peregrine",
    "app.version": "版本",

    "config.loading": "加载中…",
    "config.style": "类型",
    "config.opacity": "透明度",
    "config.color": "颜色",
    "config.targetWindow": "目标窗口",
    "config.refresh": "刷新",
    "config.none": "（未选择）",
    "config.startOverlay": "开始覆盖",
    "config.stopOverlay": "停止覆盖",

    "settings.title": "设置",
    "settings.description": "更多设置项将在后续版本加入。",
    "settings.language": "语言",
    "settings.about.title": "关于 Peregrine",
    "settings.about.description":
      "Peregrine 是一款桌面辅助贴图（准心 / 覆盖层）工具，主要用途是帮助玩家缓解 3D 眩晕。",
    "settings.about.version": "版本",
    "settings.about.license": "许可",
    "settings.about.repository": "仓库",

    "styles.edge_rect": "贴边矩形",
    "styles.cross": "准星",
    "styles.large_cross": "大准星",
    "styles.corner_dots4": "定位球 4",
    "styles.corner_dots6": "定位球 6",
    "styles.corner_dots8": "定位球 8",
    "styles.ring": "中心环",
    "styles.custom_orb": "自定义定位球",
    "styles.random_orb": "随机球",
    "styles.border_frame": "边框",
    "styles.custom_image": "自定义图片",
    "styles.edge_arrows": "箭头",

    "anchors.top": "顶部",
    "anchors.bottom": "底部",
    "anchors.left": "左侧",
    "anchors.right": "右侧",
    "anchors.center": "居中",

    "ringStyles.solid": "实线",
    "ringStyles.dashed": "虚线",
    "ringStyles.double": "双环",

    "borderStyles.solid": "完整",
    "borderStyles.gap": "四边缺口",

    "fields.width": "宽度",
    "fields.height": "高度",
    "fields.cornerRadius": "圆角",
    "fields.anchor": "贴边",
    "fields.margin": "边距",
    "fields.armLength": "臂长",
    "fields.lineWidth": "线宽",
    "fields.centerGap": "中心间隙",
    "fields.offset": "距边缘距离",
    "fields.radius": "半径",
    "fields.radiusAuto": "半径（0=自动）",
    "fields.lineWidthAuto": "线宽（自动半径时生效）",
    "fields.ringRadiusPct": "半径占屏高比例",
    "fields.ringStyle": "线型",
    "fields.countTop": "上边缘数量",
    "fields.countBottom": "下边缘数量",
    "fields.countLeft": "左边缘数量",
    "fields.countRight": "右边缘数量",
    "fields.perEdgeCount": "每边数量",
    "fields.positionJitter": "位置扰动",
    "fields.radiusMin": "最小半径",
    "fields.radiusMax": "最大半径",
    "fields.barHeight": "矩形条高度",
    "fields.borderStyle": "样式",
    "fields.borderGap": "四边中间留 20% 缺口",
    "fields.arrowSize": "箭头大小",
    "fields.arrowWidth": "宽度（0=等箭头）",
    "fields.tailPerEdge": "分别设置尾巴长度",
    "fields.tailTop": "上尾巴",
    "fields.tailBottom": "下尾巴",
    "fields.tailLeft": "左尾巴",
    "fields.tailRight": "右尾巴",
    "fields.tailLength": "尾巴长度",
    "fields.file": "文件",
    "fields.imagePathPlaceholder": "PNG 文件路径",
    "fields.browse": "浏览…",
    "fields.imageScale": "缩放比例",
    "fields.imageOffsetX": "水平偏移",
    "fields.imageOffsetY": "垂直偏移",
    "fields.enabled": "启用",
    "fields.top": "上",
    "fields.bottom": "下",
    "fields.left": "左",
    "fields.right": "右",

    "preview.placeholder": "请选择 PNG 文件",

    "license.polyform": "PolyForm Noncommercial · 个人免费 · 禁止商业贩卖",
  },
  en: {
    "app.title": "Peregrine",
    "app.version": "Version",

    "config.loading": "Loading…",
    "config.style": "Style",
    "config.opacity": "Opacity",
    "config.color": "Color",
    "config.targetWindow": "Target window",
    "config.refresh": "Refresh",
    "config.none": "(none)",
    "config.startOverlay": "Start overlay",
    "config.stopOverlay": "Stop overlay",

    "settings.title": "Settings",
    "settings.description": "More settings will be added in future versions.",
    "settings.language": "Language",
    "settings.about.title": "About Peregrine",
    "settings.about.description":
      "Peregrine is a desktop overlay / crosshair helper, mainly used to help players reduce motion sickness in 3D games.",
    "settings.about.version": "Version",
    "settings.about.license": "License",
    "settings.about.repository": "Repository",

    "styles.edge_rect": "Edge rect",
    "styles.cross": "Cross",
    "styles.large_cross": "Large cross",
    "styles.corner_dots4": "Corner dots 4",
    "styles.corner_dots6": "Corner dots 6",
    "styles.corner_dots8": "Corner dots 8",
    "styles.ring": "Ring",
    "styles.custom_orb": "Custom orb",
    "styles.random_orb": "Random orb",
    "styles.border_frame": "Border frame",
    "styles.custom_image": "Custom image",
    "styles.edge_arrows": "Edge arrows",

    "anchors.top": "Top",
    "anchors.bottom": "Bottom",
    "anchors.left": "Left",
    "anchors.right": "Right",
    "anchors.center": "Center",

    "ringStyles.solid": "Solid",
    "ringStyles.dashed": "Dashed",
    "ringStyles.double": "Double",

    "borderStyles.solid": "Solid",
    "borderStyles.gap": "Gap",

    "fields.width": "Width",
    "fields.height": "Height",
    "fields.cornerRadius": "Corner radius",
    "fields.anchor": "Anchor",
    "fields.margin": "Margin",
    "fields.armLength": "Arm length",
    "fields.lineWidth": "Line width",
    "fields.centerGap": "Center gap",
    "fields.offset": "Offset from edge",
    "fields.radius": "Radius",
    "fields.radiusAuto": "Radius (0=auto)",
    "fields.lineWidthAuto": "Line width (used when radius is auto)",
    "fields.ringRadiusPct": "Radius as % of screen height",
    "fields.ringStyle": "Line style",
    "fields.countTop": "Top count",
    "fields.countBottom": "Bottom count",
    "fields.countLeft": "Left count",
    "fields.countRight": "Right count",
    "fields.perEdgeCount": "Count per edge",
    "fields.positionJitter": "Position jitter",
    "fields.radiusMin": "Min radius",
    "fields.radiusMax": "Max radius",
    "fields.barHeight": "Bar height",
    "fields.borderStyle": "Style",
    "fields.borderGap": "Leave 20% gap in the middle of each side",
    "fields.arrowSize": "Arrow size",
    "fields.arrowWidth": "Width (0=equal arrow)",
    "fields.tailPerEdge": "Set tail length per edge",
    "fields.tailTop": "Top tail",
    "fields.tailBottom": "Bottom tail",
    "fields.tailLeft": "Left tail",
    "fields.tailRight": "Right tail",
    "fields.tailLength": "Tail length",
    "fields.file": "File",
    "fields.imagePathPlaceholder": "PNG file path",
    "fields.browse": "Browse…",
    "fields.imageScale": "Scale",
    "fields.imageOffsetX": "Horizontal offset",
    "fields.imageOffsetY": "Vertical offset",
    "fields.enabled": "Enabled",
    "fields.top": "T",
    "fields.bottom": "B",
    "fields.left": "L",
    "fields.right": "R",

    "preview.placeholder": "Please select a PNG file",

    "license.polyform": "PolyForm Noncommercial · free for personal use · commercial resale prohibited",
  },
};

export function translate(locale: Locale, key: string): string {
  return translations[locale][key] ?? translations[FALLBACK_LOCALE][key] ?? key;
}

interface I18nContextValue {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: string) => string;
}

const I18nContext = createContext<I18nContextValue | null>(null);

interface I18nProviderProps {
  children: ReactNode;
}

/** 国际化上下文提供者，默认从 localStorage 读取，未设置则通过 navigator.language 检测。 */
export function I18nProvider({ children }: I18nProviderProps) {
  const [locale, setLocaleState] = useState<Locale>(() => {
    return getStoredLocale() ?? detectLocale();
  });

  const setLocale = (next: Locale) => {
    storeLocale(next);
    setLocaleState(next);
  };

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  const value: I18nContextValue = {
    locale,
    setLocale,
    t: (key: string) => translate(locale, key),
  };

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

/** 在 React 组件中使用国际化。 */
export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext);
  if (!ctx) {
    throw new Error("useI18n must be used within <I18nProvider>");
  }
  return ctx;
}
