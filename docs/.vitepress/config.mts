import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'
import llmstxt from 'vitepress-plugin-llms'

export default withMermaid(defineConfig({
  title: 'Peregrine',
  description: 'Peregrine — a desktop visual anchor tool designed to reduce 3D motion sickness',
  lang: 'en-US',
  base: '/',
  lastUpdated: true,

  head: [
    ['link', { rel: 'icon', type: 'image/png', sizes: '16x16', href: '/img/icons/favicon-16x16.png' }],
    ['link', { rel: 'icon', type: 'image/png', sizes: '32x32', href: '/img/icons/favicon-32x32.png' }],
    ['link', { rel: 'icon', type: 'image/png', sizes: '48x48', href: '/img/icons/favicon-48x48.png' }],
    ['link', { rel: 'apple-touch-icon', sizes: '180x180', href: '/img/icons/apple-touch-icon.png' }],
    ['link', { rel: 'icon', type: 'image/png', sizes: '192x192', href: '/img/icons/favicon-192x192.png' }]
  ],

  vite: {
    plugins: [llmstxt()],
    // Explicitly empty PostCSS config to prevent VitePress from inheriting the root postcss.config.js
    // (the root config references tailwindcss / autoprefixer, which are not installed under docs/ in CI,
    // causing "Cannot find module 'tailwindcss'" build failures)
    css: {
      postcss: {}
    }
  },

  mermaid: {
    theme: 'default'
  },

  locales: {
    root: {
      label: 'English',
      lang: 'en-US',
      description: 'Peregrine — a desktop visual anchor tool designed to reduce 3D motion sickness',
      themeConfig: {
        nav: [
          { text: 'Home', link: '/' },
          { text: 'Usage', link: '/guide/usage' },
          { text: 'Intro', link: '/guide/intro' },
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Features', link: '/guide/features' },
          { text: 'GitHub', link: 'https://github.com/eeymoo/peregrine' }
        ],

        sidebar: [
          {
            text: 'Guide',
            items: [
              { text: 'Usage', link: '/guide/usage' },
              { text: 'Introduction', link: '/guide/intro' },
              { text: 'Getting Started', link: '/guide/getting-started' },
              { text: 'Features', link: '/guide/features' },
              { text: 'Configuration', link: '/guide/config' },
              { text: 'Motion Sickness Relief', link: '/guide/motion-sickness' },
              { text: 'Recommended Settings', link: '/guide/recommendations' },
              { text: 'Development', link: '/guide/development' },
              { text: 'Changelog', link: '/guide/changelog' },
              { text: 'Glossary', link: '/guide/glossary' }
            ]
          }
        ],

        socialLinks: [
          { icon: 'github', link: 'https://github.com/eeymoo/peregrine' }
        ],

        footer: {
          message: 'Released under the MIT License',
          copyright: 'Copyright © 2025 Peregrine Contributors'
        }
      }
    },
    'zh-cn': {
      label: '简体中文',
      lang: 'zh-CN',
      link: '/zh-cn/',
      description: 'Peregrine — 桌面视觉锚点工具，专为缓解 3D 眩晕而设计',
      themeConfig: {
        nav: [
          { text: '首页', link: '/zh-cn/' },
          { text: '使用说明', link: '/zh-cn/guide/usage' },
          { text: '介绍', link: '/zh-cn/guide/intro' },
          { text: '快速开始', link: '/zh-cn/guide/getting-started' },
          { text: '功能', link: '/zh-cn/guide/features' },
          { text: 'GitHub', link: 'https://github.com/eeymoo/peregrine' }
        ],

        sidebar: [
          {
            text: '指南',
            items: [
              { text: '使用说明', link: '/zh-cn/guide/usage' },
              { text: '项目介绍', link: '/zh-cn/guide/intro' },
              { text: '快速开始', link: '/zh-cn/guide/getting-started' },
              { text: '功能特性', link: '/zh-cn/guide/features' },
              { text: '配置说明', link: '/zh-cn/guide/config' },
              { text: '缓解晕 3D', link: '/zh-cn/guide/motion-sickness' },
              { text: '推荐配置', link: '/zh-cn/guide/recommendations' },
              { text: '开发构建', link: '/zh-cn/guide/development' },
              { text: '更新日志', link: '/zh-cn/guide/changelog' },
              { text: '术语表', link: '/zh-cn/guide/glossary' }
            ]
          }
        ],

        socialLinks: [
          { icon: 'github', link: 'https://github.com/eeymoo/peregrine' }
        ],

        footer: {
          message: '基于 MIT 许可发布',
          copyright: 'Copyright © 2025 Peregrine 贡献者'
        }
      }
    }
  }
}))
