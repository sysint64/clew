use std::{num::NonZeroU32, slice};

use cosmic_text::SwashCache;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tech_paws_ui::{
    Border, BorderRadius, BorderSide, ColorRgba, Gradient, Rect, TileMode, View,
    render::{Fill, RenderCommand, RenderState, Renderer},
    text::{FontResources, TextsResources},
};
use tiny_skia::{Paint, PixmapMut};

pub struct TinySkiaRenderer<D, W> {
    context: softbuffer::Context<D>,
    surface: softbuffer::Surface<D, W>,
    current_width: u32,
    current_height: u32,
    swash_cache: SwashCache,
}

impl<D: HasDisplayHandle, W: HasWindowHandle> TinySkiaRenderer<D, W> {
    pub fn new(display: D, window: W) -> Self {
        let context = softbuffer::Context::new(display).unwrap();
        let mut surface = softbuffer::Surface::new(&context, window).unwrap();

        Self {
            context,
            surface,
            current_width: 0,
            current_height: 0,
            swash_cache: SwashCache::new(),
        }
    }
}

impl<D: HasDisplayHandle, W: HasWindowHandle> Renderer for TinySkiaRenderer<D, W> {
    fn process_commands(
        &mut self,
        view: &View,
        state: &RenderState,
        fonts: &mut FontResources,
        text: &mut TextsResources,
    ) {
        let width = view.size.width as u32;
        let height = view.size.height as u32;

        if self.current_width != width || self.current_height != height {
            self.surface
                .resize(
                    NonZeroU32::new(width).unwrap(),
                    NonZeroU32::new(height).unwrap(),
                )
                .unwrap();

            self.current_width = width;
            self.current_height = height;
        }

        let mut surface_buffer = self.surface.buffer_mut().unwrap();
        let surface_buffer_u8 = unsafe {
            slice::from_raw_parts_mut(
                surface_buffer.as_mut_ptr() as *mut u8,
                surface_buffer.len() * 4,
            )
        };
        let mut pixmap = PixmapMut::from_bytes(surface_buffer_u8, width, height).unwrap();
        pixmap.fill(tiny_skia::Color::from_rgba8(0, 0, 0, 0xFF));

        let mut clip_stack: Vec<tiny_skia::Mask> = Vec::new();

        for command in state.commands() {
            let current_clip = clip_stack.last();

            match command {
                RenderCommand::Rect {
                    boundary,
                    fill,
                    border_radius,
                    border,
                } => {
                    render_rect(
                        &mut pixmap,
                        *boundary,
                        fill,
                        border_radius,
                        border,
                        current_clip,
                    );
                }
                RenderCommand::Oval {
                    boundary,
                    fill,
                    border,
                } => {
                    render_oval(&mut pixmap, *boundary, fill, border.as_ref(), current_clip);
                }
                RenderCommand::Text {
                    x: text_position_x,
                    y: text_position_y,
                    text_id,
                    tint_color,
                } => {
                    let mut paint = Paint {
                        anti_alias: false,
                        ..Default::default()
                    };

                    text.get_mut(*text_id).with_buffer_mut(|buffer| {
                        buffer.draw(
                            &mut fonts.font_system,
                            &mut self.swash_cache,
                            tint_color.unwrap_or(ColorRgba::from_hex(0xFF000000)).into(),
                            |x, y, w, h, color| {
                                let opacity = color.a() as f32 / 255.;
                                let color = tint_color
                                    .map(|c| c.with_opacity(opacity * c.a).into())
                                    .unwrap_or(color);

                                // Note: due to softbuffer and tiny_skia having incompatible internal color representations we swap
                                // the red and blue channels here
                                paint.set_color_rgba8(color.b(), color.g(), color.r(), color.a());
                                pixmap.fill_rect(
                                    tiny_skia::Rect::from_xywh(
                                        text_position_x + x as f32,
                                        text_position_y + y as f32,
                                        w as f32,
                                        h as f32,
                                    )
                                    .unwrap(),
                                    &paint,
                                    tiny_skia::Transform::identity(),
                                    None,
                                );
                            },
                        );
                    });
                }
                RenderCommand::PushClipRect(_rect) => {
                    // let clip_path = {
                    //     let mut pb = tiny_skia::PathBuilder::new();
                    //     if let Some(rect) = tiny_skia::Rect::from_xywh(*x, *y, *width, *height) {
                    //         pb.push_rect(rect);
                    //     }
                    //     pb.finish().unwrap()
                    // };

                    // let mut new_mask = tiny_skia::Mask::new(*width as u32, *height as u32).unwrap();
                    // new_mask.fill_path(
                    //     &clip_path,
                    //     tiny_skia::FillRule::Winding,
                    //     true,
                    //     tiny_skia::Transform::identity(),
                    // );

                    // // Intersect with parent clip if exists
                    // // if let Some(parent_clip) = current_clip {
                    // // new_mask.intersect_mask(parent_clip);
                    // // }

                    // clip_stack.push(new_mask);
                }
                RenderCommand::PopClip => {
                    // clip_stack.pop();
                }
            }
        }

        surface_buffer.present().unwrap();
    }
}

