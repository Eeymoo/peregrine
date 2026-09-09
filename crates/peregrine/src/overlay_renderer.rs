//! softbuffer 版遮盖层渲染器。
//!
//! 用 CPU 像素缓冲区（softbuffer）替代 wgpu swapchain，
//! 参考 simple-crosshair-overlay 的方案：
//! - `with_transparent(true)` 让 winit 启用 DWM 透明
//! - 像素格式 `0xAARRGGBB`，透明区域填 `0x00000000`
//! - Windows 上需要预乘 alpha
//!
//! 优点：不涉及 swapchain/DirectComposition，透明天然可靠。
//! 缺点：需要自己实现像素光栅化（矩形、圆、线段）。

// 像素光栅化原语参数较多（坐标、尺寸、颜色等），允许超过 clippy 默认限制。
#![allow(clippy::too_many_arguments)]

use peregrine_config::{Crosshair, CrosshairStyle, RendererBackend};
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use winit::window::Window;

/// softbuffer 版遮盖层渲染器。
pub struct OverlayRenderer {
    /// winit 窗口（`Rc` 包裹以满足 softbuffer 生命周期要求）。
    window: Arc<Window>,
    /// softbuffer 上下文。
    #[allow(dead_code)]
    context: softbuffer::Context<Arc<Window>>,
    /// softbuffer 表面。
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    /// 当前配置快照。
    config: Arc<Mutex<peregrine_config::ConfigSnapshot>>,
    /// 物料注册表（用于图层求值）。
    material_registry: Arc<peregrine_material::MaterialRegistry>,
    /// PNG 图片缓存：路径 → 解码后的 RGBA 像素。
    image_cache: Option<CachedImage>,
    /// 上一帧渲染指纹：求值输出 + 尺寸 + 抗锯齿 + 渲染后端 + 图片路径缓存纪元。
    ///
    /// 动态物料（如时钟）每秒才改变一次输出，而调度节拍可达 60/120FPS——
    /// 指纹相同的帧直接跳过全部光栅化（清屏 / Rhai 已在上游跳过 / resvg），
    /// 把「持续重绘」的稳态成本压到一次指纹比较。任一影响像素的输入
    /// （shapes 输出 / 窗口尺寸 / 设置 / 图片内容）变化都会改变指纹。
    last_frame_fingerprint: u64,
    /// 图片缓存纪元：图片（重）加载时递增，参与指纹防止「路径相同但内容变了」漏重绘。
    image_cache_epoch: u64,
}

/// 已解码的 PNG 图片，包含原始像素数据和尺寸。
struct CachedImage {
    /// 用于缓存匹配的路径。
    path: String,
    /// RGBA 像素数据（行优先，从上到下）。
    pixels: Vec<(u8, u8, u8, u8)>,
    /// 原始宽度（像素）。
    width: usize,
    /// 原始高度（像素）。
    height: usize,
}

impl OverlayRenderer {
    /// 创建渲染器。
    pub fn new(
        window: Arc<Window>,
        config: Arc<Mutex<peregrine_config::ConfigSnapshot>>,
        material_registry: Arc<peregrine_material::MaterialRegistry>,
    ) -> Self {
        // softbuffer 要求 Context 和 Surface 共享同一个 window 引用。
        let context = softbuffer::Context::new(window.clone()).expect("create softbuffer context");
        let surface =
            softbuffer::Surface::new(&context, window.clone()).expect("create softbuffer surface");

        Self {
            window,
            context,
            surface,
            config,
            material_registry,
            image_cache: None,
            last_frame_fingerprint: 0,
            image_cache_epoch: 0,
        }
    }

    /// 窗口大小变化时调用（softbuffer 在 render 时自动 resize，此处空实现）。
    pub fn resize(&mut self, _new_size: winit::dpi::PhysicalSize<u32>) {
        // softbuffer 的 resize 在 render_overlay 中按当前窗口尺寸自动处理。
    }

    /// 物料热重载后替换 registry 句柄（整体替换，无部分更新窗口）。
    ///
    /// 供 overlay 线程在 `RefreshMaterials` 命令中调用，
    /// 使运行中的渲染器无需重建即感知新物料（含新的 `is_dynamic`）。
    pub fn update_material_registry(
        &mut self,
        material_registry: Arc<peregrine_material::MaterialRegistry>,
    ) {
        self.material_registry = material_registry;
    }

