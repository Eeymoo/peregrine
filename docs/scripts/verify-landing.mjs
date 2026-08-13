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
        steps: qa('.hw-step').length,
        dlButtons: qa('.dl-button').map((a) => a.textContent.trim() + ' -> ' + a.getAttribute('href')),
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
    check(`${t} 三步上手`, r.steps === 3, String(r.steps));
    check(`${t} 下载三架构`, r.dlButtons.length === 3, JSON.stringify(r.dlButtons));
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

await browser.close();
server.close();
console.log(fail === 0 ? 'ALL PASS' : `${fail} FAILURES`);
process.exit(fail === 0 ? 0 : 1);
