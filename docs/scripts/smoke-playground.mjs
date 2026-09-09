// 临时冒烟脚本：验证 /playground 页 CodeMirror 初始化、模板切换、校验与下载按钮存在。
// 用法：node docs/scripts/smoke-playground.mjs（需先 npm run build）
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join, normalize } from 'node:path';
import { chromium } from 'playwright';

const root = join(process.cwd(), 'dist');
const mime = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml', '.json': 'application/json' };
const server = createServer(async (req, res) => {
  try {
    let p = join(root, decodeURIComponent(req.url.split('?')[0]).replace(/\/$/, '/index.html').replace(/^\//, '') || 'index.html');
    if (p.endsWith('/')) p += 'index.html';
    const data = await readFile(normalize(p));
    res.writeHead(200, { 'content-type': mime[extname(p)] ?? 'application/octet-stream' });
    res.end(data);
  } catch {
    res.writeHead(404);
    res.end('not found');
  }
});
await new Promise((r) => server.listen(0, r));
const port = server.address().port;

const browser = await chromium.launch();
const page = await browser.newPage();
const errors = [];
page.on('pageerror', (e) => errors.push(String(e)));
await page.goto(`http://127.0.0.1:${port}/playground.html`);

const hasEditor = await page.locator('.pg-editor .cm-editor').count();
const checks = await page.locator('.pg-checks li').count();
console.log('cm-editor:', hasEditor > 0 ? 'OK' : 'MISSING');
console.log('checks rendered:', checks, checks > 0 ? 'OK' : 'MISSING');

// 模板切换：切到 呼吸圆点，期望 is_dynamic 出现在文档中。
await page.click('.pg-tpl[data-tpl="breath"]');
await page.waitForTimeout(200);
const docHasDyn = await page.locator('.cm-content', { hasText: 'is_dynamic' }).count();
console.log('template switch:', docHasDyn > 0 ? 'OK' : 'FAIL');

// 破坏脚本：删掉 build 函数行，期望错误检查出现。
await page.click('.pg-editor .cm-content');
await page.keyboard.press('Control+A');
await page.keyboard.type('// Name: broken\nfn defaults() { #{} }\nfn schema() { [] }\n', { delay: 5 });
await page.waitForTimeout(200);
const errCount = await page.locator('.pg-checks li', { hasText: 'build' }).count();
console.log('missing build detected:', errCount > 0 ? 'OK' : 'FAIL');

console.log('page errors:', errors.length === 0 ? 'none' : errors.join(' | '));
await browser.close();
server.close();
