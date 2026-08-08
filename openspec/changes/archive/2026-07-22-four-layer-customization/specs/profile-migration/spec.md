## ADDED Requirements

### Requirement: 旧配置文件自动迁移到新格式

系统 SHALL 在 `load_or_create_default` 中检测旧配置格式（同时满足：含 `crosshair` 字段、无 `layers` 字段）并自动迁移为新格式（多图层）。

迁移过程 MUST：
1. 备份原始配置文件为 `<name>.legacy.bak`（保留用户数据）
2. 将旧 `Crosshair` 按"样式 → 内置物料"映射表转换为单个 `Layer`
3. 将旧 `Crosshair` 的样式专用字段（如 `ring_radius_pct`、`arrow_*`、`grid_*` 等）映射为该物料的 `params`
4. 写入新格式配置（含 `layers: Vec<Layer>`）
5. 在日志中记录迁移操作

迁移 MUST 在用户无感知的情况下完成（用户启动应用看到的视觉效果与旧版一致）。

#### Scenario: 旧 EdgeRect 配置迁移为 builtin.edge_rect 物料

- **WHEN** 加载一份旧配置，其 `profiles.default.crosshair.style = "edge_rect"`，`size = 180`，`anchor = "top"`
- **THEN** 迁移后新配置的 `profiles.default.layers` SHALL 是单元素列表
- **AND** 该图层的 `material` 为 `MaterialRef::Builtin { id: "builtin.edge_rect" }`
- **AND** 该图层的 `params` 包含 `{size: 180, anchor: "top", ...}`（其余字段取物料 defaults）
- **AND** 原配置文件被备份为 `config.json.legacy.bak`

#### Scenario: 旧 CustomImage 配置迁移

- **WHEN** 旧配置的 `crosshair.style = "custom_image"`，含 `image_path`、`image_scale`、`image_offset_x/y`
- **THEN** 迁移后图层的 `material` 为 `builtin.image`
- **AND** 图层的 `params` 含 `{image_path, image_scale, image_offset_x, image_offset_y}`

#### Scenario: 旧 RandomOrb 配置迁移保持种子一致

- **WHEN** 旧配置的 `crosshair.style = "random_orb"`，含 `random_orb_offset`、`random_orb_jitter` 等字段
- **THEN** 迁移后图层的 `params` 含所有 `random_*` 字段
- **AND** 启动后 `builtin.random_orb` 物料渲染结果与旧版完全一致（随机球位置相同）

#### Scenario: 迁移后视觉效果零变化

- **WHEN** 一份旧配置被迁移为新格式
- **THEN** 迁移后启动 overlay 的渲染结果 MUST 与迁移前旧版本渲染结果逐像素相等
- **AND** 用户视觉上无任何可感知变化

### Requirement: 迁移覆盖全部 12 种旧样式

迁移逻辑 MUST 支持全部 12 种旧 `CrosshairStyle` 变体（含 `toilet_paper` alias）。字段映射表如下：

| 旧 style | 新物料 id | 参数映射 |
|---|---|---|
| `edge_rect` / `toilet_paper` | `builtin.edge_rect` | `size, secondary_size, anchor, margin, corner_radius` |
| `cross` | `builtin.cross` | `size, thickness, gap` |
| `large_cross` | `builtin.large_cross` | `thickness` |
| `corner_dots4` | `builtin.corner_dots` | `count: 4, offset, radius, thickness` |
| `corner_dots6` | `builtin.corner_dots` | `count: 6, offset, radius, thickness` |
| `corner_dots8` | `builtin.corner_dots` | `count: 8, offset, radius, thickness` |
| `ring` | `builtin.ring` | `ring_radius_pct, ring_style, thickness` |
| `custom_orb` | `builtin.custom_orb` | `radius, offset, orb_positions, custom_orb_*_count` |
| `random_orb` | `builtin.random_orb` | `random_*` 全部字段 |
| `border_frame` | `builtin.border_frame` | `thickness, offset, border_frame_style, border_inset` |
| `custom_image` | `builtin.image` | `image_path, image_scale, image_offset_x, image_offset_y` |
| `edge_arrows` | `builtin.edge_arrows` | `size, arrow_*, orb_positions` |
| `grid` | `builtin.grid` | `grid_size, grid_alignment, thickness` |