    /// 渲染一帧遮盖层。
    ///
    /// 返回 `Result<(), Box<dyn std::error::Error>>`：仅 `surface.resize()` 失败
    /// 时上抛为 Err（对应 OVERLAY_RENDER PGR-4101），由调用方经 `safe_try!` 上报；
    /// 内部其他吞错点（`config.lock().expect` / 几何计算 / material evaluate）
    /// 维持原状（panic 由 PGR-1001 兜底，属逻辑 bug 而非运行时故障）。
    pub fn render_overlay(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let size = self.window.inner_size();
        let width = size.width;
        let height = size.height;
        if width == 0 || height == 0 {
            return Ok(());
        }

        tracing::debug!(
            width,
            height,
            scale = self.window.scale_factor(),
            "overlay render_overlay: window inner_size"
        );

        // 调整 softbuffer 缓冲区尺寸：唯一上抛为 Err 的故障点（OVERLAY_RENDER）。
        // 其余内部错误维持吞错原状（见函数文档）。
        if let Err(e) = self.surface.resize(
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        ) {
            // 不再在此处 tracing::error! 重复记录——safe_try! 上报路径已含错误信息。
            return Err(e.into());
        }

        // 读取当前准心配置。
        let config = self.config.lock().expect("config lock");
        let profile = config.active_profile();
        let antialiasing = config.settings.antialiasing;
        let renderer_backend = config.settings.renderer_backend;
        // 动态链路合取判定（design D2）：编译期总闸 AND 运行时用户开关。
        // 运行时关闭时为用户侧软关闭——求值走 static_context()，
        // 与预览 IPC、动态性判定三处门控语义一致。
        let dynamic_input_active =
            crate::MATERIAL_DYNAMIC_INPUT_ENABLED && config.settings.material.dynamic_enabled;

        // 判断走新格式（layers）还是旧格式（crosshair）：
        // - 新格式：迁移后的 profile `crosshair` 恒为 None（见 migration.rs），
        //   故以 `crosshair.is_none()` 或 layers 非空作为新格式标志
        // - 旧格式：`crosshair = Some(...)` → 调用旧 build_shapes
        // 空 layers + None crosshair 走新路径 → build_layers_shapes 空迭代 → 渲染空白，
        // 与预览一致；纯 legacy 配置（crosshair=Some）走旧路径，行为不变。
        // MATERIAL_RUNTIME_ENABLED 门控：物料运行时已软关闭时强制走旧版 Crosshair 路径。
        let use_new_format = crate::MATERIAL_RUNTIME_ENABLED
            && profile
                .map(|p| !p.layers.is_empty() || p.crosshair.is_none())
                .unwrap_or(false);

        // 旧格式路径：克隆 crosshair，供 build_shapes 使用。
        let legacy_crosshair = if !use_new_format {
            let from_crosshair = profile.and_then(|p| p.crosshair.clone());
            from_crosshair
                .or_else(|| {
                    // 迁移后新格式可能仅含 layers、crosshair 为 None，
                    // 此时从 layers[0] 反向生成 Crosshair，确保旧版渲染路径
                    // 仍能显示用户配置，而非直接回退到默认准星。
                    profile
                        .and_then(|p| p.layers.first().and_then(crate::shapes::layer_to_crosshair))
                })
                .unwrap_or_else(Crosshair::default_crosshair)
        } else {
            // 新格式不使用 crosshair，但保留默认值用于 is_custom_image 检查。
            Crosshair::default_crosshair()
        };
        let profile_clone = profile.cloned();
        drop(config);

        // 在像素缓冲区上绘制准心。
        let logical_w = width as f32 / self.window.scale_factor() as f32;
        let logical_h = height as f32 / self.window.scale_factor() as f32;
        let rect = crate::shapes::RectF {
            min_x: 0.0,
            min_y: 0.0,
            max_x: logical_w,
            max_y: logical_h,
        };
        let scale = self.window.scale_factor() as f32;

        // CustomImage 需要访问 image_cache，单独处理。
        // 先加载图片（在获取 buffer 之前，避免与 surface 借用冲突）。
        // 新格式路径下，image 加载延迟到光栅化 Image 图元时处理。
        let is_custom_image =
            !use_new_format && legacy_crosshair.style == CrosshairStyle::CustomImage;
        if is_custom_image {
            self.ensure_image_loaded(&legacy_crosshair.image_path);
        }

        // 新格式路径：求值一次，供「指纹比对 + 图片预加载 + 光栅化」共用。
        // MATERIAL_RUNTIME_ENABLED 门控：仅在物料运行时启用时才进入此分支，
        // 软关闭时 use_new_format 编译期为 false，此分支不可达。
        //
        // 帧指纹跳绘：shapes 输出 + 尺寸 + 抗锯齿 + 后端 + 图片纪元的哈希与上一帧
        // 相同 → 本次帧内容必然与上帧逐像素一致，直接返回（不清屏 / 不光栅化）。
        // 时钟等低频动态物料的稳态成本由「每帧全量光栅化」降为「一次求值 + 一次
        // 哈希比较」。
        let mut new_format_shapes: Option<Vec<(peregrine_config::Element, [f32; 4], f32)>> = None;
        if use_new_format {
            let Some(ref profile) = profile_clone else {
                return Ok(());
            };
            // 动态上下文选择：MATERIAL_DYNAMIC_INPUT_ENABLED 门控。
            // 启用时轮询真实动态输入（Win32 鼠标键盘 / 时间）；
            // 停用时用 static_context()（动态物料冻结渲染）。
            let ctx = if dynamic_input_active {
                crate::platform::poll_dynamic_context(logical_w, logical_h)
            } else {
                peregrine_material::DynamicContext::static_context()
            };
            let shapes =
                crate::shapes::build_layers_shapes(&rect, profile, &self.material_registry, &ctx);

            // 图片预加载在指纹比对之前：路径首次出现时纪元递增会改变指纹，
            // 保证「首帧图片已就绪」参与判定，不会把缺图帧误判为可跳过。
            let image_paths: Vec<String> = shapes
                .iter()
                .filter_map(|(e, _, _)| match e {
                    peregrine_config::Element::Image { path, .. } => Some(path.clone()),
                    _ => None,
                })
                .collect();
            let epoch_before = self.image_cache_epoch;
            for path in image_paths {
                self.ensure_image_loaded(&path);
            }

            // 指纹：shapes 逐项哈希（Element 含全部几何/文本字段）+ 帧级输入。
            // f32 不实现 Hash（存在 NaN 语义问题），统一经 to_bits() 压成 u32。
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            for (element, color, opacity) in &shapes {
                hash_element(&mut hasher, element);
                hasher.write_u32(color[0].to_bits());
                hasher.write_u32(color[1].to_bits());
                hasher.write_u32(color[2].to_bits());
                hasher.write_u32(color[3].to_bits());
                hasher.write_u32(opacity.to_bits());
            }
            hasher.write_u32(width);
            hasher.write_u32(height);
            hasher.write_u32(scale.to_bits());
            antialiasing.hash(&mut hasher);
            renderer_backend.hash(&mut hasher);
            hasher.write_u64(self.image_cache_epoch);
            let fingerprint = hasher.finish();

            if fingerprint == self.last_frame_fingerprint && epoch_before == self.image_cache_epoch
            {
                // 与上一帧完全一致：跳过全部光栅化（稳态帧的主路径）。
                tracing::trace!("overlay frame skipped: identical fingerprint");
                return Ok(());
            }
            self.last_frame_fingerprint = fingerprint;
            new_format_shapes = Some(shapes);
        }

        let mut buffer = match self.surface.buffer_mut() {
            Ok(b) => b,
            Err(e) => {
                // 吞错点：buffer_mut 失败维持原 tracing::error! 行为，不上抛
                // （仅 surface.resize 失败对应 OVERLAY_RENDER，见函数文档）。
                tracing::error!("softbuffer buffer_mut failed: {}", e);
                return Ok(());
            }
        };

        // 清屏为完全透明。
        buffer.fill(0x00000000);

        // 诊断：检查 buffer 长度与预期是否一致。
        tracing::debug!(
            buf_len = buffer.len(),
            expected = (width as usize) * (height as usize),
            "overlay buffer size check"
        );

        if use_new_format {
            // ===== 新格式路径：遍历图层（shapes 已在上游求值一次，此处直接消费） =====
            let Some(shapes) = new_format_shapes else {
                return Ok(());
            };

            // 分离 Image 图元、CPU 光栅化图元、SVG 后端图元。
            // Image 由 CPU 直接 blit；Rect/Circle/Triangle 等由 CPU 光栅化；Text/Polygon/Line 由 SVG 后端光栅化。
            let mut image_elements: Vec<(peregrine_config::Element, [f32; 4], f32)> = Vec::new();
            let mut cpu_elements: Vec<(peregrine_config::Element, [f32; 4], f32)> = Vec::new();
            let mut svg_elements: Vec<(peregrine_config::Element, [f32; 4], f32)> = Vec::new();
            for (element, color, opacity) in shapes {
                match &element {
                    peregrine_config::Element::Image { .. } => {
                        image_elements.push((element, color, opacity));
                    }
                    peregrine_config::Element::Rect { .. }
                    | peregrine_config::Element::Circle { .. }
                    | peregrine_config::Element::CircleStroke { .. }
                    | peregrine_config::Element::DashedCircle { .. }
                    | peregrine_config::Element::Triangle { .. } => {
                        cpu_elements.push((element, color, opacity));
                    }
                    _ => {
                        svg_elements.push((element, color, opacity));
                    }
                }
            }

            // 1. CPU 光栅化（支持抗锯齿开关）。
            for (element, color, opacity) in cpu_elements {
                let color_u32 = make_color(&color, opacity);
                rasterize_shape(
                    &mut buffer,
                    width,
                    height,
                    scale,
                    &element,
                    color_u32,
                    antialiasing,
                );
            }

            // 2. SVG 后端光栅化（Text / Polygon / Line）。
            if !svg_elements.is_empty() {
                let ok = crate::svg_renderer::render_elements_to_buffer(
                    &mut buffer,
                    width,
                    height,
                    scale,
                    &rect,
                    &svg_elements,
                );
                if !ok {
                    tracing::warn!("SVG 光栅化失败，部分图元可能未显示");
                }
            }

            // 3. Image 图元（CPU 直接 blit，覆盖在最上层）。
            for (element, _color, opacity) in image_elements {
                if let peregrine_config::Element::Image { x, y, w, h, .. } = &element {
                    if let Some(img) = &self.image_cache {
                        draw_image_at_left_top(
                            &mut buffer,
                            width,
                            height,
                            scale,
                            *x,
                            *y,
                            *w,
                            *h,
                            img,
                            opacity,
                        );
                    }
                }
            }
        } else if is_custom_image {
            // 旧格式 CustomImage 路径（保留兼容）。
            if let Some(img) = &self.image_cache {
                let opacity = legacy_crosshair.opacity;
                let img_scale = legacy_crosshair.image_scale;
                let offset_x = legacy_crosshair.image_offset_x;
                let offset_y = legacy_crosshair.image_offset_y;
                draw_image(
                    &mut buffer,
                    width,
                    height,
                    scale,
                    &rect,
                    img,
                    img_scale,
                    offset_x,
                    offset_y,
                    opacity,
                );
            }
        } else if renderer_backend == RendererBackend::Svg {
            // SVG 后端：将图元转为 SVG 由 resvg/tiny-skia 光栅化。
            let ok = crate::svg_renderer::render_shapes_to_buffer(
                &mut buffer,
                width,
                height,
                scale,
                &rect,
                &legacy_crosshair,
            );
            if !ok {
                tracing::warn!("SVG 光栅化失败，回退到 CPU 渲染");
                let color = make_color(&legacy_crosshair.color, legacy_crosshair.opacity);
                let shapes = crate::shapes::build_shapes(&rect, &legacy_crosshair);
                for shape in shapes {
                    rasterize_shape(
                        &mut buffer,
                        width,
                        height,
                        scale,
                        &shape,
                        color,
                        antialiasing,
                    );
                }
            }
        } else {
            // CPU 后端：手写像素光栅化（旧格式路径，默认）。
            let color = make_color(&legacy_crosshair.color, legacy_crosshair.opacity);
            let shapes = crate::shapes::build_shapes(&rect, &legacy_crosshair);
            for shape in shapes {
                rasterize_shape(
                    &mut buffer,
                    width,
                    height,
                    scale,
                    &shape,
                    color,
                    antialiasing,
                );
            }
        }

        // 诊断：统计非透明像素数量。
        // tracing::debug! 的参数是惰性求值——enabled 为 false 时整段扫描不执行，
        // 消除稳态每帧一次的 8MB 全缓冲遍历。
        if tracing::enabled!(tracing::Level::DEBUG) {
            let non_transparent = buffer.iter().filter(|&&p| p != 0x00000000).count();
            tracing::debug!(
                non_transparent,
                total = buffer.len(),
                "overlay pixel stats after drawing"
            );
        }

        if let Err(e) = buffer.present() {
            tracing::error!("softbuffer present failed: {}", e);
        }

        Ok(())
    }

