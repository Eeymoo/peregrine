# startup-metrics Specification

## Purpose

定义 Peregrine 的匿名启动统计能力：维护与配置解耦的 `install_id` 文件生命周期，并在遥测启用时上报一条 Info 级启动事件，用于按 install_id 去重估算真实用户数量。启动事件 MUST NOT 进入 GlitchTip issue 列表，仅出现在事件流。

## Requirements

### Requirement: install_id 独立文件生命周期

系统 SHALL 在应用数据目录（Windows：`%APPDATA%/Peregrine/`）维护独立的 `install_id` 文件，内容为 UUID v4 纯文本。首次启动时生成；之后每次启动读取复用；文件损坏或为空时 MUST 重新生成。写入 MUST 为原子写（临时文件 + rename）。install_id MUST 与 config.json 解耦：配置重置、导出、分享 MUST NOT 携带 install_id；删除 install_id 文件即视为「新安装」。install_id 为匿名随机串，MUST NOT 关联任何硬件信息或真实身份。

#### Scenario: 首次启动生成 install_id

- **WHEN** 应用数据目录中不存在 install_id 文件
- **THEN** 系统生成 UUID v4 并原子写入 install_id 文件

#### Scenario: 同一安装多次启动 id 不变

- **WHEN** 应用在同一安装环境下多次启动
- **THEN** 每次读取到的 install_id 完全一致

#### Scenario: 文件损坏自动重建

- **WHEN** install_id 文件内容为空或损坏
- **THEN** 系统重新生成 UUID v4 并原子覆盖写回

#### Scenario: 重置配置不影响 install_id

- **WHEN** 用户删除或重置 config.json 后重启应用
- **THEN** install_id 保持不变

#### Scenario: 不同安装 id 不同

- **WHEN** 两台机器（或删除 install_id 后）分别启动应用
- **THEN** 两者 install_id 不同

### Requirement: 启动统计事件

遥测已启用时，系统 SHALL 在应用启动进入主循环后上报一条 `Info` 级事件（message 为 `app_startup`），携带 tag：`code=PGR-0001`（启动专属 Code）、`event_type=startup`、`priority=p3`、`install_id`、`version`（与 Cargo.toml/package.json 一致）、`os`、`arch`。该事件 MUST 使用 Info 级别，MUST NOT 进入 GlitchTip issue 列表（仅出现在事件流），用于按 install_id 去重估算真实用户数量。

#### Scenario: 启动后上报 Info 级事件

- **WHEN** 遥测启用且应用完成启动进入主循环
- **THEN** 上报一条 Info 级 `app_startup` 事件
- **AND** 事件携带 code/event_type/install_id/version/os/arch tag

#### Scenario: 启动事件携带启动 Code

- **WHEN** 启动统计事件发送至 GlitchTip
- **THEN** 事件 `code` tag 为启动专属 Code `PGR-0001`
- **AND** 可按该 Code 在 GlitchTip 中筛出全部启动事件

#### Scenario: 启动事件不进 issue 列表

- **WHEN** 启动统计事件发送至 GlitchTip
- **THEN** 该事件不出现在 issue 列表，仅可在事件流中查看

#### Scenario: 遥测关闭时不上报启动事件

- **WHEN** `telemetry_enabled` 为 false 或未配置 DSN
- **THEN** 不产生启动统计事件，无任何网络请求
