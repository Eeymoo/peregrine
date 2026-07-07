# 配置说明

配置文件为 JSON 格式，位于 Windows 的 `%APPDATA%\Peregrine\config.json`。

## 配置结构

```json
{
  "active_profile": "default",
  "profiles": {
    "default": {
      "crosshair": {
        "style": "Cross",
        "size": 32,
        "thickness": 2,
        "color": "#ff0000",
        "opacity": 0.8
      },
      "trigger": null,
      "settings_hotkey": "Tab",
      "target_window": null
    }
  }
}
```

## 字段说明

### AppConfig

| 字段 | 类型 | 说明 |
|------|------|------|
| `active_profile` | string | 当前激活的 Profile 名称 |
| `profiles` | map | 所有 Profile，键为名称 |

### Profile

| 字段 | 类型 | 说明 |
|------|------|------|
| `crosshair` | Crosshair | 准心配置 |
| `trigger` | TriggerRule | 进程触发规则（可选） |
| `settings_hotkey` | string | 打开设置面板的快捷键 |
| `target_window` | string | 目标窗口标题（可选） |

### Crosshair

| 字段 | 类型 | 说明 |
|------|------|------|
| `style` | CrosshairStyle | 准心样式 |
| `size` | number | 尺寸 |
| `thickness` | number | 线宽 |
| `color` | string | 颜色（十六进制） |
| `opacity` | number | 不透明度（0.0 ~ 1.0） |

## 热重载

配置文件被外部编辑器修改并保存后，`ConfigWatcher` 会在约 300ms 去抖后检测变更，并通过 `ConfigNotifier` 广播新配置，渲染器立即使用最新设置，无需重启。
