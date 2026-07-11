# 缓解晕 3D

本页解释什么是 3D 眩晕、它为什么发生，以及如何缓解。Peregrine 提供的视觉锚点只是众多缓解手段中的一种，本章也会列出其他经过验证的方法，帮助你找到最适合自己的组合。

## 什么是 3D 眩晕

玩第一人称（FPS）、第三人称 3D 游戏，或使用 VR 设备时，部分玩家会感到头晕、恶心、出冷汗、甚至呕吐。这种现象在医学上被称为**晕动症（Motion Sickness）**，当它由屏幕上的虚拟运动引发时，也被称为**模拟器眩晕（Simulator Sickness）**或**电子眩晕（Cybersickness）**[[1]](https://www.healthline.com/health/cybersickness)。

简单来说，3D 眩晕和晕车、晕船、晕机是同一类反应，只是触发场景不同。

## 为什么会眩晕：感官冲突理论

目前最被广泛接受的解释是**感官冲突理论（Sensory Conflict Theory）**[[2]](https://cdnsciencepub.com/doi/10.1139/y90-044)[[3]](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/)。

人体维持平衡感依赖三套系统的协同：

- **视觉系统**（眼睛）——告诉我「我在往哪走、走多快」
- **前庭系统**（内耳）——告诉我「我的头在怎样运动、是否在加速/旋转」
- **本体感觉**（肌肉、关节）——告诉我「我的身体在什么姿势」

在现实世界里，这三套系统的信号几乎总是彼此吻合的。但在玩 3D 游戏时会出现矛盾：

> **眼睛看到**画面里的角色在奔跑、转身、下坠；
> **内耳和身体却感到**自己稳稳地坐在椅子上，没有任何运动。

大脑接收到了互相矛盾的信息，无法判断「我到底有没有在动」，于是触发了古老的防御反应——**恶心和呕吐**。学术界推测，大脑把这种感官冲突当作「中毒导致神经信号错乱」的信号，于是通过催吐来自救[[2]](https://cdnsciencepub.com/doi/10.1139/y90-044)。

这也是为什么有些人更容易晕 3D：个体对感官冲突的敏感度不同，女性、偏头痛患者、有内耳问题的人通常更易感[[3]](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/)。

## 视觉锚点为什么有效

缓解感官冲突的一个关键思路是：**给大脑提供一个稳定、静止的参照物**，让它确认「我其实没在动」。

这就是 Peregrine 的核心原理——在屏幕上叠加一个**固定的视觉锚点**（十字准星、中心圆环、边缘标记等）。当游戏画面剧烈晃动时，这个锚点始终纹丝不动，眼睛会把它当作「地面」或「地平线」，从而减轻感官冲突。

这一方法有学术证据支持。韩国 Seok 等人在 *JMIR Serious Games* 发表的研究发现：在 FPS 游戏中开启视觉锚点（视觉引导）能**显著降低** VR 眩晕症状，尤其是在使用手柄操作时；锚点尺寸为画面比例的 30% 左右、置于头部追踪方向时效果最好[[4]](https://games.jmir.org/2021/3/e18020/)。

Peregrine 将这一原理做成了通用工具：不止十字准星，你可以选择边缘矩形、中心圆环、边缘标记、边框、边缘箭头甚至自定义图片，找到最适合你的锚点形态。

## 缓解晕 3D 的方法

下面列出多种经过验证或普遍推荐的缓解方法，建议组合使用。Peregrine 对应其中的「视觉锚点」一项。

### 1. 使用视觉锚点（Peregrine 的核心功能）

在屏幕中心或边缘放置一个半透明、固定不动的参照物。这是 Peregrine 提供的主要手段，也是目前最便捷的软件级缓解方式[[4]](https://games.jmir.org/2021/3/e18020/)。

**Peregrine 使用建议：**

- 优先尝试 `cross`（十字准星）或 `ring`（中心圆环），对多数人效果最好。
- 锚点不要太大、不要太亮，半透明即可，避免遮挡游戏画面。
- 如果中心十字准星影响瞄准，可以试试 `edge_rect`（边缘矩形）或 `border_frame`（边框），把锚点放在边缘。

### 2. 调整视野（FOV）

很多第一人称游戏默认的视野（Field of View）偏窄（60°–70°），会让大脑觉得「眼前的东西离我很近」，加剧眩晕。适当**调大 FOV**（通常建议 90°–110°）能给大脑更接近自然视角的画面，减轻不适[[5]](https://www.cise.ufl.edu/~eragan/papers/Benda_TVCG2023.pdf)。

> 注意：VR 场景下的 FOV 研究结论相反——过宽的 FOV 会**加重**眩晕，开发者常在快速移动时动态收窄 FOV 来缓解[[6]](https://www.researchgate.net/publication/354730991_Tunnel_Vision_-_Dynamic_Peripheral_Vision_Blocking_Glasses_for_Reducing_Motion_Sickness_Symptoms)。普通桌面显示器游戏一般调大 FOV 即可。

### 3. 提高帧率、降低延迟

低帧率和画面撕裂会让感官冲突更严重。尽量做到：

- 游戏帧率稳定在 **60 FPS 以上**（高刷新率显示器更好）。
- 关闭或降低动态模糊（Motion Blur）、镜头摇晃（Camera Shake）、色差（Chromatic Aberration）等特效。
- 降低输入延迟，关闭垂直同步（V-Sync）如果帧率足够高[[7]](https://www.csit.carleton.ca/~rteather/pdfs/theses/thesis-yasinfarmani-final.pdf)。

### 4. 调整坐姿与观看距离

- **坐远一点**：离屏幕越近，视野被游戏画面占据的比例越大，越容易眩晕。适当拉远距离能让余光看到周围的静止环境（桌面、墙壁），提供额外的稳定参照物[[8]](https://www.hyperoptic.com/broadband/explained/lifestyle-and-entertainment/the-top-10-tips-for-avoiding-simulation-sickness-when-gaming/)。
- **开一盏灯**：不要在全黑环境里玩，保留环境光有助于大脑感知真实空间。
- **使用更小的屏幕**：有建议认为较小的屏幕能减少感官矛盾[[8]](https://www.hyperoptic.com/broadband/explained/lifestyle-and-entertainment/the-top-10-tips-for-avoiding-simulation-sickness-when-gaming/)。

### 5. 间歇休息、逐步适应

- 每玩 **30–45 分钟**就暂停，看远处或闭眼几分钟。
- 眩晕是会累积的，**一旦感到不适立即停止**，不要硬撑——继续玩只会更难受。
- 人体对感官冲突有**适应（Habituation）**能力，短时间内反复、少量地接触会逐渐减轻症状。可以从每次 10 分钟开始，逐日延长。

### 6. 其他物理与饮食方法

这些方法同样适用于晕车，来自 NHS、UC Davis Health 等权威健康机构的建议[[9]](https://www.nhs.uk/conditions/motion-sickness/)[[10]](https://health.ucdavis.edu/blog/cultivating-health/motion-sickness-how-you-can-prevent-symptoms-and-enjoy-travel/2024/05)：

- **生姜**：有临床证据显示生姜能减轻晕动症引起的恶心，玩前喝姜茶或含姜片有一定帮助[[10]](https://health.ucdavis.edu/blog/cultivating-health/motion-sickness-how-you-can-prevent-symptoms-and-enjoy-travel/2024/05)。
- **清淡饮食**：玩前避免油腻、辛辣、过饱，也不要空腹。
- **通风、降温**：闷热会加重恶心感，开窗或开空调保持凉爽。
- **避免玩时低头看手机或阅读**：额外的视觉运动冲突会雪上加霜[[9]](https://www.nhs.uk/conditions/motion-sickness/)。

### 7. 药物（严重情况）

如果上述方法都不够，可以考虑非处方晕车药（抗组胺类，如茶苯海明/乘晕宁）。但药物会带来嗜睡副作用，且效果因人而异，建议**咨询医生**后使用，不要依赖药物长期硬玩[[9]](https://www.nhs.uk/conditions/motion-sickness/)。

## 总结

| 方法 | 原理 | 是否需要工具 |
|------|------|-------------|
| 视觉锚点 | 提供静止参照物，减轻感官冲突 | ✅ Peregrine |
| 调大 FOV | 让画面更接近自然视角 | 游戏设置 |
| 提高帧率 | 减少画面撕裂和延迟 | 游戏设置 / 硬件 |
| 坐远一点 / 开灯 | 用余光提供稳定环境参照 | 无 |
| 间歇休息、逐步适应 | 利用人体适应能力 | 无 |
| 生姜、清淡饮食、通风 | 减轻恶心反应 | 无 |
| 晕车药 | 抑制前庭反应 | 需咨询医生 |

**推荐组合**：Peregrine 视觉锚点 + 调大 FOV + 稳定 60FPS + 适时休息，能覆盖大多数轻度到中度的 3D 眩晕场景。如果症状严重或持续不缓解，建议咨询医生排除前庭系统问题。

## 参考资料

- [[1] Healthline — Cybersickness: What It Is, Symptoms, Causes, and Treatments](https://www.healthline.com/health/cybersickness)
- [[2] Reason, J. T., & Brand, J. J. — Motion sickness: a synthesis and evaluation of the sensory conflict theory (Canadian Science Publishing)](https://cdnsciencepub.com/doi/10.1139/y90-044)
- [[3] Karmali, F. et al. — Validating sensory conflict theory and mitigating motion sickness with galvanic vestibular stimulation (NIH PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/)
- [[4] Seok, K. H. et al. — Using Visual Guides to Reduce Virtual Reality Sickness in FPS Games (JMIR Serious Games)](https://games.jmir.org/2021/3/e18020/)
- [[5] Benda, P. et al. — Seated vs Standing VR: Reducing Simulator Sickness (IEEE TVCG 2023)](https://www.cise.ufl.edu/~eragan/papers/Benda_TVCG2023.pdf)
- [[6] Tunnel Vision — Dynamic Peripheral Vision Blocking Glasses for Reducing Motion Sickness Symptoms](https://www.researchgate.net/publication/354730991_Tunnel_Vision_-_Dynamic_Peripheral_Vision_Blocking_Glasses_for_Reducing_Motion_Sickness_Symptoms)
- [[7] Farmani, Y. — Mitigating Cybersickness in Virtual Reality: Challenges and Solutions (Carleton University Thesis)](https://www.csit.carleton.ca/~rteather/pdfs/theses/thesis-yasinfarmani-final.pdf)
- [[8] Hyperoptic — The top 10 tips for avoiding simulation sickness when gaming](https://www.hyperoptic.com/broadband/explained/lifestyle-and-entertainment/the-top-10-tips-for-avoiding-simulation-sickness-when-gaming/)
- [[9] NHS — Motion sickness](https://www.nhs.uk/conditions/motion-sickness/)
- [[10] UC Davis Health — Motion sickness: How you can prevent symptoms and enjoy travel](https://health.ucdavis.edu/blog/cultivating-health/motion-sickness-how-you-can-prevent-symptoms-and-enjoy-travel/2024/05)