    /// 确保 image_cache 中缓存的是当前路径的图片。
    ///
    /// 如果路径为空或加载失败，清空缓存并记录警告。
    /// 图片（重）加载成功时递增 `image_cache_epoch`（帧指纹输入，
    /// 保证「路径相同但内容变化」的帧不被跳绘）。
    fn ensure_image_loaded(&mut self, path: &str) {
        // 路径未变且有缓存 → 无需重新加载。
        if let Some(cache) = &self.image_cache {
            if cache.path == path {
                return;
            }
        }

        if path.is_empty() {
            self.image_cache = None;
            return;
        }

        match load_png(path) {
            Ok((pixels, w, h)) => {
                tracing::info!(path, width = w, height = h, "loaded crosshair PNG");
                self.image_cache = Some(CachedImage {
                    path: path.to_string(),
                    pixels,
                    width: w,
                    height: h,
                });
                self.image_cache_epoch += 1;
            }
            Err(e) => {
                tracing::warn!(path, error = %e, "failed to load crosshair PNG");
                self.image_cache = None;
            }
        }
    }
}

/// 把 Element 的全部影响像素的字段喂入哈希器（帧指纹输入）。
///
/// `Element` 含 f32 字段而 f32 未实现 `Hash`（NaN 语义），故逐变体
/// 手写哈希：几何坐标统一 `to_bits()`（位模式稳定，NaN 也产生稳定指纹）。
/// 新增 Element 变体时必须同步补分支（漏字段 → 指纹漏变 → 漏重绘）。
fn hash_element<H: std::hash::Hasher>(h: &mut H, e: &peregrine_config::Element) {
    use peregrine_config::Element;
    use std::hash::Hash;
    // 变体判别值：不同变体即使字段相同指纹也必须不同。
    std::mem::discriminant(e).hash(h);
    // f32 → u32 位模式（f32 未实现 Hash；位模式稳定，NaN 也产生稳定指纹）。
    macro_rules! fb {
        ($v:expr) => {
            h.write_u32(($v).to_bits())
        };
    }
    match e {
        Element::Rect {
            x,
            y,
            w,
            h: rh,
            corner_radius,
        } => {
            fb!(*x);
            fb!(*y);
            fb!(*w);
            fb!(*rh);
            corner_radius.map(|v| v.to_bits()).hash(h);
        }
        Element::Circle { cx, cy, radius } => {
            fb!(*cx);
            fb!(*cy);
            fb!(*radius);
        }
        Element::CircleStroke {
            cx,
            cy,
            radius,
            thickness,
        } => {
            fb!(*cx);
            fb!(*cy);
            fb!(*radius);
            fb!(*thickness);
        }
        Element::DashedCircle {
            cx,
            cy,
            radius,
            thickness,
            dash_len,
            gap_len,
        } => {
            fb!(*cx);
            fb!(*cy);
            fb!(*radius);
            fb!(*thickness);
            fb!(*dash_len);
            fb!(*gap_len);
        }
        Element::Triangle {
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
        } => {
            fb!(*x1);
            fb!(*y1);
            fb!(*x2);
            fb!(*y2);
            fb!(*x3);
            fb!(*y3);
        }
        Element::Polygon { points } => {
            for p in points {
                fb!(p[0]);
                fb!(p[1]);
            }
        }
        Element::Line {
            x1,
            y1,
            x2,
            y2,
            thickness,
        } => {
            fb!(*x1);
            fb!(*y1);
            fb!(*x2);
            fb!(*y2);
            fb!(*thickness);
        }
        Element::Text {
            x,
            y,
            content,
            font_size,
            font_weight,
        } => {
            fb!(*x);
            fb!(*y);
            content.hash(h);
            fb!(*font_size);
            font_weight.hash(h);
        }
        Element::Image {
            path,
            x,
            y,
            w,
            h: ih,
        } => {
            path.hash(h);
            fb!(*x);
            fb!(*y);
            fb!(*w);
            fb!(*ih);
        }
        // Path：判别值 + fill + thickness + 逐段（cmd 判别值 + 坐标位模式）
        // + 两个 Option 覆盖色逐分量位模式。静止输入 → 指纹不变 → 跳帧。
        Element::Path {
            segments,
            fill,
            thickness,
            stroke_color,
            fill_color,
        } => {
            h.write_u8(*fill as u8);
            fb!(*thickness);
            for seg in segments {
                std::mem::discriminant(seg).hash(h);
                match *seg {
                    peregrine_config::PathSegment::M { x, y }
                    | peregrine_config::PathSegment::L { x, y } => {
                        fb!(x);
                        fb!(y);
                    }
                    peregrine_config::PathSegment::Q { x1, y1, x, y } => {
                        fb!(x1);
                        fb!(y1);
                        fb!(x);
                        fb!(y);
                    }
                    peregrine_config::PathSegment::C {
                        x1,
                        y1,
                        x2,
                        y2,
                        x,
                        y,
                    } => {
                        fb!(x1);
                        fb!(y1);
                        fb!(x2);
                        fb!(y2);
                        fb!(x);
                        fb!(y);
                    }
                    peregrine_config::PathSegment::Z => {}
                }
            }
            for color in [stroke_color.as_ref(), fill_color.as_ref()] {
                match color {
                    Some(c) => {
                        h.write_u8(1);
                        for ch in c {
                            fb!(*ch);
                        }
                    }
                    None => h.write_u8(0),
                }
            }
        }
    }
}

