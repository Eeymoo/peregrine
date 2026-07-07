---
layout: home

hero:
  name: Peregrine
  text: 桌面辅助贴图工具
  tagline: 在屏幕中心绘制半透明视觉锚点，帮助缓解 3D 眩晕
  image:
    src: /logo.svg
    alt: Peregrine
  actions:
    - theme: brand
      text: 立即下载
      link: https://github.com/eeymoo/peregrine/releases
    - theme: alt
      text: 立即了解
      link: /guide/intro
    - theme: alt
      text: GitHub 仓库
      link: https://github.com/eeymoo/peregrine

features:
  - title: 透明置顶覆盖层
    details: 使用 Win32 分层窗口与每像素 Alpha 实现透明、置顶、鼠标穿透的覆盖层，不影响游戏操作。
  - title: 目标窗口跟随
    details: 自动查找并跟随指定游戏窗口移动，锚点始终保持在正确位置。
  - title: 多种准心样式
    details: 内置卫生纸、准星、大准星、定位球、中心环、边框等多种样式，所见即所得。
  - title: 自定义 PNG 贴图
    details: 支持加载自定义 PNG 图片作为辅助贴图，满足个性化需求。
  - title: 实时设置与预览
    details: 内置 egui 设置面板，调整参数后可立即预览并自动持久化到配置文件。
  - title: 热重载配置
    details: 配置文件修改后自动检测并广播更新，无需重启程序。
---
