## ADDED Requirements

### Requirement: 独立下载页

文档站 MUST 提供独立下载页（en `/download` 与 zh-cn `/zh-cn/download`），页面上不硬编码任何版本号，版本与下载 URL 在构建时从 GitHub Releases API 动态获取。

#### Scenario: 下载页路由可用

- **WHEN** 访问 `/download`（en）与 `/zh-cn/download`（zh-cn）
- **THEN** 均返回 HTTP 200 且展示下载内容，两语言页面齐全

#### Scenario: 版本号动态获取

- **WHEN** 文档站构建
- **THEN** 下载页通过 GitHub Releases API（`https://api.github.com/repos/Eeymoo/peregrine/releases` 或 `releases/latest`）读取最新发布信息，页面展示的版本号与下载链接由 `assets[].browser_download_url` 生成，源码中无版本号硬编码

#### Scenario: API 不可用降级

- **WHEN** GitHub API 请求失败（限流 / 网络异常）
- **THEN** 下载页仍可渲染，至少提供「查看 GitHub Releases」外链，页面不空白不报错

#### Scenario: 查看更多版本入口

- **WHEN** 用户点击下载页「查看更多版本」
- **THEN** 跳转到 GitHub Releases 页面（`https://github.com/Eeymoo/peregrine/releases`）

### Requirement: 下载页交互通道

下载页 MUST 提供按架构筛选与稳定版/预发布版通道切换；zh-cn locale MUST 额外提供「GitHub 加速 / 直连」切换，en locale 不渲染加速通道。

#### Scenario: 架构筛选

- **WHEN** 用户选择某个硬件架构（x64 / x86 / ARM64）
- **THEN** 表格仅显示对应架构的行，其余隐藏

#### Scenario: 下载通道切换

- **WHEN** 用户切换「稳定版 / 预发布版」通道
- **THEN** 表格切换到对应通道的版本资产；若某通道无对应 release，则不渲染该通道选项

#### Scenario: 中文加速通道

- **WHEN** 用户在 zh-cn 下载页选择「GitHub 加速」并选定一个代理前缀
- **THEN** 下载链接拼接所选 gh-proxy 前缀（如 `https://ghfast.top/`），可下拉切换多个候选前缀；选择「直连」则使用原始 `browser_download_url`

#### Scenario: 英文无加速通道

- **WHEN** 用户访问 en 下载页
- **THEN** 页面不渲染「GitHub 加速」选项，仅提供直连链接

## MODIFIED Requirements

### Requirement: URL 路径保持不变

下载页的引入放宽既有「URL 路径保持不变」约束：存量页面路径 MUST 保持不变，同时新增 `/download`（en）与 `/zh-cn/download`（zh-cn）两个新路径。

#### Scenario: 存量路径不变

- **WHEN** 访问既有 `/guide/usage`、`/zh-cn/guide/usage`、`/` 与 `/zh-cn/`
- **THEN** 仍返回 HTTP 200 且内容不变

#### Scenario: 新增下载页路径

- **WHEN** 访问 `/download`（en）与 `/zh-cn/download`（zh-cn）
- **THEN** 返回 HTTP 200 且展示下载内容
