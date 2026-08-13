/**
 * 设置面板截图脚本（docs 截图管线主入口）
 *
 * 前置：根仓库 Vite dev server 运行中（npm run dev -- --port 5199 --strictPort）。
 * 做法：Playwright headless + addInitScript 注入 mock-tauri，加载设置面板，
 *       截取多图层编辑态的设置界面（OverlayTab 含 LayersEditor + 真实图元预览）。
 *
 * 产物：docs/public/img/screenshots/settings-layers.png（1600×1000）
 *       注：主程序 UI 为固定深色主题（不跟随 prefers-color-scheme），故只产单张。
 *
 * 运行：node docs/scripts/capture-screenshots.mjs
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';
import { buildMockInitScript } from './mock-tauri.js';

const DEV_URL = process.env.PEREGRINE_DEV_URL ?? 'http://localhost:5199/';
const outDir = path.join(path.dirname(fileURLToPath(import.meta.url)), '../public/img/screenshots');
fs.mkdirSync(outDir, { recursive: true });

const browser = await chromium.launch();
const results = [];

for (const theme of ['dark']) {
  const ctx = await browser.newContext({
    viewport: { width: 1600, height: 1000 },
    colorScheme: theme,
    deviceScaleFactor: 1,
  });
  await ctx.addInitScript(buildMockInitScript());
  const page = await ctx.newPage();

  const errors = [];
  page.on('pageerror', (e) => errors.push('pageerror: ' + e.message));
  page.on('console', (m) => {
    if (m.type() === 'error') errors.push('console: ' + m.text());
  });

  await page.goto(DEV_URL, { waitUntil: 'networkidle' });

  // 等待多图层编辑器渲染：图层名"中心准星"可见即 fixtures 加载完成。
  await page.waitForSelector('text=中心准星', { timeout: 15000 });

  // 等待预览画布绘制真实图元（canvas 像素非均匀即绘制成功）。
  const canvasOk = await page.evaluate(async () => {
    const canvas = document.querySelector('canvas');
    if (!canvas) return { ok: false, reason: 'no canvas' };
    // 等两帧确保绘制完成。
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    const c = canvas.getContext('2d');
    if (!c || canvas.width === 0) return { ok: false, reason: 'no ctx' };
    const data = c.getImageData(0, 0, canvas.width, canvas.height).data;
    const colors = new Set();
    for (let i = 0; i < data.length; i += 4013 * 4) {
      colors.add(`${data[i]},${data[i + 1]},${data[i + 2]}`);
    }
    return { ok: colors.size > 2, distinctColors: colors.size };
  });

  // 等 UI 稳定后截图。
  await page.waitForTimeout(800);
  const file = path.join(outDir, 'settings-layers.png');
  await page.screenshot({ path: file });

  results.push({ theme, canvasOk, errors: errors.slice(0, 5) });
  await ctx.close();
}

await browser.close();
console.log(JSON.stringify(results, null, 2));
const failed = results.some((r) => !r.canvasOk.ok || r.errors.length > 0);
console.log(failed ? 'CAPTURE WITH ISSUES' : 'CAPTURE OK');
