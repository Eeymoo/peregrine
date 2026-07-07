import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'
import llmstxt from 'vitepress-plugin-llms'

export default withMermaid(defineConfig({
  title: 'Peregrine',
  description: 'Peregrine — 桌面辅助贴图工具，专为缓解 3D 眩晕而设计',
  lang: 'zh-CN',
  base: '/peregrine/',
  lastUpdated: true,

  vite: {
    plugins: [llmstxt()]
  },

  mermaid: {
    theme: 'default'
  },

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
          { text: '开发构建', link: '/guide/development' }
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
}))