/// 将 PNG 图片绘制到指定左上角坐标 + 宽高（用于新格式 Image 图元）。
///
/// 与 `draw_image`（基于中心点）不同，这个函数直接用左上角坐标，
/// 简化 Element::Image 的渲染。
fn draw_image_at_left_top(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    img: &CachedImage,
    opacity: f32,
) {
    let scaled_w = w;
    let scaled_h = h;
    let _ = scaled_w;
    let _ = scaled_h;

    let px_start_x = (x * scale).round() as i32;
    let px_start_y = (y * scale).round() as i32;
    let px_w = (w * scale).round() as usize;
    let px_h = (h * scale).round() as usize;

    for py in 0..px_h {
        let dst_y = px_start_y + py as i32;
        if dst_y < 0 || dst_y >= pixel_h as i32 {
            continue;
        }
        for px in 0..px_w {
            let dst_x = px_start_x + px as i32;
            if dst_x < 0 || dst_x >= pixel_w as i32 {
                continue;
            }
            let src_x = (px as f32 / px_w as f32 * img.width as f32) as usize;
            let src_y = (py as f32 / px_h as f32 * img.height as f32) as usize;
            let src_x = src_x.min(img.width - 1);
            let src_y = src_y.min(img.height - 1);
            let (r, g, b, a) = img.pixels[src_y * img.width + src_x];

            let final_alpha = (a as f32 / 255.0 * opacity).clamp(0.0, 1.0);
            if final_alpha < 0.01 {
                continue;
            }

            let ai = (final_alpha * 255.0) as u32;
            let ri = (r as f32 * final_alpha) as u32;
            let gi = (g as f32 * final_alpha) as u32;
            let bi = (b as f32 * final_alpha) as u32;
            let pixel = (ai << 24) | (ri << 16) | (gi << 8) | bi;

            let idx = dst_y as usize * pixel_w as usize + dst_x as usize;
            if idx < buffer.len() {
                buffer[idx] = pixel;
            }
        }
    }
}

