/**
 * 设置面板截图脚本（docs 截图管线主入口）
 *
 * 前置：根仓库 Vite dev server 运行中（npm run dev -- --port 5199 --strictPort）。
 * 做法：Playwright headless + addInitScript 注入 mock-tauri（windowLabel 区分两个窗口）：
 *       - label 'settings' → SettingsApp：按 Tab 依次点击（通用 / 覆盖层 / 物料 / 快捷键），
 *         每 Tab 等待渲染稳定后截图。Tab 定位使用 Radix 生成的稳定 id 后缀
 *         （`-trigger-<value>` / `-content-<value>`），与界面语言无关。
 *       - label 'config' → ConfigApp：截多图层编辑态（LayersEditor + 真实图元预览）。
 *
 * 产物：docs/public/img/screenshots/ 下（1600×1000 视口截图）：
 *       - settings-general.png  设置窗口·通用 Tab
 *       - settings-overlay.png  设置窗口·覆盖层 Tab
 *       - settings-material.png 设置窗口·物料 Tab
 *       - settings-hotkeys.png  设置窗口·快捷键 Tab
 *       - settings-layers.png   ConfigApp 多图层编辑器（兼容旧引用）
 *       注：主程序 UI 为固定深色主题（不跟随 prefers-color-scheme），故只产单套。
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

// SettingsApp 逐 Tab 截图清单：tab = Radix TabsTrigger 的 value，file = 输出文件名。
const TAB_SHOTS = [
  { tab: 'general', file: 'settings-general.png' },
  { tab: 'overlay', file: 'settings-overlay.png' },
  { tab: 'material', file: 'settings-material.png' },
  { tab: 'hotkeys', file: 'settings-hotkeys.png' },
];

const browser = await chromium.launch();
const results = [];

/** 打开一个注入 mock-tauri 的新上下文并等待配置加载完成。 */
async function openMockPage(windowLabel) {
  const ctx = await browser.newContext({
    viewport: { width: 1600, height: 1000 },
    colorScheme: 'dark',
    deviceScaleFactor: 1,
  });
  await ctx.addInitScript(buildMockInitScript({ windowLabel }));
  const page = await ctx.newPage();
  const errors = [];
  page.on('pageerror', (e) => errors.push('pageerror: ' + e.message));
  page.on('console', (m) => {
    if (m.type() === 'error') errors.push('console: ' + m.text());
  });
  await page.goto(DEV_URL, { waitUntil: 'networkidle' });
  return { ctx, page, errors };
}

// —— SettingsApp：逐 Tab 截图 ——
{
  const { ctx, page, errors } = await openMockPage('settings');
  // 等待设置面板渲染完成（任一 Tab 面板激活即 fixtures 配置已加载）。
  await page.waitForSelector('[role="tab"]', { timeout: 15000 });

  const shots = [];
  for (const { tab, file } of TAB_SHOTS) {
    // Radix TabsTrigger 的 id 形如 `radix-:rn:-trigger-general`，后缀稳定、与语言无关。
    const trigger = page.locator(`[role="tab"][id$="trigger-${tab}"]`);
    if ((await trigger.count()) === 0) {
      shots.push({ tab, ok: false, reason: 'tab trigger not found' });
      continue;
    }
    await trigger.click();
    // 等待 Tab 面板激活（Radix 挂 data-state="active"）再留一拍渲染稳定。
    await page
      .waitForSelector(`[data-state="active"][id$="content-${tab}"]`, { timeout: 5000 })
      .catch(() => {});
    await page.waitForTimeout(800);
    await page.screenshot({ path: path.join(outDir, file) });
    shots.push({ tab, ok: true, file });
  }

  results.push({ window: 'settings', shots, errors: errors.slice(0, 5) });
  await ctx.close();
}

// —— ConfigApp：多图层编辑器截图（既有产物，兼容旧引用） ——
{
  const { ctx, page, errors } = await openMockPage('config');
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

  await page.waitForTimeout(800);
  await page.screenshot({ path: path.join(outDir, 'settings-layers.png') });
  results.push({ window: 'config', canvasOk, errors: errors.slice(0, 5) });
  await ctx.close();
}

await browser.close();
console.log(JSON.stringify(results, null, 2));
const failed = results.some(
  (r) => r.errors.length > 0 || r.shots?.some((s) => !s.ok) || r.canvasOk?.ok === false,
);
console.log(failed ? 'CAPTURE WITH ISSUES' : 'CAPTURE OK');
