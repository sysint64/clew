use cosmic_text::{Buffer, FontSystem};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::sync::Arc;
use tech_paws_ui::{
    Border, BorderRadius, ColorRgb, ColorRgba, Gradient, Rect, TileMode, View,
    assets::Assets,
    render::{Fill, RenderCommand, RenderState, Renderer},
    text::{FontResources, TextsResources},
};
use vello::{
    AaConfig, Glyph, RenderParams, RendererOptions, Scene,
    kurbo::{Affine, RoundedRect, RoundedRectRadii},
    peniko::{
        self, Blob, Brush, Color, Fill as VelloFill, FontData, Gradient as VelloGradient, StyleRef,
    },
    util::RenderContext,
    wgpu,
};

pub struct VelloRenderer {
    render_cx: RenderContext,
    surface: Option<vello::util::RenderSurface<'static>>,
    renderer: Option<vello::Renderer>,
    scene: Scene,

    // Glyphon text rendering
    // font_system: FontSystem,
    // swash_cache: SwashCache,
    // text_cache: Cache,
    // text_atlas: Option<TextAtlas>,
    // text_renderer: Option<TextRenderer>,
    // viewport: Viewport,
    current_width: u32,
    current_height: u32,
}

impl VelloRenderer {
    pub async fn new<W>(window: Arc<W>, width: u32, height: u32) -> Self
    where
        W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static,
    {
        let mut render_cx = RenderContext::new();

        // Create the surface
        let surface = render_cx
            .create_surface(window.clone(), width, height, wgpu::PresentMode::AutoVsync)
            .await
            .expect("Failed to create surface");

        let device = &render_cx.devices[surface.dev_id].device;

        // Create Vello renderer
        let renderer = vello::Renderer::new(device, RendererOptions::default())
            .expect("Failed to create Vello renderer");

        Self {
            render_cx,
            surface: Some(surface),
            renderer: Some(renderer),
            scene: Scene::new(),

            current_width: width,
            current_height: height,
        }
    }

    /// Resize the renderer surface
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        if self.current_width == width && self.current_height == height {
            return;
        }

        self.current_width = width;
        self.current_height = height;

        if let Some(surface) = &mut self.surface {
            self.render_cx.resize_surface(surface, width, height);
        }
    }

    /// Begin a new frame
    pub fn begin_frame(&mut self) {
        self.scene.reset();
    }

    /// End frame and present
    pub fn end_frame(&mut self, fill_color: &ColorRgb) {
        let Some(surface) = &self.surface else { return };
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        let device = &self.render_cx.devices[surface.dev_id].device;
        let queue = &self.render_cx.devices[surface.dev_id].queue;

        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("Failed to get surface texture");

        #[cfg(target_os = "macos")]
        #[allow(invalid_reference_casting)]
        unsafe {
            if let Some(hal_surface) = surface.surface.as_hal::<wgpu::hal::api::Metal>() {
                let raw = (&*hal_surface) as *const wgpu::hal::metal::Surface
                    as *mut wgpu::hal::metal::Surface;
                (*raw).present_with_transaction = true;
            }
        }

        let render_params = RenderParams {
            base_color: convert_rgb_color(fill_color),
            width: self.current_width,
            height: self.current_height,
            antialiasing_method: AaConfig::Msaa16,
        };

        renderer
            // .render_to_texture(device, queue, &self.scene, &surface_texture, &render_params)
            .render_to_texture(
                device,
                queue,
                &self.scene,
                &surface.target_view,
                &render_params,
            )
            .expect("Failed to render to surface");

        // surface_texture.present();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Surface Blit"),
        });
        surface.blitter.copy(
            &device,
            &mut encoder,
            &surface.target_view,
            &surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );
        queue.submit([encoder.finish()]);
        // Queue the texture to be presented on the surface
        surface_texture.present();

        device.poll(wgpu::PollType::Poll).unwrap();
    }

    /// Draw a filled rectangle
    pub fn draw_rect(
        &mut self,
        boundary: Rect,
        fill: Option<&Fill>,
        border_radius: Option<&BorderRadius>,
        border: Option<&Border>,
    ) {
        let rect = vello::kurbo::Rect::new(
            boundary.x as f64,
            boundary.y as f64,
            (boundary.x + boundary.width) as f64,
            (boundary.y + boundary.height) as f64,
        );

        let shape = if let Some(br) = border_radius {
            RoundedRect::from_rect(
                rect,
                RoundedRectRadii::new(
                    br.top_left as f64,
                    br.top_right as f64,
                    br.bottom_right as f64,
                    br.bottom_left as f64,
                ),
            )
        } else {
            RoundedRect::from_rect(rect, 0.0)
        };

        if let Some(fill) = fill {
            if let Some(brush) = create_brush_from_fill(fill, boundary) {
                self.scene
                    .fill(VelloFill::NonZero, Affine::IDENTITY, &brush, None, &shape);
            }
        }

        // TODO: Handle borders
    }

    /// Draw an oval/ellipse
    pub fn draw_oval(&mut self, boundary: Rect, fill: Option<&Fill>) {
        let ellipse = vello::kurbo::Ellipse::new(
            (
                (boundary.x + boundary.width / 2.0) as f64,
                (boundary.y + boundary.height / 2.0) as f64,
            ),
            (
                (boundary.width / 2.0) as f64,
                (boundary.height / 2.0) as f64,
            ),
            0.0,
        );

        if let Some(fill) = fill {
            if let Some(brush) = create_brush_from_fill(fill, boundary) {
                self.scene
                    .fill(VelloFill::NonZero, Affine::IDENTITY, &brush, None, &ellipse);
            }
        }
    }
}