/// 把 [f32;4] RGBA + opacity 转换为预乘 alpha 的 0xAARRGGBB u32。
fn make_color(color: &[f32; 4], opacity: f32) -> u32 {
    let a = (color[3] * opacity).clamp(0.0, 1.0);
    let r = color[0] * a;
    let g = color[1] * a;
    let b = color[2] * a;
    let ai = (a * 255.0) as u32;
    let ri = (r * 255.0) as u32;
    let gi = (g * 255.0) as u32;
    let bi = (b * 255.0) as u32;
    (ai << 24) | (ri << 16) | (gi << 8) | bi
}

/// 将一条 [`Shape`]（共享几何图元）光栅化到 softbuffer 像素缓冲区。
///
/// 这是 overlay 侧的渲染器：与前端 `Preview` 组件的预览渲染一一对应。
/// 两者调用相同的 `build_shapes`，确保预览与实际效果完全一致。
fn rasterize_shape(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    shape: &crate::shapes::Shape,
    color: u32,
    antialiasing: bool,
) {
    use crate::shapes::Shape;
    match shape {
        Shape::Rect {
            x,
            y,
            w,
            h,
            corner_radius,
        } => {
            // 任务 9.5：CPU 光栅化圆角矩形实现复杂（需四角圆弧填充），
            // 当前降级为直角矩形。若 corner_radius > 0 则记录 warn 日志提示。
            if let Some(r) = corner_radius {
                if *r > 0.0 {
                    tracing::debug!(
                        r,
                        "CPU renderer does not support rounded rect; degrading to sharp corners"
                    );
                }
            }
            draw_rect(buffer, pixel_w, pixel_h, scale, *x, *y, *w, *h, color);
        }
        Shape::Circle { cx, cy, radius } => {
            if antialiasing {
                draw_circle(buffer, pixel_w, pixel_h, scale, *cx, *cy, *radius, color);
            } else {
                draw_circle_fast(buffer, pixel_w, pixel_h, scale, *cx, *cy, *radius, color);
            }
        }
        Shape::CircleStroke {
            cx,
            cy,
            radius,
            thickness,
        } => {
            if antialiasing {
                draw_circle_stroke(
                    buffer, pixel_w, pixel_h, scale, *cx, *cy, *radius, *thickness, color,
                );
            } else {
                draw_circle_stroke_fast(
                    buffer, pixel_w, pixel_h, scale, *cx, *cy, *radius, *thickness, color,
                );
            }
        }
        Shape::DashedCircle {
            cx,
            cy,
            radius,
            thickness,
            dash_len,
            gap_len,
        } => {
            if antialiasing {
                draw_dashed_circle(
                    buffer, pixel_w, pixel_h, scale, *cx, *cy, *radius, *thickness, *dash_len,
                    *gap_len, color,
                );
            } else {
                draw_dashed_circle_fast(
                    buffer, pixel_w, pixel_h, scale, *cx, *cy, *radius, *thickness, *dash_len,
                    *gap_len, color,
                );
            }
        }
        Shape::Triangle {
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
        } => {
            if antialiasing {
                draw_triangle(
                    buffer, pixel_w, pixel_h, scale, *x1, *y1, *x2, *y2, *x3, *y3, color,
                );
            } else {
                draw_triangle_fast(
                    buffer, pixel_w, pixel_h, scale, *x1, *y1, *x2, *y2, *x3, *y3, color,
                );
            }
        }
        Shape::Polygon { .. } | Shape::Line { .. } | Shape::Text { .. } => {
            // 旧 crosshair 路径下不渲染这三类图元（旧版 build_shapes 不产出）；
            // 新格式路径由 SVG 后端光栅化。
            tracing::debug!("rasterize_shape: 该图元类型在旧 crosshair 路径下不渲染");
        }
        Shape::Image { x, y, w, h, path } => {
            // 旧 crosshair 路径下的 CustomImage 由上层专门分支处理；
            // 此处仅记录路径，不直接 blit。
            let _ = (x, y, w, h, path);
        }
        // Path 不实现 CPU softbuffer 直绘（与 Text/Polygon/Line 同策略）：
        // 新格式路径中 Path 已在上游分流到 SVG 后端图元集合，
        // 此 arm 仅满足穷尽性，到达即为路由错误，记录 warn。
        Shape::Path { .. } => {
            tracing::warn!("rasterize_shape: Path element reached CPU path (expected SVG backend)");
        }
    }
}

/// 把颜色分量按覆盖率混合写入像素缓冲区（前景预乘 alpha，背景透明）。
///
/// 由于覆盖层背景始终透明（0x00000000），混合结果为预乘值：
/// `out = fg_premul * coverage + bg * (1 - coverage)`。
/// 背景为 0 时简化为 `out = fg_premul * coverage`。
fn blend_pixel(buffer: &mut [u32], idx: usize, color: u32, coverage: f32) {
    if coverage <= 0.0 || idx >= buffer.len() {
        return;
    }
    let cov = coverage.min(1.0);
    // 颜色已经是预乘 alpha 格式，直接按覆盖率缩放各分量。
    let ai = ((color >> 24) & 0xFF) as f32 * cov;
    let ri = ((color >> 16) & 0xFF) as f32 * cov;
    let gi = ((color >> 8) & 0xFF) as f32 * cov;
    let bi = (color & 0xFF) as f32 * cov;
    buffer[idx] = ((ai as u32) << 24) | ((ri as u32) << 16) | ((gi as u32) << 8) | (bi as u32);
}