fn render_rect(
    pixmap: &mut PixmapMut,
    boundary: Rect,
    fill: &Fill,
    border_radius: &BorderRadius,
    border: &Border,
    clip_mask: Option<&tiny_skia::Mask>,
) {
    let has_radius = border_radius.top_left > 0.0
        || border_radius.top_right > 0.0
        || border_radius.bottom_left > 0.0
        || border_radius.bottom_right > 0.0;

    let path = if has_radius {
        create_rounded_rect_path(boundary, border_radius)
    } else {
        let mut pb = tiny_skia::PathBuilder::new();
        if let Some(rect) =
            tiny_skia::Rect::from_xywh(boundary.x, boundary.y, boundary.width, boundary.height)
        {
            pb.push_rect(rect);
        }
        pb.finish().unwrap()
    };

    // Render fill
    if let Some(paint) = create_paint_from_fill(fill, boundary) {
        pixmap.fill_path(
            &path,
            &paint,
            tiny_skia::FillRule::Winding,
            tiny_skia::Transform::identity(),
            clip_mask,
        );
    }

    // Render border
    render_border(pixmap, &path, border, clip_mask);
}

fn render_oval(
    pixmap: &mut PixmapMut,
    boundary: Rect,
    fill: &Fill,
    border: Option<&BorderSide>,
    clip_mask: Option<&tiny_skia::Mask>,
) {
    let cx = boundary.x + boundary.width / 2.0;
    let cy = boundary.y + boundary.height / 2.0;
    let rx = boundary.width / 2.0;
    let ry = boundary.height / 2.0;

    let path = {
        let mut pb = tiny_skia::PathBuilder::new();
        // Create ellipse using cubic bezier curves
        // Magic constant for circle/ellipse approximation with bezier curves
        const KAPPA: f32 = 0.5522847498;

        let ox = rx * KAPPA; // control point offset x
        let oy = ry * KAPPA; // control point offset y

        pb.move_to(cx - rx, cy);
        pb.cubic_to(cx - rx, cy - oy, cx - ox, cy - ry, cx, cy - ry);
        pb.cubic_to(cx + ox, cy - ry, cx + rx, cy - oy, cx + rx, cy);
        pb.cubic_to(cx + rx, cy + oy, cx + ox, cy + ry, cx, cy + ry);
        pb.cubic_to(cx - ox, cy + ry, cx - rx, cy + oy, cx - rx, cy);
        pb.close();

        pb.finish().unwrap()
    };

    // Render fill
    if let Some(paint) = create_paint_from_fill(fill, boundary) {
        pixmap.fill_path(
            &path,
            &paint,
            tiny_skia::FillRule::Winding,
            tiny_skia::Transform::identity(),
            clip_mask,
        );
    }

    // Render border
    if let Some(border_side) = border {
        let mut stroke = tiny_skia::Stroke::default();
        stroke.width = border_side.width;

        let mut paint = tiny_skia::Paint::default();
        paint.set_color(convert_color(&border_side.color));
        paint.anti_alias = true;

        pixmap.stroke_path(
            &path,
            &paint,
            &stroke,
            tiny_skia::Transform::identity(),
            clip_mask,
        );
    }
}

fn create_rounded_rect_path(rect: Rect, border_radius: &BorderRadius) -> tiny_skia::Path {
    let mut pb = tiny_skia::PathBuilder::new();

    let right = rect.x + rect.width;
    let bottom = rect.y + rect.height;

    // Clamp radii to not exceed half the width/height
    let max_radius_x = rect.width / 2.0;
    let max_radius_y = rect.height / 2.0;

    let tl = border_radius.top_left.min(max_radius_x).min(max_radius_y);
    let tr = border_radius.top_right.min(max_radius_x).min(max_radius_y);
    let br = border_radius
        .bottom_right
        .min(max_radius_x)
        .min(max_radius_y);
    let bl = border_radius
        .bottom_left
        .min(max_radius_x)
        .min(max_radius_y);

    // Start from top-left, after the corner radius
    pb.move_to(rect.x + tl, rect.y);

    // Top edge
    pb.line_to(right - tr, rect.y);

    // Top-right corner
    if tr > 0.0 {
        pb.quad_to(right, rect.y, right, rect.y + tr);
    }

    // Right edge
    pb.line_to(right, bottom - br);

    // Bottom-right corner
    if br > 0.0 {
        pb.quad_to(right, bottom, right - br, bottom);
    }

    // Bottom edge
    pb.line_to(rect.x + bl, bottom);

    // Bottom-left corner
    if bl > 0.0 {
        pb.quad_to(rect.x, bottom, rect.x, bottom - bl);
    }

    // Left edge
    pb.line_to(rect.x, rect.y + tl);

    // Top-left corner
    if tl > 0.0 {
        pb.quad_to(rect.x, rect.y, rect.x + tl, rect.y);
    }

    pb.close();
    pb.finish().unwrap()
}

