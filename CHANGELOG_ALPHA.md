# 测试版更新日志

以下版本为正式版发布前的测试与开发迭代，仅供预览与反馈。

正式版日志见 **[CHANGELOG.md](CHANGELOG.md)**。

---

## [v0.2.0-alpha.0] — 2026-07-06

### 新增

- PNG 图片支持：可加载自定义 PNG 作为覆盖层贴图。
- 预览与覆盖层统一几何模块，减少逻辑重复。

### 变更

- 覆盖层改用 softbuffer 像素缓冲区方案（参考 simple-crosshair-overlay）。
- 设置 UI 与覆盖层渲染共享几何绘制逻辑。

---

## [v0.1.0-alpha.12] — 2026-07-02

- 架构重构为双窗口：设置窗口与独立 Overlay 窗口分离。

## [v0.1.0-alpha.11] — 2026-07-02

- 移除所有非 Windows 平台代码，项目聚焦 Windows。

## [v0.1.0-alpha.10] — 2026-07-02

- 修复透明度彻底失效：强制 Bgra8Unorm 避免 sRGB gamma 导致颜色键不匹配。

## [v0.1.0-alpha.9] — 2026-07-02

- 修复日志默认不输出：EnvFilter 改为默认 info 级别。

## [v0.1.0-alpha.8] — 2026-07-02

- 修复 HWND 跨线程获取失败。
- 新增未选窗口防护。
- 修复窗口尺寸恢复与清理冗余。

## [v0.1.0-alpha.7] — 2026-07-02

- 修复颜色键吃黑色准心。
- 修复覆盖层切换闪烁。
- 修复窗口标题匹配逻辑。

## [v0.1.0-alpha.6] — 2026-07-02

- 新增"开始覆盖"按钮。
- 修复透明颜色键。
- 添加选择窗口日志，清理调试打印。

## [v0.1.0-alpha.5] — 2026-07-02

- 编译优化。
- 嵌入 Windows exe 图标。

## [v0.1.0-alpha.4] — 2026-07-02

- Windows 覆盖层保留 Bgra8UnormSrgb 以修复 DWM 透明合成。

## [v0.1.0-alpha.3] — 2026-07-02

- 修复 Windows 窗口选择：统一枚举源并健壮循环。

## [v0.1.0-alpha.2] — 2026-07-01

- 修复 Windows 黑窗口问题。
- 修复中文方框字体。
- 修复窗口选择与透明叠加。

## [v0.1.0-alpha.1] — 2026-07-01

- Release 工作流仅构建并发布 Windows (x86_64)。

## [v0.1.0-alpha.0] — 2026-07-01

- 首个测试版本。
- 新增 Windows Overlay 透明置顶穿透窗口。
- 新增目标窗口跟随功能。
- 基础准心样式支持。

---

[v0.2.0-alpha.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.2.0-alpha.0
[v0.1.0-alpha.12]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.12
[v0.1.0-alpha.11]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.11
[v0.1.0-alpha.10]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.10
[v0.1.0-alpha.9]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.9
[v0.1.0-alpha.8]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.8
[v0.1.0-alpha.7]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.7
[v0.1.0-alpha.6]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.6
[v0.1.0-alpha.5]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.5
[v0.1.0-alpha.4]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.4
[v0.1.0-alpha.3]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.3
[v0.1.0-alpha.2]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.2
[v0.1.0-alpha.1]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.1
[v0.1.0-alpha.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.0