/// 绘制填充三角形（逻辑坐标，边距离抗锯齿）。
///
/// 使用三条边的有符号距离（重心坐标）判断像素在三角形内/外的程度：
/// 三条边线函数同号 → 内部；覆盖率取最接近 0 的边线值的平滑映射，
/// 在边缘 1 像素范围内平滑过渡。
fn draw_triangle(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    color: u32,
) {
    // 转换到物理像素坐标。
    let (px1, py1) = (x1 * scale, y1 * scale);
    let (px2, py2) = (x2 * scale, y2 * scale);
    let (px3, py3) = (x3 * scale, y3 * scale);

    // 包围盒。
    let min_x = px1.min(px2).min(px3).floor() as i32;
    let max_x = px1.max(px2).max(px3).ceil() as i32;
    let min_y = py1.min(py2).min(py3).floor() as i32;
    let max_y = py1.max(py2).max(py3).ceil() as i32;

    let x0 = min_x.max(0);
    let y0 = min_y.max(0);
    let x1_clip = (max_x + 1).min(pixel_w as i32);
    let y1_clip = (max_y + 1).min(pixel_h as i32);

    // 三角形面积（2 倍），用于确定绕序方向。
    let area = (px2 - px1) * (py3 - py1) - (px3 - px1) * (py2 - py1);
    if area.abs() < 0.01 {
        return;
    }
    // sign > 0 表示逆时针，sign < 0 表示顺时针。
    // 标准化：使所有内部像素的 w0/w1/w2 ≥ 0。
    let sign = if area > 0.0 { 1.0 } else { -1.0 };

    // 三条边的向量长度，用于将边线函数值归一化为像素距离。
    let len_e0 = ((px3 - px2).powi(2) + (py3 - py2).powi(2)).sqrt();
    let len_e1 = ((px1 - px3).powi(2) + (py1 - py3).powi(2)).sqrt();
    let len_e2 = ((px2 - px1).powi(2) + (py2 - py1).powi(2)).sqrt();

    for py in y0..y1_clip {
        for px in x0..x1_clip {
            let pxc = px as f32 + 0.5;
            let pyc = py as f32 + 0.5;
            // 三条边线函数（乘以 sign 使内部为正）。
            let w0 = sign * ((px2 - pxc) * (py3 - pyc) - (px3 - pxc) * (py2 - pyc));
            let w1 = sign * ((px3 - pxc) * (py1 - pyc) - (px1 - pxc) * (py3 - pyc));
            let w2 = sign * ((px1 - pxc) * (py2 - pyc) - (px2 - pxc) * (py1 - pyc));

            // 完全在外（任一边线为负且距离 > 1px）→ 跳过。
            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                // 近似：如果最大负值对应边线距离 > 1 像素则跳过。
                let min_w = w0.min(w1).min(w2);
                let max_len = len_e0.max(len_e1).max(len_e2);
                if min_w < -max_len {
                    continue;
                }
            }

            // 将边线函数归一化为到边的距离（除以边长）。
            let d0 = w0 / len_e0;
            let d1 = w1 / len_e1;
            let d2 = w2 / len_e2;

            // 覆盖率 = 最小边距的平滑映射（在 0~1px 过渡）。
            let min_d = d0.min(d1).min(d2);
            let coverage = (min_d + 0.5).clamp(0.0, 1.0);

            if coverage > 0.0 {
                let idx = (py as u32) * pixel_w + (px as u32);
                blend_pixel(buffer, idx as usize, color, coverage);
            }
        }
    }
}

/// 绘制填充矩形（逻辑坐标）。
fn draw_rect(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: u32,
) {
    let x0 = (x * scale).round() as i32;
    let y0 = (y * scale).round() as i32;
    let x1 = ((x + w) * scale).round() as i32;
    let y1 = ((y + h) * scale).round() as i32;
    let x0 = x0.max(0);
    let y0 = y0.max(0);
    let x1 = x1.min(pixel_w as i32);
    let y1 = y1.min(pixel_h as i32);
    for py in y0..y1 {
        for px in x0..x1 {
            let idx = (py as u32) * pixel_w + (px as u32);
            if (idx as usize) < buffer.len() {
                buffer[idx as usize] = color;
            }
        }
    }
}

/// 绘制填充圆（逻辑坐标，距离场抗锯齿）。
///
/// 使用像素中心到圆心的距离与半径的关系计算覆盖率：
/// `coverage = clamp(radius + 0.5 - dist, 0, 1)`，
/// 在边缘 1 像素范围内平滑过渡，消除锯齿。
fn draw_circle(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    cx: f32,
    cy: f32,
    radius: f32,
    color: u32,
) {
    let pcx = cx * scale;
    let pcy = cy * scale;
    let pr = radius * scale;
    // 包围盒扩展 1px 以覆盖抗锯齿过渡区域。
    let x0 = ((pcx - pr - 1.0).floor() as i32).max(0);
    let y0 = ((pcy - pr - 1.0).floor() as i32).max(0);
    let x1 = ((pcx + pr + 1.0).ceil() as i32).min(pixel_w as i32);
    let y1 = ((pcy + pr + 1.0).ceil() as i32).min(pixel_h as i32);
    for py in y0..y1 {
        for px in x0..x1 {
            let dx = px as f32 + 0.5 - pcx;
            let dy = py as f32 + 0.5 - pcy;
            let dist = (dx * dx + dy * dy).sqrt();
            let coverage = (pr + 0.5 - dist).clamp(0.0, 1.0);
            if coverage > 0.0 {
                let idx = (py as u32) * pixel_w + (px as u32);
                blend_pixel(buffer, idx as usize, color, coverage);
            }
        }
    }
}

