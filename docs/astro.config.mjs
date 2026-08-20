// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightLlmsTxt from 'starlight-llms-txt';
import tailwindcss from '@tailwindcss/vite';

/**
 * 显式 sidebar 配置（放弃 autogenerate）
 *
 * 设计决策 D5：sidebar 标签 ≠ 页面 H1，autogenerate 会以 H1 作标签导致退化。
 * 因此逐项写出 16 项，en 为默认 locale 标签，zh-cn 通过 translations 提供中文标签。
 * slug 为 locale 相对路径，root locale 映射到 /guide/*，zh-cn 映射到 /zh-cn/guide/*。
 */
const sidebar = [
  // 顶部导航的移动侧栏镜像（决策 D3）：AukCraft 外链 + Docs / Download 内链，随移动端抽屉展示。
  {
    label: 'AukCraft',
    link: 'https://www.aukcraft.org/',
    attrs: { target: '_blank', rel: 'noopener' },
  },
  { label: 'Docs', translations: { 'zh-CN': '文档' }, slug: 'guide/intro' },
  { label: 'Download', translations: { 'zh-CN': '下载' }, slug: 'download' },
  {
    label: 'Guide',
    translations: { 'zh-CN': '指南' },
    items: [
      { label: 'Usage', translations: { 'zh-CN': '使用说明' }, slug: 'guide/usage' },
      { label: 'Introduction', translations: { 'zh-CN': '项目介绍' }, slug: 'guide/intro' },
      { label: 'Getting Started', translations: { 'zh-CN': '快速开始' }, slug: 'guide/getting-started' },
      { label: 'Features', translations: { 'zh-CN': '功能特性' }, slug: 'guide/features' },
      { label: 'Configuration', translations: { 'zh-CN': '配置说明' }, slug: 'guide/config' },
      { label: 'Settings', translations: { 'zh-CN': '设置详解' }, slug: 'guide/settings' },
      { label: 'Material Scripting', translations: { 'zh-CN': '物料脚本创作' }, slug: 'guide/material-scripting' },
      { label: 'Layers', translations: { 'zh-CN': '图层管理' }, slug: 'guide/layers' },
      { label: 'Motion Sickness Relief', translations: { 'zh-CN': '缓解晕 3D' }, slug: 'guide/motion-sickness' },
      { label: 'Recommended Settings', translations: { 'zh-CN': '推荐配置' }, slug: 'guide/recommendations' },
      { label: 'Privacy & Telemetry', translations: { 'zh-CN': '隐私与遥测' }, slug: 'guide/privacy' },
      { label: 'Development', translations: { 'zh-CN': '开发构建' }, slug: 'guide/development' },
      { label: 'Report Codes', translations: { 'zh-CN': '上报 Code 登记表' }, slug: 'guide/report-codes' },
      { label: 'Contributing', translations: { 'zh-CN': '贡献指南' }, slug: 'guide/contributing' },
      { label: 'Help', translations: { 'zh-CN': '使用帮助' }, slug: 'guide/help' },
      { label: 'Changelog', translations: { 'zh-CN': '更新日志' }, slug: 'guide/changelog' },
      { label: 'Glossary', translations: { 'zh-CN': '术语表' }, slug: 'guide/glossary' },
    ],
  },
];

export default defineConfig({
  site: 'https://peregrine.aukcraft.org',
  base: '/',
  // 与 VitePress cleanUrls: false 一致：产出 .html 文件，内部链接指向干净 URL。
  // 注意：内容集合会剥离 index slug，zh-cn/index.mdx 产出的仍是 zh-cn.html 而非 zh-cn/index.html，
  // 故 /zh-cn/（带斜杠）需由 Cloudflare 重定向规则兜底（见 design.md Open Questions / Risks）。
  trailingSlash: 'never',
  build: {
    format: 'file',
  },
  integrations: [
    starlight({
      title: 'Peregrine',
      description: 'Peregrine — a desktop visual anchor tool designed to reduce 3D motion sickness',
      logo: {
        src: './public/logo.svg',
        alt: 'Peregrine',
        // 图标 + 文字并存（aukcraft 头栏品牌标识形态）
        replacesTitle: false,
      },
      defaultLocale: 'root',
      locales: {
        root: {
          label: 'English',
          lang: 'en',
        },
        'zh-cn': {
          label: '简体中文',
          lang: 'zh-CN',
        },
      },
      sidebar,
      customCss: ['./src/styles/global.css', './src/styles/starlight-polish.css'],
      // 仅覆写 Hero（首页落地页化）与 Header（顶部导航链接，仅插入导航区、其余结构不动）。
      // 不增加其他覆写，降低升级脆弱性；Header 覆写理由与维护注意见 AGENTS.md。
      components: {
        Hero: './src/components/landing/LandingHero.astro',
        Header: './src/components/Header.astro',
      },
      // 「在 GitHub 上编辑此页面」链接（Starlight 自动拼接 src/content/docs 相对路径并适配双语子目录）。
      editLink: {
        baseUrl: 'https://github.com/Eeymoo/peregrine/edit/main/docs/',
      },
      lastUpdated: true,
      favicon: '/img/icons/favicon-32x32.png',
      head: [
        { tag: 'link', attrs: { rel: 'icon', type: 'image/png', sizes: '16x16', href: '/img/icons/favicon-16x16.png' } },
        { tag: 'link', attrs: { rel: 'icon', type: 'image/png', sizes: '48x48', href: '/img/icons/favicon-48x48.png' } },
        { tag: 'link', attrs: { rel: 'apple-touch-icon', sizes: '180x180', href: '/img/icons/apple-touch-icon.png' } },
        { tag: 'link', attrs: { rel: 'icon', type: 'image/png', sizes: '192x192', href: '/img/icons/favicon-192x192.png' } },
      ],
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/eeymoo/peregrine' },
      ],
      plugins: [starlightLlmsTxt()],
    }),
  ],
  vite: {
    plugins: [tailwindcss()],
  },
});