// Helper functions

fn convert_rgba_color(color: &ColorRgba) -> Color {
    Color::from_rgba8(
        (color.r * 255.) as u8,
        (color.g * 255.) as u8,
        (color.b * 255.) as u8,
        (color.a * 255.) as u8,
    )
}

fn convert_rgb_color(color: &ColorRgb) -> Color {
    Color::from_rgb8(
        (color.r * 255.) as u8,
        (color.g * 255.) as u8,
        (color.b * 255.) as u8,
    )
}

fn create_brush_from_fill(fill: &Fill, rect: Rect) -> Option<Brush> {
    match fill {
        Fill::None => None,
        Fill::Color(color) => Some(Brush::Solid(convert_rgba_color(color))),
        Fill::Gradient(gradient) => create_gradient_brush(gradient, rect),
    }
}

fn create_gradient_brush(gradient: &Gradient, rect: Rect) -> Option<Brush> {
    match gradient {
        Gradient::Linear(linear) => {
            let start_x = rect.x + linear.start.0 * rect.width;
            let start_y = rect.y + linear.start.1 * rect.height;
            let end_x = rect.x + linear.end.0 * rect.width;
            let end_y = rect.y + linear.end.1 * rect.height;

            let stops: Vec<peniko::ColorStop> = linear
                .stops
                .iter()
                .map(|stop| peniko::ColorStop {
                    offset: stop.offset,
                    color: convert_rgba_color(&stop.color).into(),
                })
                .collect();

            let grad = VelloGradient::new_linear(
                (start_x as f64, start_y as f64),
                (end_x as f64, end_y as f64),
            )
            .with_stops(stops.as_slice());

            Some(Brush::Gradient(grad))
        }
        Gradient::Radial(radial) => {
            let center_x = rect.x + radial.center.0 * rect.width;
            let center_y = rect.y + radial.center.1 * rect.height;
            let radius = radial.radius * rect.width.max(rect.height);

            let stops: Vec<peniko::ColorStop> = radial
                .stops
                .iter()
                .map(|stop| peniko::ColorStop {
                    offset: stop.offset,
                    color: convert_rgba_color(&stop.color).into(),
                })
                .collect();

            let grad = VelloGradient::new_radial((center_x as f64, center_y as f64), radius)
                .with_stops(stops.as_slice());

            Some(Brush::Gradient(grad))
        }
        Gradient::Sweep(sweep) => {
            // Vello supports sweep gradients
            let center_x = rect.x + sweep.center.0 * rect.width;
            let center_y = rect.y + sweep.center.1 * rect.height;

            let stops: Vec<peniko::ColorStop> = sweep
                .stops
                .iter()
                .map(|stop| peniko::ColorStop {
                    offset: stop.offset,
                    color: convert_rgba_color(&stop.color).into(),
                })
                .collect();

            let grad =
                VelloGradient::new_sweep((center_x, center_y), sweep.start_angle, sweep.end_angle)
                    .with_stops(stops.as_slice());

            Some(Brush::Gradient(grad))
        }
    }
}

/// Draw a cosmic_text Buffer to a Vello Scene
pub fn draw_text(
    scene: &mut Scene,
    font_system: &mut FontSystem,
    buffer: &Buffer,
    x: f32,
    y: f32,
    color: Color,
) {
    let brush = Brush::Solid(color);

    for run in buffer.layout_runs() {
        let line_y = y + run.line_y;

        for glyph in run.glyphs.iter() {
            let physical = glyph.physical((x, line_y), 1.0);

            // Get the font data from cosmic_text's font system
            let font_id = glyph.font_id;
            if let Some(font) = font_system.get_font(font_id) {
                let font_data = font.data();

                // Create Vello FontData from raw bytes
                let vello_font = FontData::new(Blob::new(Arc::new(font_data.to_vec())), 0);

                let font_size = f32::from_bits(physical.cache_key.font_size_bits);

                // Create a Vello Glyph
                let vello_glyph = Glyph {
                    id: physical.cache_key.glyph_id as u32,
                    x: 0.0,
                    y: 0.0,
                };

                // Draw the glyph using Vello 0.6 API
                scene
                    .draw_glyphs(&vello_font)
                    .font_size(font_size)
                    .transform(Affine::translate((physical.x as f64, physical.y as f64)))
                    .brush(&brush)
                    .draw(
                        StyleRef::Fill(peniko::Fill::NonZero),
                        [vello_glyph].into_iter(),
                    );
            }
        }
    }
}