/// 绘制圆环描边（逻辑坐标，距离场抗锯齿）。
///
/// 使用圆环的 SDF（有符号距离场）计算覆盖率：
/// `sdf = |dist - center_r| - half_thickness`
/// `coverage = clamp(0.5 - sdf, 0, 1)`，
/// 在内外边缘各 1 像素范围内平滑过渡。
fn draw_circle_stroke(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    cx: f32,
    cy: f32,
    radius: f32,
    thickness: f32,
    color: u32,
) {
    let pcx = cx * scale;
    let pcy = cy * scale;
    let center_r = radius * scale;
    let half_t = thickness * scale / 2.0;
    let outer_r = center_r + half_t;
    // 包围盒扩展 1px 以覆盖抗锯齿过渡区域。
    let x0 = ((pcx - outer_r - 1.0).floor() as i32).max(0);
    let y0 = ((pcy - outer_r - 1.0).floor() as i32).max(0);
    let x1 = ((pcx + outer_r + 1.0).ceil() as i32).min(pixel_w as i32);
    let y1 = ((pcy + outer_r + 1.0).ceil() as i32).min(pixel_h as i32);
    for py in y0..y1 {
        for px in x0..x1 {
            let dx = px as f32 + 0.5 - pcx;
            let dy = py as f32 + 0.5 - pcy;
            let dist = (dx * dx + dy * dy).sqrt();
            // 圆环 SDF：到中心半径的距离减去半厚度。
            let sdf = (dist - center_r).abs() - half_t;
            let coverage = (0.5 - sdf).clamp(0.0, 1.0);
            if coverage > 0.0 {
                let idx = (py as u32) * pixel_w + (px as u32);
                blend_pixel(buffer, idx as usize, color, coverage);
            }
        }
    }
}

/// 绘制虚线圆（逻辑坐标）。
fn draw_dashed_circle(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    cx: f32,
    cy: f32,
    radius: f32,
    thickness: f32,
    dash_len: f32,
    gap_len: f32,
    color: u32,
) {
    let circumference = 2.0 * std::f32::consts::PI * radius;
    let unit = dash_len + gap_len;
    let segments = (circumference / unit).ceil() as usize;
    let step_angle = 2.0 * std::f32::consts::PI / segments as f32;
    let dash_angle = step_angle * (dash_len / unit);
    for i in 0..segments {
        let a0 = i as f32 * step_angle;
        let a1 = a0 + dash_angle;
        // 在 [a0, a1] 范围内逐角度采样绘制。
        let steps = ((a1 - a0) * radius).ceil() as usize + 1;
        for s in 0..steps {
            let t = if steps > 0 {
                s as f32 / steps as f32
            } else {
                0.0
            };
            let a = a0 + (a1 - a0) * t;
            let x = cx + radius * a.cos();
            let y = cy + radius * a.sin();
            draw_circle(
                buffer,
                pixel_w,
                pixel_h,
                scale,
                x,
                y,
                thickness / 2.0,
                color,
            );
        }
    }
}

// ===== 关闭抗锯齿时的快速路径（硬二值光栅化，无 sqrt） =====

/// 绘制填充圆（无抗锯齿）。
fn draw_circle_fast(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    cx: f32,
    cy: f32,
    radius: f32,
    color: u32,
) {
    let pcx = (cx * scale).round() as i32;
    let pcy = (cy * scale).round() as i32;
    let pr = (radius * scale).round() as i32;
    let pr_sq = (pr * pr) as f32;
    let x0 = (pcx - pr).max(0);
    let y0 = (pcy - pr).max(0);
    let x1 = (pcx + pr).min(pixel_w as i32);
    let y1 = (pcy + pr).min(pixel_h as i32);
    for py in y0..y1 {
        for px in x0..x1 {
            let dx = px - pcx;
            let dy = py - pcy;
            if (dx * dx + dy * dy) as f32 <= pr_sq {
                let idx = (py as u32) * pixel_w + (px as u32);
                if (idx as usize) < buffer.len() {
                    buffer[idx as usize] = color;
                }
            }
        }
    }
}

/// 绘制圆环描边（无抗锯齿）。
fn draw_circle_stroke_fast(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    cx: f32,
    cy: f32,
    radius: f32,
    thickness: f32,
    color: u32,
) {
    let pcx = (cx * scale).round() as i32;
    let pcy = (cy * scale).round() as i32;
    let outer_r = ((radius + thickness / 2.0) * scale).round() as i32;
    let inner_r = ((radius - thickness / 2.0) * scale).round().max(0.0) as i32;
    let outer_sq = (outer_r * outer_r) as f32;
    let inner_sq = (inner_r * inner_r) as f32;
    let x0 = (pcx - outer_r).max(0);
    let y0 = (pcy - outer_r).max(0);
    let x1 = (pcx + outer_r).min(pixel_w as i32);
    let y1 = (pcy + outer_r).min(pixel_h as i32);
    for py in y0..y1 {
        for px in x0..x1 {
            let dx = px - pcx;
            let dy = py - pcy;
            let d_sq = (dx * dx + dy * dy) as f32;
            if d_sq <= outer_sq && d_sq >= inner_sq {
                let idx = (py as u32) * pixel_w + (px as u32);
                if (idx as usize) < buffer.len() {
                    buffer[idx as usize] = color;
                }
            }
        }
    }
}

/// 绘制虚线圆（无抗锯齿）。
fn draw_dashed_circle_fast(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    cx: f32,
    cy: f32,
    radius: f32,
    thickness: f32,
    dash_len: f32,
    gap_len: f32,
    color: u32,
) {
    let circumference = 2.0 * std::f32::consts::PI * radius;
    let unit = dash_len + gap_len;
    let segments = (circumference / unit).ceil() as usize;
    let step_angle = 2.0 * std::f32::consts::PI / segments as f32;
    let dash_angle = step_angle * (dash_len / unit);
    for i in 0..segments {
        let a0 = i as f32 * step_angle;
        let a1 = a0 + dash_angle;
        let steps = ((a1 - a0) * radius).ceil() as usize + 1;
        for s in 0..steps {
            let t = if steps > 0 {
                s as f32 / steps as f32
            } else {
                0.0
            };
            let a = a0 + (a1 - a0) * t;
            let x = cx + radius * a.cos();
            let y = cy + radius * a.sin();
            draw_circle_fast(
                buffer,
                pixel_w,
                pixel_h,
                scale,
                x,
                y,
                thickness / 2.0,
                color,
            );
        }
    }
}

