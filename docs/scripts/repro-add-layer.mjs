/**
 * 复现脚本：点击图层面板「+」按钮，检测添加图层对话框是否导致 UI 卡死。
 *
 * 做法：mock-tauri 注入 + 打开图层编辑器 + 点击「+」+ 在限时内等待对话框出现。
 * 若页面主线程被长任务阻塞，waitForSelector 会超时 → 判定卡死。
 *
 * 运行：node /tmp/opencode/repro-add-layer.mjs
 */
import { chromium } from 'playwright';
import { buildMockInitScript } from '/_home/Codes/peregrine/docs/scripts/mock-tauri.js';

const DEV_URL = 'http://localhost:5199/';
const browser = await chromium.launch();

const ctx = await browser.newContext({ viewport: { width: 1600, height: 1000 } });
await ctx.addInitScript(buildMockInitScript());
const page = await ctx.newPage();

const errors = [];
const longTasks = [];
page.on('pageerror', (e) => errors.push('pageerror: ' + e.message));
await ctx.addInitScript(() => {
  new PerformanceObserver((list) => {
    list.getEntries().forEach((e) => {
      if (e.duration > 200) window.__longTasks.push(Math.round(e.duration));
    });
  }).observe({ entryTypes: ['longtask'] });
  window.__longTasks = [];
});
page.on('console', (m) => {
  if (m.type() === 'error') errors.push('console: ' + m.text());
});

await page.goto(DEV_URL, { waitUntil: 'networkidle' });
await page.waitForSelector('text=中心准星', { timeout: 15000 });
console.log('[1] 图层编辑器加载 OK');

// 找「+」按钮（LayerPanel 标题栏，title=添加图层）。
const addBtn = page.locator('button[title="添加图层"]');
console.log('[2] 「+」按钮数量:', await addBtn.count());

await addBtn.first().click({ timeout: 5000 });
console.log('[3] 已点击「+」');

// 卡死检测：对话框标题「添加图层」应在 5s 内出现。
try {
  await page.waitForSelector('text=选择物料', { timeout: 5000 }).catch(async () => {
    // 宽松匹配：对话框里任一物料名
    await page.waitForSelector('input[type="radio"]', { timeout: 5000 });
  });
  console.log('[4] 对话框出现 — 未卡死');
} catch {
  console.log('[4] ✗ 对话框 5s 未出现 — 疑似卡死！');
}

// 主线程健康度：evaluate 能否在 2s 内返回。
try {
  const pong = await page.evaluate(() => 'pong', null, { timeout: 2000 });
  console.log('[5] 主线程响应:', pong);
} catch (e) {
  console.log('[5] ✗ 主线程无响应:', e.message.split('\n')[0]);
}

// 再试一次真实交互：选一个物料 + 填名字 + 点「添加」。
try {
  const firstRadio = page.locator('input[type="radio"]').first();
  await firstRadio.click({ timeout: 3000 });
  const nameInput = page.locator('.fixed input[type="text"], .fixed input:not([type])').last();
  await nameInput.fill('测试图层', { timeout: 3000 });
  // 对话框在 fixed 遮罩内，限定范围避免与面板「+」等按钮歧义
  const dialog = page.locator('.fixed.inset-0').last();
  await dialog.getByRole('button', { name: '添加' }).click({ timeout: 3000 });
  await page.waitForTimeout(1000);
  console.log('[6] 添加图层流程完成 — 图层数:', await page.locator('text=测试图层').count());
} catch (e) {
  console.log('[6] ✗ 添加流程失败:', e.message.split('\n')[0]);
}

await page.screenshot({ path: '/tmp/opencode/add-layer-state.png' });
console.log('errors:', errors.length ? errors.slice(0, 5) : '(none)');
console.log('long tasks >200ms:', await page.evaluate(() => window.__longTasks));
await browser.close();
