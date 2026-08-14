/* 落地页程序化验收：结构 / 主题 / 截图引用 / 布局断点 */
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { chromium } from 'playwright';

const DIST = path.resolve(import.meta.dirname, '../dist');
const MIME = { '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.svg': 'image/svg+xml', '.png': 'image/png', '.xml': 'text/xml', '.txt': 'text/plain', '.woff2': 'font/woff2' };
const server = http.createServer((req, res) => {
  const p = decodeURIComponent(new URL(req.url, 'http://x').pathname);
  for (const c of [p, p + '.html', path.join(p, 'index.html')]) {
    const f = path.join(DIST, c);
    if (fs.existsSync(f) && fs.statSync(f).isFile()) {
      res.writeHead(200, { 'content-type': MIME[path.extname(f)] || 'application/octet-stream' });
      return fs.createReadStream(f).pipe(res);
    }
  }
  res.writeHead(404);
  res.end('nf');
});
await new Promise((r) => server.listen(4399, r));

const browser = await chromium.launch();
let fail = 0;
const check = (label, cond, detail = '') => {
  console.log((cond ? 'PASS' : 'FAIL') + ' ' + label + (detail ? '  | ' + detail : ''));
  if (!cond) fail++;
};

for (const theme of ['dark', 'light']) {
  for (const [name, url] of [['en', '/'], ['zh', '/zh-cn']]) {
    const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 }, colorScheme: theme });
    const page = await ctx.newPage();
    await page.goto('http://localhost:4399' + url, { waitUntil: 'networkidle' });
    const r = await page.evaluate(() => {
      const q = (s) => document.querySelector(s);
      const qa = (s) => [...document.querySelectorAll(s)];
      const cs = (sel, props) => {
        const el = document.querySelector(sel);
        if (!el) return null;
        const s = getComputedStyle(el);
        return Object.fromEntries(props.map((p) => [p, s[p]]));
      };
      const shot = q('.lh-shot');
      return {
        hero: !!q('.landing-hero'),
        eyebrow: q('.lh-eyebrow')?.textContent,
        title: q('.lh-title')?.textContent?.trim(),
        taglineWords: q('.lh-tagline')?.textContent?.trim().split(/\s+/).length,
        ctas: qa('.lh-actions a').map((a) => a.textContent.trim()),
        shotSrc: shot?.getAttribute('src'),
        shotLoaded: shot ? shot.complete && shot.naturalWidth > 0 : false,
        gridCols: getComputedStyle(q('.lh-grid')).gridTemplateColumns.split(' ').length,
        features: qa('.fg-card').length,
        featureIcons: qa('.fg-icon svg.lucide').length,
        steps: qa('.hw-step').length,
        // SectionHeading / FeatureGrid / DownloadCta / 滚入揭示（twcss-migration Section 4）
        sh: qa('.sh-heading').length,
        shIndex: cs('.sh-index', ['fontFamily', 'fontSize', 'letterSpacing', 'textTransform', 'color']),
        shLabel: cs('.sh-label', ['fontFamily', 'textTransform', 'letterSpacing', 'color']),
        shRule: cs('.sh-rule', ['height', 'backgroundColor']),
        fgGrid: cs('.fg-grid', ['display', 'gap', 'backgroundColor', 'gridTemplateColumns']),
        fgCard: cs('.fg-card', ['padding', 'backgroundColor', 'transitionProperty', 'transitionDuration']),
        fgCard5: (() => {
          const c5 = document.querySelectorAll('.fg-card')[4];
          return c5 ? getComputedStyle(c5).gridColumn : null;
        })(),
        fgIcon: cs('.fg-icon', ['color', 'backgroundColor', 'padding', 'borderRadius']),
        dlArch: cs('.dl-arch', ['fontFamily', 'textTransform', 'letterSpacing', 'fontSize']),
        dlButton: cs('.dl-button', ['transitionProperty', 'borderRadius', 'padding']),
        revealInit: (() => {
          const el = document.querySelector('[data-reveal]');
          if (!el) return null;
          const s = getComputedStyle(el);
          return { opacity: s.opacity, transition: s.transitionDuration, revealed: el.classList.contains('is-revealed') };
        })(),
        // LandingHero 迁移断言（Section 5）：animate-rise token + 截图框 hover 浮起
        lhEyebrowAnim: (() => {
          const el = document.querySelector('.lh-eyebrow');
          if (!el) return null;
          const s = getComputedStyle(el);
          return { name: s.animationName, duration: s.animationDuration };
        })(),
        lhVisualAnim: (() => {
          const el = document.querySelector('.lh-visual');
          if (!el) return null;
          const s = getComputedStyle(el);
          return { name: s.animationName, delay: s.animationDelay, duration: s.animationDuration };
        })(),
        lhFrame: cs('.lh-frame', ['borderRadius', 'borderWidth', 'transitionProperty', 'transitionDuration']),
        // HowItWorks 计算样式基线（twcss 迁移断言先行，twcss-migration change）
        hwGrid: cs('.hw-grid', ['display', 'gap', 'gridTemplateColumns', 'marginTop', 'paddingLeft', 'listStyleType']),
        hwStep: cs('.hw-step', ['borderTopWidth', 'borderTopStyle', 'paddingTop']),
        hwIndex: cs('.hw-index', ['fontSize', 'fontWeight', 'letterSpacing', 'fontVariantNumeric']),
        hwIndexColor: cs('.hw-index', ['color'])?.color,
        hwTitle: cs('.hw-title', ['marginTop', 'marginBottom', 'fontSize', 'fontWeight', 'color']),
        hwDetails: cs('.hw-details', ['marginTop', 'fontSize', 'lineHeight', 'color']),
        dlButtons: qa('.dl-button').map((a) => a.textContent.trim() + ' -> ' + a.getAttribute('href')),
        headerNav: qa('.header-nav-link').map((a) => a.getAttribute('href')),
        // Header 导航区计算样式基线（aukcraft 平铺头栏，header-aukcraft-nav）
        headerBox: cs('div.header', ['maxWidth', 'marginLeft']),
        hnNav: cs('.header-nav', ['display', 'alignItems', 'gap']),
        hnLink: cs('.header-nav-link', ['fontFamily', 'fontSize', 'color', 'textDecorationLine', 'whiteSpace']),
        hnLinkAfter: (() => {
          const el = q('.header-nav-link');
          if (!el) return null;
          const s = getComputedStyle(el, '::after');
          return { height: s.height, transform: s.transform, duration: s.transitionDuration };
        })(),
        // 新头栏控件：语言平铺切换 / 主题图标按钮 / 图标化搜索
        langToggle: (() => {
          const el = q('.lang-toggle');
          if (!el) return null;
          return {
            text: el.textContent?.replace(/\s+/g, ''),
            links: [...el.querySelectorAll('a')].map((a) => a.getAttribute('href')),
            current: el.querySelector('[aria-current]')?.textContent?.trim(),
          };
        })(),
        themeToggleBtn: (() => {
          const el = q('[data-theme-toggle]');
          if (!el) return null;
          const sun = el.querySelector('.theme-icon-sun');
          const moon = el.querySelector('.theme-icon-moon');
          return {
            sunDisplay: sun ? getComputedStyle(sun).display : null,
            moonDisplay: moon ? getComputedStyle(moon).display : null,
            ariaLabel: el.getAttribute('aria-label'),
          };
        })(),
        searchIconBtn: (() => {
          const el = q('site-search button[data-open-modal]');
          if (!el) return null;
          const s = getComputedStyle(el);
          const span = el.querySelector('span');
          const kbd = el.querySelector('kbd');
          return {
            borderWidth: s.borderWidth,
            backgroundColor: s.backgroundColor,
            iconVisible: !!el.querySelector('svg'),
            labelHidden: !span || getComputedStyle(span).display === 'none',
            kbdHidden: !kbd || getComputedStyle(kbd).display === 'none',
          };
        })(),
        // 旧控件已移除（下拉主题/语言选择器、social 图标、AukCraft 头栏链接）
        legacyControlsAbsent: !q('starlight-theme-select') && !q('starlight-lang-select') && !q('.social-icons'),
        logoImg: !!q('.site-title img, .site-title svg'),
        topbar: !!q('site-title, .site-title'),
        themeToggle: !!q('starlight-theme-select, [data-theme-toggle], starlight-theme-select'),
        // Hero 首屏可见性：CTA 在 900px 视口内
        ctaInViewport: (() => { const a = q('.lh-actions a'); return a && a.getBoundingClientRect().bottom <= 900; })(),
        // aukcraft 壳层断言（docs-aukcraft-shell，D3/D4/D5/D6/D8/D10）
        dotField: !!q('#dot-field'),
        noise: !!q('.noise'),
        bodyBg: cs('body', ['backgroundColor'])?.backgroundColor,
        emFont: (() => { const e = q('.lh-title em'); return e ? getComputedStyle(e).fontFamily : null; })(),
        dlButtonPrimaryColor: cs('.dl-button', ['color'])?.color,
        // 落地页关键容器圆角 + 阴影采样（D3/D4：≤4px + 零阴影）
        landingRadii: ['.lh-frame', '.dl-button', '.fg-icon', '.lh-actions a'].map((s) => {
          const el = q(s); return el ? { s, r: getComputedStyle(el).borderRadius } : null;
        }),
        landingShadows: ['.lh-frame', '.dl-button', '.fg-card', '.fg-grid', '.sh-rule'].map((s) => {
          const el = q(s); return el ? { s, sh: getComputedStyle(el).boxShadow } : null;
        }),
      };
    });
    const t = `${theme}/${name}`;
    check(`${t} Hero 渲染`, r.hero);
    check(`${t} Hero 标题`, !!r.title, r.title);
    check(`${t} 副标题 ≤20 词`, (r.taglineWords ?? 99) <= 20, String(r.taglineWords));
    check(`${t} CTA ≤2 个`, r.ctas.length >= 1 && r.ctas.length <= 2, JSON.stringify(r.ctas));
    check(`${t} CTA 首屏可见`, r.ctaInViewport === true);
    // LandingHero（Section 5）：animate-rise 生效 + 截图框样式与 hover 浮起
    check(
      `${t} hero 入场渐显（animate-rise）`,
      r.lhEyebrowAnim?.name === 'rise' && r.lhVisualAnim?.name === 'rise' && r.lhVisualAnim?.delay === '0.16s',
      JSON.stringify({ copy: r.lhEyebrowAnim, visual: r.lhVisualAnim }),
    );
    check(
      `${t} 截图框样式`,
      r.lhFrame?.borderRadius === '4px' && r.lhFrame?.borderWidth === '1px',
      JSON.stringify(r.lhFrame),
    );
    const frameHover = await page.locator('.lh-frame').hover().then(() =>
      page.evaluate(() => getComputedStyle(document.querySelector('.lh-frame')).boxShadow),
    );
    check(`${t} 截图框 hover 静态零阴影`, frameHover === 'none', frameHover);
    check(`${t} 截图引用本地资产`, r.shotSrc === '/img/screenshots/settings-layers.png', r.shotSrc);
    check(`${t} 截图加载成功`, r.shotLoaded === true);
    check(`${t} 桌面双列 hero 网格`, r.gridCols === 2, String(r.gridCols));
    check(`${t} 特性 6 卡`, r.features === 6, String(r.features));
    check(`${t} 特性图标渲染（lucide）`, r.featureIcons === 6, String(r.featureIcons));
    check(`${t} 三步上手`, r.steps === 3, String(r.steps));
    // HowItWorks 计算样式（twcss 迁移锁，与主题相关的颜色按主题断言）
    const hwIdxExpect = theme === 'dark' ? 'oklch(0.882 0.059 254.128)' : 'oklch(0.546 0.245 262.881)';
    const hwTitleExpect = theme === 'dark' ? 'rgb(245, 247, 249)' : 'rgb(20, 24, 29)';
    const hwDetailsExpect = theme === 'dark' ? 'rgb(198, 207, 215)' : 'rgb(69, 78, 90)';
    check(
      `${t} hw 网格结构`,
      r.hwGrid?.display === 'grid' &&
        r.hwGrid?.gap === '40px' &&
        r.hwGrid?.gridTemplateColumns.split(' ').length === 3 &&
        r.hwGrid?.marginTop === '0px' &&
        r.hwGrid?.paddingLeft === '0px' &&
        r.hwGrid?.listStyleType === 'none',
      JSON.stringify(r.hwGrid),
    );
    check(
      `${t} hw 步骤无额外线（仅标题后一条）`,
      r.hwStep?.borderTopWidth === '0px' && r.hwStep?.paddingTop === '20px',
      JSON.stringify(r.hwStep),
    );
    check(
      `${t} hw 序号样式`,
      r.hwIndex?.fontSize === '13px' &&
        r.hwIndex?.fontWeight === '600' &&
        r.hwIndex?.letterSpacing === '1.56px' &&
        r.hwIndex?.fontVariantNumeric === 'tabular-nums' &&
        r.hwIndexColor === hwIdxExpect,
      JSON.stringify(r.hwIndex) + ' ' + r.hwIndexColor,
    );
    check(
      `${t} hw 标题样式`,
      r.hwTitle?.marginTop === '8px' &&
        r.hwTitle?.marginBottom === '6px' &&
        r.hwTitle?.fontSize === '18px' &&
        r.hwTitle?.fontWeight === '600' &&
        r.hwTitle?.color === hwTitleExpect,
      JSON.stringify(r.hwTitle),
    );
    // Section 4 断言：SectionHeading / hairline 网格 / CTA mono 微标签 / 滚入揭示
    const accentText = theme === 'dark' ? 'oklch(0.882 0.059 254.128)' : 'oklch(0.546 0.245 262.881)';
    const headingText = theme === 'dark' ? 'rgb(245, 247, 249)' : 'rgb(20, 24, 29)';
    check(`${t} SectionHeading ×3`, r.sh === 3, String(r.sh));
    check(
      `${t} SectionHeading 序号 mono 微标签`,
      r.shIndex?.fontFamily?.includes('mono') && r.shIndex?.textTransform === 'uppercase' && r.shIndex?.letterSpacing === '1.95px' && r.shIndex?.color === accentText,
      JSON.stringify(r.shIndex),
    );
    check(
      `${t} SectionHeading 标签 + hairline 贯穿线`,
      r.shLabel?.fontFamily?.includes('mono') && r.shLabel?.textTransform === 'uppercase' && r.shLabel?.color === headingText && r.shRule?.height === '1px',
      JSON.stringify({ shLabel: r.shLabel, shRule: r.shRule }),
    );
    check(
      `${t} 特性卡 hairline 分隔网格`,
      r.fgGrid?.display === 'grid' && r.fgGrid?.gap === '1px' && r.fgGrid?.gridTemplateColumns.split(' ').length === 3 && r.fgGrid?.backgroundColor !== 'rgba(0, 0, 0, 0)',
      JSON.stringify(r.fgGrid),
    );
    check(
      `${t} 特性卡 hover 反馈 + 矩形网格`,
      r.fgCard?.padding === '22px' && r.fgCard?.transitionProperty?.includes('background-color') && r.fgCard5 === 'auto',
      JSON.stringify({ fgCard: r.fgCard, fgCard5: r.fgCard5 }),
    );
    check(
      `${t} 特性图标品牌色淡染`,
      r.fgIcon?.color === accentText && r.fgIcon?.backgroundColor?.includes('0.12') && r.fgIcon?.padding === '8px',
      JSON.stringify(r.fgIcon),
    );
    check(
      `${t} CTA mono 微标签 + 方形描边`,
      r.dlArch?.fontFamily?.includes('mono') && r.dlArch?.textTransform === 'uppercase' && r.dlButton?.borderRadius === '4px',
      JSON.stringify({ dlArch: r.dlArch, dlButton: r.dlButton }),
    );
    check(
      `${t} 滚入揭示（html.js 门控）`,
      r.revealInit && (r.revealInit.revealed === true || (r.revealInit.opacity === '0' && r.revealInit.transition === '0.6s')),
      JSON.stringify(r.revealInit),
    );
    check(
      `${t} hw 详情样式`,
      r.hwDetails?.marginTop === '0px' &&
        r.hwDetails?.fontSize === '14px' &&
        r.hwDetails?.lineHeight === '23.1px' &&
        r.hwDetails?.color === hwDetailsExpect,
      JSON.stringify(r.hwDetails),
    );
    check(`${t} 下载三架构`, r.dlButtons.length === 3, JSON.stringify(r.dlButtons));
    check(
      `${t} 下载按钮指向站内下载页`,
      r.dlButtons.every((b) => b.endsWith(name === 'zh' ? '/zh-cn/download' : '/download')),
      JSON.stringify(r.dlButtons),
    );
    check(
      `${t} 顶栏导航 3 链接（Docs/Download/GitHub）`,
      r.headerNav.length === 3 &&
        r.headerNav[0].endsWith('/guide/intro') &&
        r.headerNav[1].endsWith('/download') &&
        r.headerNav[2] === 'https://github.com/eeymoo/peregrine',
      JSON.stringify(r.headerNav),
    );
    check(`${t} 顶栏保留`, r.topbar === true);
    check(`${t} 品牌标识图标+文字`, r.logoImg === true);
    check(`${t} 旧下拉控件已移除`, r.legacyControlsAbsent === true);
    // aukcraft 壳层断言（docs-aukcraft-shell）
    check(`${t} DotField 点阵背景`, r.dotField === true);
    check(`${t} 胶片颗粒 overlay`, r.noise === true);
    const bgExpect = theme === 'dark' ? 'rgb(11, 14, 17)' : 'rgb(245, 247, 249)';
    check(`${t} 页面底色蓝调近黑/近白`, r.bodyBg === bgExpect, r.bodyBg);
    check(
      `${t} Hero serif-italic 强调`,
      !!r.emFont && (name === 'zh' ? /Songti|Noto Serif CJK|Source Han Serif/.test(r.emFont) : r.emFont.includes('Newsreader')),
      r.emFont,
    );
    check(
      `${t} 落地页容器圆角 ≤4px`,
      r.landingRadii?.every((x) => x && parseFloat(x.r) <= 4),
      JSON.stringify(r.landingRadii),
    );
    check(
      `${t} 落地页容器零阴影`,
      r.landingShadows?.every((x) => !x || x.sh === 'none'),
      JSON.stringify(r.landingShadows),
    );
    if (theme === 'dark') {
      // 暗色 CTA primary 文字对比度 ≥4.5:1（D6：accent-400 亮阶）。
      const lum = (rgb) => {
        const m = rgb.match(/\d+/g);
        if (!m) return 0;
        const [R, G, B] = m.map((n) => {
          const s = n / 255;
          return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
        });
        return 0.2126 * R + 0.7152 * G + 0.0722 * B;
      };
      const fg = lum(r.dlButtonPrimaryColor);
      const bg = lum(r.bodyBg);
      const contrast = (Math.max(fg, bg) + 0.05) / (Math.min(fg, bg) + 0.05);
      check(`${t} 暗色 CTA primary 对比度 ≥4.5:1`, contrast >= 4.5, contrast.toFixed(2));
    }
    // Header 导航区计算样式（aukcraft 平铺头栏 + .link-line 统一基元）
    const hnLinkColor = theme === 'dark' ? 'oklch(0.882 0.059 254.128)' : 'oklch(0.546 0.245 262.881)';
    check(
      `${t} 头栏容器 max-w-5xl 居中`,
      r.headerBox?.maxWidth === '1024px' && parseFloat(r.headerBox?.marginLeft) > 0,
      JSON.stringify(r.headerBox),
    );
    check(
      `${t} 导航容器（右侧平铺）`,
      r.hnNav?.display === 'flex' && r.hnNav?.alignItems === 'baseline' && r.hnNav?.gap === '24px',
      JSON.stringify(r.hnNav),
    );
    check(
      `${t} 导航链接（link-line 基元：mono 12px 品牌蓝）`,
      r.hnLink?.fontFamily?.includes('mono') &&
        r.hnLink?.fontSize === '12px' &&
        r.hnLink?.color === hnLinkColor &&
        r.hnLink?.textDecorationLine === 'none' &&
        r.hnLink?.whiteSpace === 'nowrap',
      JSON.stringify(r.hnLink),
    );
    // 语言平铺切换：EN/中文，当前语言 aria-current，另一语言为同页互转链接
    check(
      `${t} 语言平铺切换 EN/中文`,
      r.langToggle &&
        r.langToggle.text === 'EN/中文' &&
        r.langToggle.links.length === 1 &&
        (name === 'zh' ? r.langToggle.current === '中文' : r.langToggle.current === 'EN'),
      JSON.stringify(r.langToggle),
    );
    // 主题图标按钮：dark 下显示 sun、light 下显示 moon
    check(
      `${t} 主题图标按钮（平铺切换）`,
      r.themeToggleBtn &&
        !!r.themeToggleBtn.ariaLabel &&
        (theme === 'dark'
          ? r.themeToggleBtn.sunDisplay !== 'none' && r.themeToggleBtn.moonDisplay === 'none'
          : r.themeToggleBtn.moonDisplay !== 'none' && r.themeToggleBtn.sunDisplay === 'none'),
      JSON.stringify(r.themeToggleBtn),
    );
    // 搜索图标按钮化：无边框无底色，文字标签与 kbd 隐藏，仅保留图标
    check(
      `${t} 搜索图标按钮（与头栏同效）`,
      r.searchIconBtn &&
        r.searchIconBtn.borderWidth === '0px' &&
        (r.searchIconBtn.backgroundColor === 'rgba(0, 0, 0, 0)' || r.searchIconBtn.backgroundColor === 'transparent') &&
        r.searchIconBtn.iconVisible === true &&
        r.searchIconBtn.labelHidden === true &&
        r.searchIconBtn.kbdHidden === true,
      JSON.stringify(r.searchIconBtn),
    );
    // 首页无 active 链接（落地页不在导航项内）
    const hnNoActive = await page.evaluate(() =>
      [...document.querySelectorAll('.header-nav-link')].every((a) => a.getAttribute('aria-current') !== 'page'),
    );
    check(`${t} 导航首页无 active 态`, hnNoActive === true);
    // link-line 下划线：基态 scaleX(0)、300ms 过渡，hover 后生长为 scaleX(1)
    check(
      `${t} link-line 下划线基态`,
      r.hnLinkAfter?.height === '1px' && r.hnLinkAfter?.duration === '0.3s' && /matrix\(0, 0, 0, 1/.test(r.hnLinkAfter?.transform ?? ''),
      JSON.stringify(r.hnLinkAfter),
    );
    await page.hover('.header-nav-link');
    await page.waitForTimeout(400);
    const hnHoverScale = await page.evaluate(
      () => getComputedStyle(document.querySelector('.header-nav-link'), '::after').transform,
    );
    check(`${t} link-line hover 下划线生长`, /matrix\(1, 0, 0, 1/.test(hnHoverScale ?? ''), hnHoverScale);
    // focus-visible 焦点环（规则级断言）：外链 CSS 无 CORS 头时 cssRules 为空，故通过
    // page 上下文 fetch 同源 CSS 文本做规则存在性断言；另验证元素 class 与规则选择器对应。
    const focusRule = await page.evaluate(async () => {
      const hrefs = [...document.querySelectorAll('link[rel="stylesheet"]')].map((l) => l.href);
      let css = '';
      for (const h of hrefs) css += (await (await fetch(h)).text()) + '\n';
      const el = document.querySelector('.header-nav-link');
      const cl = el?.className ?? '';
      return {
        hasWidth: css.includes('.focus-visible\\:outline-2:focus-visible'),
        hasOffset: css.includes('.focus-visible\\:outline-offset-4:focus-visible'),
        hasAccentLight: css.includes('.focus-visible\\:outline-accent-600:focus-visible'),
        hasAccentDark: css.includes('.dark\\:focus-visible\\:outline-accent-200:where([data-theme=dark],[data-theme=dark] *):focus-visible'),
        elHasClasses: ['focus-visible:outline-2', 'focus-visible:outline-offset-4', 'focus-visible:outline-accent-600', 'dark:focus-visible:outline-accent-200'].every((c) => cl.includes(c)),
      };
    });
    check(
      `${t} 导航 focus-visible 焦点环`,
      focusRule.hasWidth && focusRule.hasOffset && focusRule.hasAccentLight && focusRule.hasAccentDark && focusRule.elHasClasses,
      JSON.stringify(focusRule),
    );
    await ctx.close();
  }
}

