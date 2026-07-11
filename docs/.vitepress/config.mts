import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'
import llmstxt from 'vitepress-plugin-llms'

export default withMermaid(defineConfig({
  title: 'Peregrine',
  description: 'Peregrine — 桌面辅助贴图工具，专为缓解 3D 眩晕而设计',
  lang: 'zh-CN',
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
    // 显式置空 PostCSS 配置，阻止 VitePress 继承仓库根目录的 postcss.config.js
    // （根配置引用了 tailwindcss / autoprefixer，在 CI 中 docs/ 下未安装这些依赖，
    // 会导致 "Cannot find module 'tailwindcss'" 构建失败）
    css: {
      postcss: {}
    }
  },

  mermaid: {
    theme: 'default'
  },

  locales: {
    root: {
      label: '简体中文',
      lang: 'zh-CN',
      description: 'Peregrine — 桌面辅助贴图工具，专为缓解 3D 眩晕而设计',
      themeConfig: {
        nav: [
          { text: '首页', link: '/' },
          { text: '使用说明', link: '/guide/usage' },
          { text: '介绍', link: '/guide/intro' },
          { text: '快速开始', link: '/guide/getting-started' },
          { text: '功能', link: '/guide/features' },
          { text: 'GitHub', link: 'https://github.com/eeymoo/peregrine' }
        ],

        sidebar: [
          {
            text: '指南',
            items: [
              { text: '使用说明', link: '/guide/usage' },
              { text: '项目介绍', link: '/guide/intro' },
              { text: '快速开始', link: '/guide/getting-started' },
              { text: '功能特性', link: '/guide/features' },
              { text: '配置说明', link: '/guide/config' },
              { text: '缓解晕 3D', link: '/guide/motion-sickness' },
              { text: '推荐配置', link: '/guide/recommendations' },
              { text: '开发构建', link: '/guide/development' },
              { text: '术语表', link: '/guide/glossary' }
            ]
          }
        ],

        socialLinks: [
          { icon: 'github', link: 'https://github.com/eeymoo/peregrine' }
        ],

        footer: {
          message: '基于 PolyForm Noncommercial 许可发布',
          copyright: 'Copyright © 2025 Peregrine 贡献者'
        }
      }
    },
    en: {
      label: 'English',
      lang: 'en-US',
      link: '/en/',
      description: 'Peregrine — a desktop tool designed to reduce 3D motion sickness',
      themeConfig: {
        nav: [
          { text: 'Home', link: '/en/' },
          { text: 'Usage', link: '/en/guide/usage' },
          { text: 'Intro', link: '/en/guide/intro' },
          { text: 'Getting Started', link: '/en/guide/getting-started' },
          { text: 'Features', link: '/en/guide/features' },
          { text: 'GitHub', link: 'https://github.com/eeymoo/peregrine' }
        ],

        sidebar: [
          {
            text: 'Guide',
            items: [
              { text: 'Usage', link: '/en/guide/usage' },
              { text: 'Introduction', link: '/en/guide/intro' },
              { text: 'Getting Started', link: '/en/guide/getting-started' },
              { text: 'Features', link: '/en/guide/features' },
              { text: 'Configuration', link: '/en/guide/config' },
              { text: 'Motion Sickness Relief', link: '/en/guide/motion-sickness' },
              { text: 'Recommended Settings', link: '/en/guide/recommendations' },
              { text: 'Development', link: '/en/guide/development' },
              { text: 'Glossary', link: '/en/guide/glossary' }
            ]
          }
        ],

        socialLinks: [
          { icon: 'github', link: 'https://github.com/eeymoo/peregrine' }
        ],

        footer: {
          message: 'Released under the PolyForm Noncommercial License',
          copyright: 'Copyright © 2025 Peregrine Contributors'
        }
      }
    }
  }
}))