fn create_paint_from_fill(fill: &Fill, rect: Rect) -> Option<tiny_skia::Paint<'static>> {
    match fill {
        Fill::None => None,
        Fill::Color(color) => {
            let mut paint = tiny_skia::Paint::default();
            paint.set_color(convert_color(color));
            paint.anti_alias = true;
            Some(paint)
        }
        Fill::Gradient(gradient) => {
            let shader = create_gradient_shader(gradient, rect)?;
            let mut paint = tiny_skia::Paint::default();
            paint.shader = shader;
            paint.anti_alias = true;
            Some(paint)
        }
    }
}

fn create_gradient_shader(gradient: &Gradient, rect: Rect) -> Option<tiny_skia::Shader<'static>> {
    match gradient {
        Gradient::Linear(linear) => {
            let stops: Vec<tiny_skia::GradientStop> = linear
                .stops
                .iter()
                .map(|stop| tiny_skia::GradientStop::new(stop.offset, convert_color(&stop.color)))
                .collect();

            // Convert normalized coordinates to absolute coordinates
            let start_x = rect.x + linear.start.0 * rect.width;
            let start_y = rect.y + linear.start.1 * rect.height;
            let end_x = rect.x + linear.end.0 * rect.width;
            let end_y = rect.y + linear.end.1 * rect.height;

            tiny_skia::LinearGradient::new(
                tiny_skia::Point::from_xy(start_x, start_y),
                tiny_skia::Point::from_xy(end_x, end_y),
                stops,
                convert_tile_mode(&linear.tile_mode),
                tiny_skia::Transform::identity(),
            )
        }
        Gradient::Radial(radial) => {
            let stops: Vec<tiny_skia::GradientStop> = radial
                .stops
                .iter()
                .map(|stop| tiny_skia::GradientStop::new(stop.offset, convert_color(&stop.color)))
                .collect();

            // Convert normalized coordinates to absolute coordinates
            let center_x = rect.x + radial.center.0 * rect.width;
            let center_y = rect.y + radial.center.1 * rect.height;
            let radius = radial.radius * rect.width.max(rect.height);

            // Use focal point if provided, otherwise use center
            let (focal_x, focal_y) = if let Some(focal) = radial.focal {
                (
                    rect.x + focal.0 * rect.width,
                    rect.y + focal.1 * rect.height,
                )
            } else {
                (center_x, center_y)
            };

            tiny_skia::RadialGradient::new(
                tiny_skia::Point::from_xy(center_x, center_y),
                tiny_skia::Point::from_xy(focal_x, focal_y),
                radius,
                stops,
                convert_tile_mode(&radial.tile_mode),
                tiny_skia::Transform::identity(),
            )
        }
        Gradient::Sweep(_sweep) => {
            // tiny-skia doesn't have native sweep gradient support
            // You could implement it using a custom shader or fall back to radial
            None
        }
    }
}

fn convert_tile_mode(tile_mode: &TileMode) -> tiny_skia::SpreadMode {
    match tile_mode {
        TileMode::Clamp => tiny_skia::SpreadMode::Pad,
        TileMode::Repeat => tiny_skia::SpreadMode::Repeat,
        TileMode::Mirror => tiny_skia::SpreadMode::Reflect,
        TileMode::Decal => tiny_skia::SpreadMode::Pad, // tiny-skia doesn't have decal
    }
}

fn render_border(
    pixmap: &mut PixmapMut,
    path: &tiny_skia::Path,
    border: &Border,
    clip_mask: Option<&tiny_skia::Mask>,
) {
    // For uniform borders, we can stroke once
    // For non-uniform borders, we'd need to stroke each side separately
    // This is a simplified implementation that strokes all sides with the same width

    // Find the maximum border width to use
    let max_width = [
        border.top.as_ref().map(|s| s.width).unwrap_or(0.0),
        border.right.as_ref().map(|s| s.width).unwrap_or(0.0),
        border.bottom.as_ref().map(|s| s.width).unwrap_or(0.0),
        border.left.as_ref().map(|s| s.width).unwrap_or(0.0),
    ]
    .into_iter()
    .fold(0.0f32, f32::max);

    if max_width > 0.0 {
        // Use the first available border side's color
        let color = border
            .top
            .as_ref()
            .or(border.right.as_ref())
            .or(border.bottom.as_ref())
            .or(border.left.as_ref())
            .map(|s| s.color)
            .unwrap_or(ColorRgba::TRANSPARENT);

        let mut stroke = tiny_skia::Stroke::default();
        stroke.width = max_width;

        let mut paint = tiny_skia::Paint::default();
        paint.set_color(convert_color(&color));
        paint.anti_alias = true;

        pixmap.stroke_path(
            path,
            &paint,
            &stroke,
            tiny_skia::Transform::identity(),
            clip_mask,
        );
    }
}

fn convert_color(color: &ColorRgba) -> tiny_skia::Color {
    // Note: due to softbuffer and tiny_skia having incompatible internal color representations we swap
    // the red and blue channels here
    tiny_skia::Color::from_rgba(color.b, color.g, color.r, color.a).unwrap()
}