// 移动视口检查
const ctx = await browser.newContext({ viewport: { width: 390, height: 844 }, colorScheme: 'dark' });
const page = await ctx.newPage();
await page.goto('http://localhost:4399/', { waitUntil: 'networkidle' });
const m = await page.evaluate(() => {
  const q = (s) => document.querySelector(s);
  return {
    gridCols: getComputedStyle(q('.lh-grid')).gridTemplateColumns.split(' ').length,
    fgCols: getComputedStyle(q('.fg-grid')).gridTemplateColumns.split(' ').length,
    overflowX: document.documentElement.scrollWidth <= 391,
  };
});
check('mobile hero 单列', m.gridCols === 1, String(m.gridCols));
check('mobile 特性单列', m.fgCols === 1, String(m.fgCols));
check('mobile 无横向溢出', m.overflowX === true);
await ctx.close();

// 下载页（release-docs-sync 新增）：通道/筛选/加速（zh-cn）/查看更多/降级或数据两态。
for (const theme of ['dark', 'light']) {
  for (const [name, url, expectProxy] of [
    ['en', '/download', false],
    ['zh', '/zh-cn/download', true],
  ]) {
    const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 }, colorScheme: theme });
    const page = await ctx.newPage();
    await page.goto('http://localhost:4399' + url, { waitUntil: 'networkidle' });
    const r = await page.evaluate(() => {
      const q = (s) => document.querySelector(s);
      const qa = (s) => [...document.querySelectorAll(s)];
      const visible = (el) => el && getComputedStyle(el).display !== 'none';
      return {
        hasTable: !!q('.download-table'),
        fallback: !!q('.dt-fallback'),
        channels: qa('.dt-channel').length,
        tabs: qa('.dt-tab').length,
        filters: qa('.dt-filter').length,
        proxy: !!q('.dt-proxy-select'),
        proxyOptions: qa('.dt-proxy-select option').length,
        dlLinks: qa('.dt-dl').length,
        visibleRows: qa('.dt-channel tbody tr').filter(visible).length,
        more: q('.dt-more-link')?.getAttribute('href'),
        // DownloadTable 迁移断言（Section 6）
        dtTab: (() => {
          const el = q('.dt-tab');
          if (!el) return null;
          const s = getComputedStyle(el);
          return { borderRadius: s.borderRadius, fontSize: s.fontSize, transitionProperty: s.transitionProperty };
        })(),
        dtTabActive: (() => {
          const el = q('.dt-tab.is-active');
          if (!el) return null;
          const s = getComputedStyle(el);
          return { backgroundColor: s.backgroundColor, ariaPressed: el.getAttribute('aria-pressed') };
        })(),
        dtDl: (() => {
          const el = q('.dt-dl');
          if (!el) return null;
          const s = getComputedStyle(el);
          return { color: s.color, fontWeight: s.fontWeight, textDecorationLine: s.textDecorationLine };
        })(),
        dtMoreLink: (() => {
          const el = q('.dt-more-link');
          if (!el) return null;
          const s = getComputedStyle(el);
          return { borderRadius: s.borderRadius, fontWeight: s.fontWeight };
        })(),
        overflowX: document.documentElement.scrollWidth <= 1440,
      };
    });
    const t = `${theme}/${name}`;
    check(`${t} 下载页渲染`, r.hasTable);
    if (!r.fallback) {
      // API 可用态：通道/筛选/下载链接齐全（降级态跳过这些断言）。
      check(`${t} 通道切换 ≥1`, r.tabs >= 1, String(r.tabs));
      check(`${t} 架构筛选 4 项`, r.filters === 4, String(r.filters));
      check(`${t} 下载链接存在`, r.dlLinks > 0, String(r.dlLinks));
      check(`${t} 可见行 ≥3`, r.visibleRows >= 3, String(r.visibleRows));
      check(`${t} 查看更多版本 → Releases`, r.more === 'https://github.com/Eeymoo/peregrine/releases', r.more);
      // DownloadTable 迁移断言（Section 6）：方形描边形态 / aria-pressed active 态 / 链接样式
      const accentDl = theme === 'dark' ? 'oklch(0.882 0.059 254.128)' : 'oklch(0.546 0.245 262.881)';
      check(
        `${t} 通道/筛选按钮方形描边`,
        r.dtTab && r.dtTab.borderRadius === '4px' && r.dtTab.fontSize === '14px' && r.dtTab.transitionProperty?.includes('border-color'),
        JSON.stringify(r.dtTab),
      );
      check(
        `${t} active 态（aria-pressed + 背景淡染）`,
        r.dtTabActive?.ariaPressed === 'true' && r.dtTabActive?.backgroundColor?.includes('0.16'),
        JSON.stringify(r.dtTabActive),
      );
      check(
        `${t} 下载链接样式`,
        r.dtDl?.color === accentDl && r.dtDl?.fontWeight === '500' && r.dtDl?.textDecorationLine === 'none',
        JSON.stringify(r.dtDl),
      );
      check(
        `${t} 查看更多按钮方形描边`,
        r.dtMoreLink && r.dtMoreLink.borderRadius === '4px' && r.dtMoreLink.fontWeight === '600',
        JSON.stringify(r.dtMoreLink),
      );
    }
    check(`${t} 加速通道${expectProxy ? '存在' : '不渲染'}`, r.proxy === expectProxy);
    if (expectProxy && r.proxy) {
      check(`${t} 加速候选 ≥4 项`, r.proxyOptions >= 4, String(r.proxyOptions));
      // 架构筛选交互：点击 ARM64 后当前通道仅 1 行可见；加速选择重写下载链接 href。
      const inter = await page.evaluate(() => {
        const q = (s) => document.querySelector(s);
        const qa = (s) => [...document.querySelectorAll(s)];
        const visible = (el) => el && getComputedStyle(el).display !== 'none';
        const out = {};
        const btn = qa('.dt-filter').find((b) => b.dataset.arch === 'arm64');
        if (btn) {
          btn.click();
          out.rows = qa('.dt-channel:not(.is-hidden) tbody tr').filter(visible).length;
          qa('.dt-filter').find((b) => b.dataset.arch === 'all')?.click();
        }
        const sel = q('.dt-proxy-select');
        if (sel && q('.dt-dl')) {
          sel.selectedIndex = 1;
          sel.dispatchEvent(new Event('change'));
          out.proxied = q('.dt-dl').getAttribute('href');
          sel.selectedIndex = 0;
          sel.dispatchEvent(new Event('change'));
          out.restored = q('.dt-dl').getAttribute('href');
        }
        return out;
      });
      if (inter.rows !== undefined) check(`${t} 架构筛选交互`, inter.rows === 1, String(inter.rows));
      if (inter.proxied) {
        check(`${t} 加速重写 href`, inter.proxied.startsWith('https://ghfast.top/https://github.com/'), inter.proxied);
        check(`${t} 直连恢复 href`, inter.restored.startsWith('https://github.com/'), inter.restored);
      }
    }
    await ctx.close();
  }
}

await browser.close();
server.close();
console.log(fail === 0 ? 'ALL PASS' : `${fail} FAILURES`);
process.exit(fail === 0 ? 0 : 1);