/// 绘制填充三角形（无抗锯齿，重心坐标法）。
fn draw_triangle_fast(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    color: u32,
) {
    let (px1, py1) = (x1 * scale, y1 * scale);
    let (px2, py2) = (x2 * scale, y2 * scale);
    let (px3, py3) = (x3 * scale, y3 * scale);
    let min_x = px1.min(px2).min(px3).floor() as i32;
    let max_x = px1.max(px2).max(px3).ceil() as i32;
    let min_y = py1.min(py2).min(py3).floor() as i32;
    let max_y = py1.max(py2).max(py3).ceil() as i32;
    let x0 = min_x.max(0);
    let y0 = min_y.max(0);
    let x1_clip = max_x.min(pixel_w as i32);
    let y1_clip = max_y.min(pixel_h as i32);
    let area = (px2 - px1) * (py3 - py1) - (px3 - px1) * (py2 - py1);
    if area.abs() < 0.01 {
        return;
    }
    for py in y0..y1_clip {
        for px in x0..x1_clip {
            let pxc = px as f32 + 0.5;
            let pyc = py as f32 + 0.5;
            let w0 = (px2 - pxc) * (py3 - pyc) - (px3 - pxc) * (py2 - pyc);
            let w1 = (px3 - pxc) * (py1 - pyc) - (px1 - pxc) * (py3 - pyc);
            let w2 = (px1 - pxc) * (py2 - pyc) - (px2 - pxc) * (py1 - pyc);
            let inside =
                (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0);
            if inside {
                let idx = (py as u32) * pixel_w + (px as u32);
                if (idx as usize) < buffer.len() {
                    buffer[idx as usize] = color;
                }
            }
        }
    }
}

// ===== PNG 图片加载与绘制 =====

/// 加载 PNG 文件，解码为 RGBA 像素向量。
///
/// 返回 (pixels, width, height)，pixels 为行优先从上到下的 RGBA 元组。
#[allow(clippy::type_complexity)]
fn load_png(
    path: &str,
) -> Result<(Vec<(u8, u8, u8, u8)>, usize, usize), Box<dyn std::error::Error>> {
    let decoder = png::Decoder::new(std::fs::File::open(path)?);
    let mut reader = decoder.read_info()?;
    let info = reader.info().clone();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut buf)?;
    let bytes = &buf[..frame.buffer_size()];

    let w = info.width as usize;
    let h = info.height as usize;

    // 根据 PNG 的颜色类型转换为统一的 RGBA 元组。
    let pixels: Vec<(u8, u8, u8, u8)> = match info.color_type {
        png::ColorType::Rgba => bytes
            .chunks_exact(4)
            .map(|c| (c[0], c[1], c[2], c[3]))
            .collect(),
        png::ColorType::Rgb => {
            // RGB 无 alpha，默认不透明。
            bytes
                .chunks_exact(3)
                .map(|c| (c[0], c[1], c[2], 255))
                .collect()
        }
        png::ColorType::Grayscale => bytes.iter().map(|&v| (v, v, v, 255)).collect(),
        png::ColorType::GrayscaleAlpha => bytes
            .chunks_exact(2)
            .map(|c| (c[0], c[0], c[0], c[1]))
            .collect(),
        png::ColorType::Indexed => {
            // 调色板模式：reader 已将输出转为 RGBA（png crate 的 output 转换），
            // 但如果 output 仍为 indexed，则按 RGB 处理。
            bytes
                .chunks_exact(3)
                .map(|c| (c[0], c[1], c[2], 255))
                .collect()
        }
    };

    Ok((pixels, w, h))
}

/// 将 PNG 图片按缩放比例绘制到 softbuffer 像素缓冲区。
///
/// - `img_scale`：图片缩放比例（1.0 = 原始大小，逻辑像素）。
/// - `offset_x`/`offset_y`：相对屏幕中心的偏移（逻辑像素）。
/// - `opacity`：全局不透明度（与图片 alpha 相乘）。
fn draw_image(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    rect: &crate::shapes::RectF,
    img: &CachedImage,
    img_scale: f32,
    offset_x: f32,
    offset_y: f32,
    opacity: f32,
) {
    // 缩放后的逻辑尺寸。
    let scaled_w = img.width as f32 * img_scale;
    let scaled_h = img.height as f32 * img_scale;

    // 图片中心在屏幕中心 + 偏移。
    let center_x = rect.center_x() + offset_x;
    let center_y = rect.center_y() + offset_y;

    // 图片左上角的物理像素坐标。
    let px_start_x = ((center_x - scaled_w / 2.0) * scale).round() as i32;
    let px_start_y = ((center_y - scaled_h / 2.0) * scale).round() as i32;
    // 图片覆盖的物理像素尺寸。
    let px_w = (scaled_w * scale).round() as usize;
    let px_h = (scaled_h * scale).round() as usize;

    for py in 0..px_h {
        let dst_y = px_start_y + py as i32;
        if dst_y < 0 || dst_y >= pixel_h as i32 {
            continue;
        }
        for px in 0..px_w {
            let dst_x = px_start_x + px as i32;
            if dst_x < 0 || dst_x >= pixel_w as i32 {
                continue;
            }
            // 将物理像素映射回原图坐标（最近邻采样）。
            let src_x = (px as f32 / px_w as f32 * img.width as f32) as usize;
            let src_y = (py as f32 / px_h as f32 * img.height as f32) as usize;
            let src_x = src_x.min(img.width - 1);
            let src_y = src_y.min(img.height - 1);
            let (r, g, b, a) = img.pixels[src_y * img.width + src_x];

            let final_alpha = (a as f32 / 255.0 * opacity).clamp(0.0, 1.0);
            if final_alpha < 0.01 {
                continue;
            }

            // 预乘 alpha 的 0xAARRGGBB 格式。
            let ai = (final_alpha * 255.0) as u32;
            let ri = (r as f32 * final_alpha) as u32;
            let gi = (g as f32 * final_alpha) as u32;
            let bi = (b as f32 * final_alpha) as u32;
            let pixel = (ai << 24) | (ri << 16) | (gi << 8) | bi;

            let idx = dst_y as usize * pixel_w as usize + dst_x as usize;
            if idx < buffer.len() {
                buffer[idx] = pixel;
            }
        }
    }
}