通用字段 `color`、`opacity` 映射到 `LayerStyle.color` 和 `LayerStyle.opacity`。

#### Scenario: corner_dots4/6/8 映射到同一物料的 count 参数

- **WHEN** 旧配置 `style = "corner_dots6"`
- **THEN** 迁移后图层 `material = builtin.corner_dots`，`params.count = 6`

#### Scenario: toilet_paper alias 正确迁移

- **WHEN** 旧配置 `style = "toilet_paper"`（旧别名）
- **THEN** 迁移后图层 `material = builtin.edge_rect`（与 `edge_rect` 等价）

### Requirement: 已迁移配置不被重复迁移

系统 MUST 在迁移后写入新格式配置（含 `layers` 字段）。下次启动时 `load_or_create_default` 检测到 `layers` 字段存在且 `crosshair` 字段不存在时，SHALL 直接按新格式加载，不再迁移。

若检测到 `layers` 与 `crosshair` 同时存在（异常情况），系统 SHALL 以 `layers` 为准并记录警告日志。

#### Scenario: 迁移后再次启动不重复迁移

- **WHEN** 一份配置在 v0.2.0 中被迁移为新格式
- **AND** 用户再次启动应用
- **THEN** 配置 SHALL 直接按新格式加载，不触发迁移逻辑
- **AND** 不产生新的 `.legacy.bak` 文件

#### Scenario: 同时含 layers 和 crosshair 的异常配置

- **WHEN** 配置文件同时包含 `layers` 和 `crosshair` 字段（可能由手动编辑或第三方工具导致）
- **THEN** 系统 SHALL 以 `layers` 为准加载
- **AND** 记录 warn 级别日志说明此异常
- **AND** 不报错，不阻塞启动

### Requirement: 迁移失败时降级到默认配置

若迁移过程中发生任何错误（如旧 `crosshair` 字段缺失关键 key、样式枚举值未知、参数值非法），系统 SHALL：
1. 备份损坏的配置文件为 `<name>.legacy.bak.error`（区别于正常迁移的 `.legacy.bak`）
2. 回退到默认配置（含一个 `builtin.edge_rect` 单图层 Profile）
3. 写入默认配置
4. 记录 warning 级别日志说明错误详情
5. 不阻塞启动

#### Scenario: 迁移时遇到未知样式枚举

- **WHEN** 旧配置的 `crosshair.style = "future_style"`（新版本未知样式）
- **THEN** 系统 SHALL 回退到默认配置
- **AND** 原文件被备份为 `config.json.legacy.bak.error`
- **AND** 日志记录"无法识别的 crosshair style: future_style"

#### Scenario: 迁移时参数值非法

- **WHEN** 旧配置的 `crosshair.size = -1`（负值，违反校验）
- **THEN** 系统 SHALL 回退到默认配置
- **AND** 日志记录迁移失败原因
- **AND** 不抛出 panic，应用正常启动

### Requirement: 迁移逻辑有完整单元测试覆盖

`crates/config` MUST 提供迁移模块的单元测试，至少覆盖：
- 12 种旧样式各自的迁移用例
- `toilet_paper` alias 迁移
- 字段值完全保留（数值、字符串、枚举）
- 视觉等价性验证（迁移后 `build_shapes` 输出与旧版本逐元素相等）
- 异常输入降级到默认配置

测试 MUST 使用真实物料库（`builtin_cross.rhai` 等）求值，而非 mock。

#### Scenario: 所有旧样式的迁移测试通过

- **WHEN** 运行 `cargo test -p peregrine_config migration`
- **THEN** 13 个迁移测试用例 MUST 全部通过（12 种样式 + `toilet_paper` alias）
- **AND** 每个用例验证迁移后图层 `params` 与预期值逐字段相等

#### Scenario: 迁移前后视觉一致性的回归测试

- **WHEN** 对每个旧 style 用同一组典型参数（如 `size: 100, thickness: 4, opacity: 0.7`）生成旧 `Crosshair` 并 `build_shapes`
- **AND** 迁移为新格式后再次 `build_shapes`
- **THEN** 两次输出 MUST 逐元素相等（Element 数量、类型、坐标完全一致）