impl Renderer for VelloRenderer {
    fn process_commands(
        &mut self,
        view: &View,
        state: &RenderState,
        fill_color: ColorRgb,
        fonts: &mut FontResources,
        text: &mut TextsResources,
        assets: &Assets,
    ) {
        profiling::scope!("Tech Paws UI - Vello - Render");

        let width = view.size.width;
        let height = view.size.height;

        self.resize(width, height);
        self.begin_frame();

        for command in state.commands() {
            match command {
                RenderCommand::Rect {
                    boundary,
                    fill,
                    border_radius,
                    border,
                    ..
                } => {
                    self.draw_rect(
                        *boundary,
                        fill.as_ref(),
                        border_radius.as_ref(),
                        border.as_ref(),
                    );
                }
                RenderCommand::Oval {
                    boundary,
                    fill,
                    border,
                    ..
                } => {
                    self.draw_oval(*boundary, fill.as_ref());
                    // TODO: Handle oval borders
                }
                RenderCommand::Text {
                    x,
                    y,
                    text_id,
                    tint_color,
                    ..
                } => {
                    // TODO: Implement text rendering with Glyphon
                }
                RenderCommand::PushClipRect(rect) => {
                    // TODO: Implement clipping
                }
                RenderCommand::PopClip => {
                    // TODO: Implement clip pop
                }
                RenderCommand::Svg {
                    boundary,
                    asset_id,
                    tint_color,
                    ..
                } => {
                    // TODO: Implement SVG rendering with vello_svg or usvg
                }
            }
        }

        self.end_frame(&fill_color);

        tracy_client::frame_mark();
    }
}

// /// Alternative: Batch glyphs by font for better performance
// pub fn draw_text_batched(
//     scene: &mut Scene,
//     font_system: &mut FontSystem,
//     buffer: &Buffer,
//     x: f32,
//     y: f32,
//     color: Color,
// ) {
//     use std::collections::HashMap;

//     let brush = Brush::Solid(color);

//     // Group glyphs by font for batched rendering
//     let mut font_glyphs: HashMap<cosmic_text::fontdb::ID, Vec<(PhysicalGlyph, f32)>> = HashMap::new();

//     for run in buffer.layout_runs() {
//         let line_y = y + run.line_y;

//         for glyph in run.glyphs.iter() {
//             let physical = glyph.physical((x, line_y), 1.0);
//             let font_size = physical.cache_key.font_size_bits as f32;

//             font_glyphs
//                 .entry(glyph.font_id)
//                 .or_default()
//                 .push((physical, font_size));
//         }
//     }

//     // Render each font's glyphs in a batch
//     for (font_id, glyphs) in font_glyphs {
//         if let Some(font) = font_system.get_font(font_id) {
//             let font_data = font.data();

//             if let Ok(file_ref) = FileRef::new(font_data) {
//                 let font_ref = match file_ref {
//                     FileRef::Font(f) => Some(f),
//                     FileRef::Collection(c) => c.get(font_id.index() as u32).ok(),
//                 };

//                 if let Some(font_ref) = font_ref {
//                     // Assuming uniform font size within a buffer (common case)
//                     let font_size = glyphs.first().map(|(_, s)| *s).unwrap_or(16.0);

//                     let glyph_iter = glyphs.iter().map(|(physical, _)| {
//                         let glyph_id = vello::peniko::GlyphId::new(physical.cache_key.glyph_id as u32);
//                         (glyph_id, physical.x as f32, physical.y as f32)
//                     });

//                     scene
//                         .draw_glyphs(&font_ref)
//                         .font_size(font_size)
//                         .brush(&brush)
//                         .fill(Fill::NonZero)
//                         .draw(glyph_iter);
//                 }
//             }
//         }
//     }
// }

// /// Even simpler: if you manage fonts yourself
// pub fn draw_text_simple(
//     scene: &mut Scene,
//     font_data: &[u8],
//     buffer: &Buffer,
//     x: f32,
//     y: f32,
//     color: Color,
// ) {
//     let brush = Brush::Solid(color);

//     let Ok(file_ref) = FileRef::new(font_data) else { return };
//     let Some(font_ref) = (match file_ref {
//         FileRef::Font(f) => Some(f),
//         FileRef::Collection(c) => c.get(0).ok(),
//     }) else { return };

//     for run in buffer.layout_runs() {
//         let line_y = y + run.line_y;

//         let glyphs = run.glyphs.iter().map(|glyph| {
//             let physical = glyph.physical((x, line_y), 1.0);
//             let glyph_id = vello::peniko::GlyphId::new(physical.cache_key.glyph_id as u32);
//             (glyph_id, physical.x as f32, physical.y as f32)
//         });

//         // Get font size from first glyph in run
//         if let Some(first) = run.glyphs.first() {
//             let physical = first.physical((x, line_y), 1.0);
//             let font_size = f32::from_bits(physical.cache_key.font_size_bits);

//             scene
//                 .draw_glyphs(&font_ref)
//                 .font_size(font_size)
//                 .brush(&brush)
//                 .fill(Fill::NonZero)
//                 .draw(glyphs);
//         }
//     }
// }
