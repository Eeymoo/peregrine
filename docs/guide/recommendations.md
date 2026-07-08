# 推荐配置

本页先介绍晕 3D 的常见常识，再针对不同类型的游戏给出 Peregrine 的参考配置。关于眩晕原理和通用缓解方法的完整说明，请先阅读 [缓解晕 3D](./motion-sickness.md)。

> ⚠️ **声明**：以下所有推荐配置**仅为参考意见**，不是权威处方。每个人对感官冲突的敏感度不同，最佳配置因人而异。建议把推荐值作为起点，根据自身感受微调。

## 晕 3D 常见常识

### 哪些人更容易晕

3D 眩晕的易感程度因人而异，以下人群通常更容易出现症状[[1]](https://www.healthline.com/health/cybersickness)[[2]](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/)：

- **女性**：统计上女性比男性更容易发生晕动症。
- **偏头痛患者**：偏头痛患者的视觉-前庭系统更敏感。
- **有内耳 / 前庭问题的人**：前庭系统本身已不平衡，更容易被虚拟运动干扰。
- **有晕车 / 晕船 / 晕机史的人**：现实中容易晕的人，玩 3D 游戏通常也更容易晕。
- **平时很少玩 3D 游戏的人**：缺乏适应，初次接触时症状更明显。
- **儿童和青少年**：前庭系统尚未发育完全，对感官冲突更敏感。

### 哪些游戏因素会加重眩晕

很多游戏设置都会影响眩晕程度，以下是最常见的「加重因素」[[3]](https://www.alibaba.com/product-insights/why-do-i-get-motion-sickness-playing-fps-games-potential-fixes.html)[[4]](https://access-ability.uk/2022/04/25/gaming-with-motion-sickness/)[[5]](https://www.polygon.com/gaming/502730/indiana-jones-great-circle-motion-sick-settings-fix-head-bob/)：

| 因素 | 说明 |
|------|------|
| **FOV 过窄** | 视野低于 90° 会产生「望远镜效应」，画面像在隧道里移动，大脑无法感知周围空间。 |
| **帧率低 / 掉帧** | 低于 60 FPS 或不稳定帧率会让画面撕裂、卡顿，加剧感官冲突。 |
| **动态模糊（Motion Blur）** | 模拟高速运动的模糊效果，但会让大脑误判运动方向。 |
| **镜头摇晃 / 头部晃动（Head Bob）** | 模拟走路时的上下颠簸，对很多人是最强的眩晕触发器。 |
| **镜头摇晃（Camera Shake）** | 爆炸、开火、重击等场景的镜头震动。 |
| **色差 / 景深 / 暗角** | 后处理特效会让画面边缘变形或模糊，干扰空间感知。 |
| **快速转身 / 高灵敏度** | 鼠标灵敏度过高导致画面快速旋转，是最常见的 FPS 眩晕诱因。 |
| **垂直运动** | 爬山、飞行、坠落等大量上下视角变化比水平移动更容易引起眩晕。 |
| **游戏内无明确参照物** | 浓雾、黑暗、广阔空旷的场景缺少视觉参照，大脑更难定位。 |

### 常见症状

眩晕症状因人而异，通常会按以下顺序逐渐加重[[1]](https://www.healthline.com/health/cybersickness)：

1. **轻度**：轻微头晕、注意力难以集中、轻微出汗。
2. **中度**：明显头晕、头痛、出汗增多、唾液增多、胃部不适。
3. **重度**：恶心、想吐、面色苍白、冷汗、无法继续游戏。

### 眩晕会累积

3D 眩晕**不会在停止游戏后立刻消失**，症状通常会持续 10 分钟到数小时。而且同一天内多次接触会叠加——如果你上午已经晕过一次，晚上再玩会更快出现症状。因此：

- **一旦感到不适，立即停止**，不要硬撑。
- 症状消退前不要再次玩 3D 游戏。
- 连续多天玩会逐渐减轻（适应效应），但中断后再玩可能重新出现。

## 分场景参考配置

### 场景一：FPS（第一人称射击）

**代表游戏**：CS2、Apex 英雄、使命召唤、守望先锋

**眩晕特点**：FPS 是最容易引起眩晕的类型。第一人称视角 + 快速水平转身 + 开火时的镜头震动，三者叠加会让感官冲突非常强烈[[3]](https://www.alibaba.com/product-insights/why-do-i-get-motion-sickness-playing-fps-games-potential-fixes.html)。

**推荐配置**：

| 参数 | 推荐值 | 说明 |
|------|--------|------|
| 样式（style） | `cross` | 十字准星，提供精确的中心参照，不遮挡视野 |
| 主尺寸（size） | `8`–`12` | 小而精确，避免遮挡画面 |
| 厚度（thickness） | `2` | 细线即可，看得清又不抢眼 |
| 间隙（gap） | `4`–`6` | 中心留空，不干扰瞄准 |
| 不透明度（opacity） | `0.5`–`0.6` | 半透明，能看见但不刺眼 |
| 颜色（color） | 绿色 `[0.2, 1.0, 0.2, 1.0]` 或青色 `[0.0, 1.0, 1.0, 1.0]` | 冷色调在大多数画面中辨识度高，不像白色那样容易融入天空 |

**原理**：FPS 的核心眩晕来自快速水平转身。十字准星标记屏幕正中心，转身时眼睛可以锁定准星，让大脑确认「我其实没在转」。小尺寸是为了不遮挡敌人——FPS 需要时刻关注画面细节。

**额外建议**：务必调大游戏内 FOV（建议 90°–100°），降低鼠标灵敏度以减少快速转身。

---

### 场景二：第三人称射击（TPS）

**代表游戏**：PUBG、全境封锁、战争机器

**眩晕特点**：第三人称视角比第一人称**相对不容易晕**，因为相机距离角色更远，视野更宽，镜头运动更稳定[[4]](https://access-ability.uk/2022/04/25/gaming-with-motion-sickness/)。但 PUBG 这类游戏有大量**跑步时的镜头晃动**和**开镜（ADS）时的视角缩放**，仍可能引起不适[[6]](https://forums.ea.com/discussions/battlefield-6-general-discussion-en/head-bobbing-and-camera-shake-causing-nausea/12760421)。

**推荐配置**：

| 参数 | 推荐值 | 说明 |
|------|--------|------|
| 样式（style） | `cross` 或 `ring` | 十字准星或中心环均可 |
| 主尺寸（size） | `10`–`15`（cross）/ `ring_radius_pct: 0.04` | 比稍大，第三人称视野更宽，中心锚点可以大一点 |
| 厚度（thickness） | `2`–`3` | 略粗以增加可见度 |
| 间隙（gap） | `4`–`8` | |
| 不透明度（opacity） | `0.4`–`0.5` | 第三人称比 FPS 更不易晕，锚点可以更淡 |
| 颜色（color） | 白色 `[1.0, 1.0, 1.0, 1.0]` 或浅绿 `[0.3, 1.0, 0.3, 1.0]` | |

**原理**：第三人称的眩晕主要来自**跑步晃动**和**视角缩放**。锚点的作用是在镜头晃动时提供一个稳定的「地平线」参照。中心环（`ring`）在 TPS 里效果尤其好，因为它不依赖精确瞄准，只需提供一个静止的圆心。

**额外建议**：在游戏设置里关闭或降低「镜头晃动」「头部晃动」「开镜时的动画」选项。

---

### 场景三：跑酷 / 高速移动

**代表游戏**：消逝的光芒、镜之边缘、泰坦陨落 2

**眩晕特点**：跑酷类游戏同时具备**高速移动 + 大量垂直运动（跳跃、滑铲、攀爬）+ 第一人称视角**，是最具挑战性的类型。消逝的光芒的开发者甚至在游戏中内置了「相机稳定性」选项来缓解眩晕[[7]](https://steelseries.com/blog/dying-light-the-beast-tips)。镜之_edge 的开发者则通过极简的白色城市设计来减少视觉干扰[[8]](https://www.notebookcheck.net/Devs-Mirror-s-Edge-s-clean-white-city-a-practical-fix-not-a-pure-artistic-vision.1208014.0.html)。

**推荐配置**：

| 参数 | 推荐值 | 说明 |
|------|--------|------|
| 样式（style） | `ring` 或 `edge_rect` | 中心环适合跳跃参照；贴边矩形提供额外的边缘参照 |
| 主尺寸（size） | `ring_radius_pct: 0.05` / `edge_rect`: `size: 100, secondary_size: 60` | |
| 厚度（thickness） | `2`–`3` | |
| 不透明度（opacity） | `0.5`–`0.7` | 高速移动时画面变化快，锚点需要更明显 |
| 贴边位置（anchor） | `top`（如果是 edge_rect） | 顶部锚点模拟「天花板参照」，跳跃时能快速感知高度变化 |
| 颜色（color） | 高对比度，如亮绿 `[0.0, 1.0, 0.3, 1.0]` 或橙色 `[1.0, 0.6, 0.0, 1.0]` | 跑酷场景画面复杂，需要醒目的颜色 |

**原理**：跑酷游戏的眩晕来自**多方向的剧烈运动**——水平跑动 + 垂直跳跃 + 镜头上下颠簸同时发生。中心环能在最混乱的时刻提供「我相对于地面的位置」参照。贴边矩形则模拟一个固定框架，让大脑在剧烈晃动中感知画面边界。

**额外建议**：务必关闭动态模糊和头部晃动；优先升级「稳定性 / 减震」类技能或装备（消逝的光芒的 Explorer 套装就有此效果）[[7]](https://steelseries.com/blog/dying-light-the-beast-tips)。

---

### 场景四：横向移动为主

**代表游戏**：2D 横版动作、平台跳跃、横版格斗

**眩晕特点**：纯 2D 横版游戏很少引起眩晕，因为画面只有水平移动，没有深度欺骗。但如果玩的是 **3D 渲染的横版游戏**（如《三位一体》、《奥日》的部分场景），或者 2.5D 平台跳跃，仍然可能有轻微不适。

**推荐配置**：

| 参数 | 推荐值 | 说明 |
|------|--------|------|
| 样式（style） | `edge_rect`（anchor: `left` 或 `right`） | 侧边锚点，不干扰中心操作 |
| 主尺寸（size） | `60`–`80` | |
| 厚度（thickness） | `2` | |
| 不透明度（opacity） | `0.3`–`0.4` | 横版眩晕本来就轻，锚点可以很淡 |
| 贴边位置（anchor） | `left` 或 `right` | 放在移动方向的反方向，提供「固定背景」参照 |

**原理**：横向移动时画面整体左右平移，大脑最容易丢失「我在哪」的感觉。侧边锚点相当于一个「墙」，当画面向右滚动时，锚点不动，大脑就能确认画面在动而我没动。

---

### 场景五：纵向移动为主

**代表游戏**：攀爬类、飞行模拟、垂直卷轴、坠落机制多的游戏

**眩晕特点**：**垂直运动比水平运动更容易引起眩晕**，因为内耳对上下加速度（重力方向）的感知更敏感[[2]](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/)。大量上升、坠落、俯冲的场景会强烈刺激前庭系统的冲突感。

**推荐配置**：

| 参数 | 推荐值 | 说明 |
|------|--------|------|
| 样式（style） | `large_cross` 或 `border_frame` | 大准星提供完整的水平和垂直参照线；边框提供整体框架 |
| 主尺寸（size） | `large_cross`: 屏幕半径 | 大准星从中心延伸到边缘，提供完整的纵横参照 |
| 厚度（thickness） | `1`–`2` | 细线避免遮挡过多画面 |
| 不透明度（opacity） | `0.3`–`0.5` | 纵向场景眩晕重，但锚点也要避免太抢眼 |
| 颜色（color） | 白色 `[1.0, 1.0, 1.0, 1.0]` 或浅蓝 `[0.3, 0.5, 1.0, 1.0]` | |
| 边框缺口（border_gap） | `true`（如果用 border_frame） | 避免遮挡顶部状态栏和底部小地图 |

**原理**：纵向运动时，画面整体上下移动，大脑最难判断「我是在上升还是下坠」。大准星的垂直线就像一根**铅垂线**，无论画面怎么晃动，这根线始终垂直于地面，帮助大脑确认重力的真实方向。边框则像一个固定窗口，把整个画面框住。

**额外建议**：如果游戏有坠落机制（如《只狼》的跳跃、《艾尔登法环》的下落），尽量控制下落速度，避免长时间自由落体。

## 通用调参建议

### 锚点大小

- **越小越好**，只要能看清就行。锚点太大反而会成为新的视觉干扰源。
- FPS / 竞技类游戏优先小尺寸（8–15px），休闲 / 探索类可以稍大。
- 如果看不清，先尝试**增加不透明度**而不是增大尺寸。

### 不透明度

- 从 `0.4` 开始，逐步调整到「能注意到但不分散注意力」的程度。
- 通常 `0.3`–`0.6` 是大多数人的舒适区间。
- 画面整体偏暗的游戏（如恐怖游戏）可以适当提高不透明度。

### 颜色

- 选择与游戏主色调**对比度高**的颜色。
  - 红色 / 橙色画面多的游戏（如战争、火焰场景）→ 用青色或绿色锚点。
  - 绿色森林 / 草地多的游戏 → 用白色或红色锚点。
  - 天空 / 海洋蓝色调多的游戏 → 用橙色或黄色锚点。
- 白色是最通用的选择，但在明亮场景（雪地、天空）中可能融入背景。
- 也可以使用自定义 PNG 贴图（`CustomImage` 样式）来实现更精细的视觉效果。

### 位置

- **中心**（`Cross` / `Ring`）：最常见的选择，适合大多数场景。
- **边缘**（`EdgeRect` / `BorderFrame`）：当你不想在画面正中央放东西时，边缘锚点可以提供框架感而不干扰操作。
- **角落**（`CornerDots`）：最不显眼的方案，只在四角放小点，提供最小限度的参照。

### 经验法则

1. **先从默认配置开始**，玩 5–10 分钟，感受是否有效。
2. 如果仍然晕，**先尝试换样式**（cross → ring → edge_rect → border_frame），找到最适合自己的锚点类型。
3. 找到合适的样式后，**再微调尺寸和不透明度**。
4. **一次只改一个参数**，这样能判断是哪个改动起的作用。
5. 记录下自己最舒适的配置组合，保存为 Profile 方便日后调用。

## 参考资料

- [[1] Healthline — Cybersickness: What It Is, Symptoms, Causes, and Treatments](https://www.healthline.com/health/cybersickness)
- [[2] Karmali, F. et al. — Validating sensory conflict theory and mitigating motion sickness (NIH PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/)
- [[3] Alibaba Product Insights — Why Do I Get Motion Sickness Playing FPS Games: Potential Fixes](https://www.alibaba.com/product-insights/why-do-i-get-motion-sickness-playing-fps-games-potential-fixes.html)
- [[4] Access Ability — Gaming with Motion Sickness](https://access-ability.uk/2022/04/25/gaming-with-motion-sickness/)
- [[5] Polygon — How to fix Indiana Jones and the Great Circle motion sickness problems](https://www.polygon.com/gaming/502730/indiana-jones-great-circle-motion-sick-settings-fix-head-bob/)
- [[6] EA Forums — Head bobbing and camera shake causing nausea](https://forums.ea.com/discussions/battlefield-6-general-discussion-en/head-bobbing-and-camera-shake-causing-nausea/12760421)
- [[7] SteelSeries — How to Survive the Night in Dying Light: The Beast](https://steelseries.com/blog/dying-light-the-beast-tips)
- [[8] Notebookcheck — Devs: Mirror's Edge's clean white city a practical fix](https://www.notebookcheck.net/Devs-Mirror-s-Edge-s-clean-white-city-a-practical-fix-not-a-pure-artistic-vision.1208014.0.html)
