/* 精修程序化验收：内嵌静态服务器（clean URL → .html），无外部进程依赖 */
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { chromium } from 'playwright';

const DIST = '/home/opencode/Codes/peregrine/docs/dist';
// 校验页选取：config 覆盖表格/代码，usage 覆盖 aside/正文链接。
const PAGES = { table: ['/guide/config', '/zh-cn/guide/config'], aside: ['/guide/usage', '/zh-cn/guide/usage'] };
const MIME = { '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.svg': 'image/svg+xml', '.png': 'image/png', '.json': 'application/json', '.xml': 'text/xml', '.txt': 'text/plain', '.woff2': 'font/woff2' };

const server = http.createServer((req, res) => {
  let p = decodeURIComponent(new URL(req.url, 'http://x').pathname);
  const candidates = [p, p + '.html', path.join(p, 'index.html')];
  for (const c of candidates) {
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
const check = (label, cond, detail) => {
  console.log((cond ? 'PASS' : 'FAIL') + ' ' + label + (detail ? '  | ' + detail : ''));
  if (!cond) fail++;
};

for (const theme of ['dark', 'light']) {
  for (const [name, url] of [['en', PAGES.table[0]], ['zh', PAGES.table[1]]]) {
    const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 }, colorScheme: theme });
    const page = await ctx.newPage();
    await page.goto('http://localhost:4399' + url, { waitUntil: 'networkidle' });
    const r = await page.evaluate(() => {
      const cs = (sel, props) => {
        const el = document.querySelector(sel);
        if (!el) return null;
        const s = getComputedStyle(el);
        return Object.fromEntries(props.map((p) => [p, s[p]]));
      };
      return {
        lineHeight: cs('.sl-markdown-content', ['lineHeight'])?.lineHeight,
        h2Border: cs('.sl-markdown-content h2', ['borderBottomWidth'])?.borderBottomWidth,
        inlineCode: cs('.sl-markdown-content :not(pre) > code', ['borderRadius', 'backgroundColor']),
        thead: cs('.sl-markdown-content thead', ['backgroundColor'])?.backgroundColor,
        tableRadius: cs('.sl-markdown-content table', ['borderRadius'])?.borderRadius,
        aside: cs('.starlight-aside', ['borderRadius', 'borderInlineStartWidth']),
        linkDeco: cs('.sl-markdown-content p a', ['textDecorationLine', 'textUnderlineOffset']),
        bodyText: cs('.sl-markdown-content p', ['color'])?.color,
        pageBg: cs('body', ['backgroundColor'])?.backgroundColor,
        sidebarCurrent: cs('.sidebar-content a[aria-current="page"]', ['borderInlineStartWidth']),
      };
    });
    const t = `${theme}/${name}`;
    check(`${t} 正文行高 1.75`, r.lineHeight?.endsWith('px') && Math.abs(parseFloat(r.lineHeight) / parseFloat(r.bodyTextSize || 16) - 1.75) < 0.3 || r.lineHeight === '28px', r.lineHeight);
    check(`${t} h2 底部发丝线`, r.h2Border === '1px', r.h2Border);
    check(`${t} 行内代码圆角+底色`, r.inlineCode?.borderRadius === '6px' && r.inlineCode?.backgroundColor !== 'rgba(0, 0, 0, 0)' && r.inlineCode?.backgroundColor !== 'transparent', JSON.stringify(r.inlineCode));
    check(`${t} 表头底色`, !!r.thead && r.thead !== 'rgba(0, 0, 0, 0)' && r.thead !== 'transparent', r.thead);
    check(`${t} 表格圆角`, r.tableRadius === '12px', r.tableRadius);
    check(`${t} 侧边栏当前页强调条`, r.sidebarCurrent?.borderInlineStartWidth === '2px', JSON.stringify(r.sidebarCurrent));
    await ctx.close();
  }
  // aside / 正文链接在 usage 页校验（config 页不含这两种元素）。
  for (const [name, url] of [['en', PAGES.aside[0]], ['zh', PAGES.aside[1]]]) {
    const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 }, colorScheme: theme });
    const page = await ctx.newPage();
    await page.goto('http://localhost:4399' + url, { waitUntil: 'networkidle' });
    const r = await page.evaluate(() => {
      const cs = (sel, props) => {
        const el = document.querySelector(sel);
        if (!el) return null;
        const s = getComputedStyle(el);
        return Object.fromEntries(props.map((p) => [p, s[p]]));
      };
      return {
        aside: cs('.starlight-aside', ['borderRadius', 'borderInlineStartWidth']),
        linkDeco: cs('.sl-markdown-content p a, .sl-markdown-content li a', ['textDecorationLine', 'textUnderlineOffset']),
      };
    });
    const t = `${theme}/${name}`;
    check(`${t} aside 圆角+强调条`, r.aside?.borderRadius === '12px' && r.aside?.borderInlineStartWidth === '4px', JSON.stringify(r.aside));
    check(`${t} 链接下划线+偏移`, r.linkDeco?.textDecorationLine?.includes('underline') && r.linkDeco?.textUnderlineOffset !== '0px', JSON.stringify(r.linkDeco));
    // Header 导航 active 态（现代化项 1）：guide 页 Docs 链接应带 aria-current + is-active，下划线常显
    const hnActive = await page.evaluate(() => {
      const docsLink = [...document.querySelectorAll('.header-nav-link')].find((a) => (a.getAttribute('href') || '').includes('/guide/'));
      if (!docsLink) return null;
      const underline = docsLink.querySelector('span[aria-hidden="true"]:last-child');
      return {
        aria: docsLink.getAttribute('aria-current'),
        isActive: docsLink.classList.contains('is-active'),
        color: getComputedStyle(docsLink).color,
        underlineScale: underline ? getComputedStyle(underline).scale : null,
      };
    });
    const expectActiveColor = theme === 'dark' ? 'rgb(255, 255, 255)' : 'oklch(0.21 0.006 285.885)';
    check(
      `${t} 导航 active 态（Docs 当前页）`,
      hnActive?.aria === 'page' &&
        hnActive?.isActive === true &&
        hnActive?.color === expectActiveColor &&
        (hnActive?.underlineScale === '1' || hnActive?.underlineScale === '1 1'),
      JSON.stringify(hnActive),
    );
    await ctx.close();
  }
}
await browser.close();
server.close();
console.log(fail === 0 ? 'ALL PASS' : `${fail} FAILURES`);
process.exit(fail === 0 ? 0 : 1);
