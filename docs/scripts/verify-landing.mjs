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
        // HowItWorks 计算样式基线（twcss 迁移断言先行，twcss-migration change）
        hwGrid: cs('.hw-grid', ['display', 'gap', 'gridTemplateColumns', 'marginTop', 'paddingLeft', 'listStyleType']),
        hwStep: cs('.hw-step', ['borderTopWidth', 'borderTopStyle', 'paddingTop']),
        hwIndex: cs('.hw-index', ['fontSize', 'fontWeight', 'letterSpacing', 'fontVariantNumeric']),
        hwIndexColor: cs('.hw-index', ['color'])?.color,
        hwTitle: cs('.hw-title', ['marginTop', 'marginBottom', 'fontSize', 'fontWeight', 'color']),
        hwDetails: cs('.hw-details', ['marginTop', 'fontSize', 'lineHeight', 'color']),
        dlButtons: qa('.dl-button').map((a) => a.textContent.trim() + ' -> ' + a.getAttribute('href')),
        headerNav: qa('.header-nav-link').map((a) => a.getAttribute('href')),
        topbar: !!q('site-title, .site-title'),
        themeToggle: !!q('starlight-theme-select, [data-theme-toggle], starlight-theme-select'),
        // Hero 首屏可见性：CTA 在 900px 视口内
        ctaInViewport: (() => { const a = q('.lh-actions a'); return a && a.getBoundingClientRect().bottom <= 900; })(),
      };
    });
    const t = `${theme}/${name}`;
    check(`${t} Hero 渲染`, r.hero);
    check(`${t} Hero 标题`, !!r.title, r.title);
    check(`${t} 副标题 ≤20 词`, (r.taglineWords ?? 99) <= 20, String(r.taglineWords));
    check(`${t} CTA ≤2 个`, r.ctas.length >= 1 && r.ctas.length <= 2, JSON.stringify(r.ctas));
    check(`${t} CTA 首屏可见`, r.ctaInViewport === true);
    check(`${t} 截图引用本地资产`, r.shotSrc === '/img/screenshots/settings-layers.png', r.shotSrc);
    check(`${t} 截图加载成功`, r.shotLoaded === true);
    check(`${t} 桌面双列 hero 网格`, r.gridCols === 2, String(r.gridCols));
    check(`${t} 特性 6 卡`, r.features === 6, String(r.features));
    check(`${t} 特性图标渲染（lucide）`, r.featureIcons === 6, String(r.featureIcons));
    check(`${t} 三步上手`, r.steps === 3, String(r.steps));
    // HowItWorks 计算样式（twcss 迁移锁，与主题相关的颜色按主题断言）
    const hwIdxExpect = theme === 'dark' ? 'oklch(0.882 0.059 254.128)' : 'oklch(0.546 0.245 262.881)';
    const hwTitleExpect = theme === 'dark' ? 'rgb(255, 255, 255)' : 'oklch(0.21 0.006 285.885)';
    const hwDetailsExpect = theme === 'dark' ? 'oklch(0.871 0.006 286.286)' : 'oklch(0.37 0.013 285.805)';
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
      `${t} hw 步骤发丝线`,
      r.hwStep?.borderTopWidth === '1px' && r.hwStep?.borderTopStyle === 'solid' && r.hwStep?.paddingTop === '20px',
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
      `${t} 顶栏导航 3 链接`,
      r.headerNav.length === 3 && r.headerNav[0] === 'https://www.aukcraft.org/',
      JSON.stringify(r.headerNav),
    );
    check(`${t} 顶栏保留`, r.topbar === true);
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
