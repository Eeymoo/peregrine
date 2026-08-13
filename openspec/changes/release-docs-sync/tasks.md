# Tasks：发版与文档同步固化

## 1. 下载页（en + zh-cn）

- [x] 1.1 新增 `docs/src/components/landing/DownloadTable.astro`：表格组件，props 注入数据（releases 列表 / 当前通道 / 是否显示加速通道）
- [x] 1.2 数据获取：docs 构建时请求 GitHub Releases API，解析 `assets[].browser_download_url`；失败降级为「查看 Releases」外链
- [x] 1.3 通道切换：稳定版 / 预发布版（按 `prerelease` 字段或 tag 含 `-` 判定；无对应 release 时不渲染该通道）
- [x] 1.4 加速/直连切换（仅 zh-cn）：内置 gh-proxy 候选（ghfast.top / gh-proxy.com / ghproxy.net），用户可下拉选择
- [x] 1.5 架构筛选：x64 / x86 / ARM64；表格列：操作系统 / 架构 / 安装包 exe / 便携 zip / 注意事项
- [x] 1.6 「查看更多版本」按钮 → GitHub Releases 页面
- [x] 1.7 新增 `docs/src/content/docs/download.mdx`（en）与 `zh-cn/download.mdx`（zh-cn，`showProxy=true`），文案按 easytier 式结构编写
- [x] 1.8 验证：`npm run build` 成功；API 可用/降级两态均渲染；双语页面路径 `/download` 与 `/zh-cn/download` HTTP 200

## 2. 顶部导航（桌面顶栏 + 移动侧栏）

- [x] 2.1 新增 Header 覆写组件（只插入导航链接区，不动其他内部结构）：AukCraft ↗ / Docs / Download
- [x] 2.2 `astro.config.mjs` 注册 `components: { Header }`；CSS 窄屏隐藏桌面导航（移动端走 sidebar）
- [x] 2.3 sidebar 顶层增加 `link` 项：AukCraft（外链）/ Docs / Download（内链，双语各指向自己的 `/download`）
- [ ] 2.4 更新 AGENTS.md：「仅覆写 Hero」约定改为「仅覆写 Hero + Header」，注明理由与维护注意

## 3. 图标修复 + lucide 统一（用户批准 lucide 系）

- [x] 3.1 `docs/package.json` 新增 devDependency `lucide-static`（lucide 官方纯 SVG 包，零运行时）
- [x] 3.2 删除 `FeatureGrid.astro:55` 的 `set:html={''}`，修复 6 张卡片图标全空 bug
- [x] 3.3 将 FeatureGrid 的 6 枚图标从手写 SVG path 替换为 `lucide-static` 的 path（与主应用 `lucide-react` 同源数据）
- [x] 3.4 更正 `FeatureGrid.astro:6` 错误注释「内联 Phosphor 风格」→「lucide 系 path，与主应用同源」
- [x] 3.5 下载页表格 / 导航链接图标同样从 lucide 取 path，全程不手写 SVG
- [x] 3.6 验证：构建产物中 6 张卡片 svg 均含 lucide path；`npm run build` 成功

## 4. 落地页下载按钮修复

- [x] 4.1 `DownloadCta.astro`：三架构按钮 href 改为站内 `/download`（en）/ `/zh-cn/download`（zh-cn），保留 Releases 外链文案
- [x] 4.2 验证：落地页不再产生指向 404 资产名的链接

## 5. editLink

- [x] 5.1 `astro.config.mjs` 启用 `editLink.baseUrl`，指向 `https://github.com/Eeymoo/peregrine/edit/main/docs/src/content/docs/`
- [x] 5.2 验证：guide 页面出现「Edit this page / 在 GitHub 上编辑此页面」，en/zh-cn 均指向正确源文件

## 6. CI 闸门（L1 fail + L3 job）

- [ ] 6.1 release.yml 新增 `verify-docs` 前置 job：解析 changelog 最新 `## [vX.Y.Z]`，与 tag 版本比对；纯版本号 tag 强制，含 `-` 跳过；不等 → exit 1
- [ ] 6.2 ci.yml 新增 `docs-consistency` job：比对 `package.json` 版本与 changelog 最新条目，不一致 fail（PR 阶段提醒）
- [ ] 6.3 验证：构造不一致样例确认 fail 路径；正常发版路径通过

## 7. 规范与文档同步

- [ ] 7.1 更新 `.agents/skills/release/SKILL.md`：固化「先合 changelog、后打 tag」时序 + CI 闸门说明
- [ ] 7.2 更新 `docs/src/content/docs/guide/development.md`（中英）：发版流程新增 changelog 强制说明与下载页动态版本说明

## 8. 验收

- [ ] 8.1 全量本地构建 `cd docs && npm run build` 通过
- [ ] 8.2 明暗双主题 × 中英文抽查下载页 / 导航 / 落地页，无裸默认样式
- [ ] 8.3 CI 三平台编译测试不受影响（ci.yml 新增 job 独立）
- [ ] 8.4 `docs/scripts/verify-landing.mjs` / `verify-polish.mjs` 对新增元素适配后全 PASS
