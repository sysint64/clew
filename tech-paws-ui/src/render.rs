use std::collections::HashMap;

use glam::Vec2;

use crate::{
    Border, BorderRadius, BorderSide, ColorRgb, ColorRgba, DebugBoundary, Gradient,
    LayoutDirection, Rect, View, WidgetType,
    assets::Assets,
    interaction::{InteractionState, handle_interaction},
    io::UserInput,
    layout::{WidgetPlacement, layout},
    state::UiState,
    text::{FontResources, StringId, StringInterner, TextId, TextsResources},
    widgets::{self, builder::BuildContext},
};

#[derive(Debug, Default)]
pub struct RenderState {
    pub(crate) commands: Vec<RenderCommand>,
}

impl RenderState {
    pub fn commands(&self) -> &[RenderCommand] {
        &self.commands
    }
}

pub trait Renderer {
    fn upload_svg(&mut self, name: &'static str, tree: &usvg::Tree) {}

    fn on_scale_factor_update(&mut self, scale_factor: f32) {}

    fn process_commands(
        &mut self,
        view: &View,
        state: &RenderState,
        fill_color: ColorRgb,
        fonts: &mut FontResources,
        text: &mut TextsResources,
        assets: &Assets,
    );
}

pub struct RenderContext<'a, 'b> {
    pub interaction: &'a InteractionState,
    pub input: &'a UserInput,
    pub view: &'a View,
    pub text: &'a mut TextsResources<'b>,
    pub fonts: &'a mut FontResources,
    pub string_interner: &'a mut StringInterner,
    pub strings: &'a mut HashMap<StringId, TextId>,
    pub layout_direction: LayoutDirection,
    commands: &'a mut Vec<RenderCommand>,
}

impl RenderContext<'_, '_> {
    pub fn push_command(&mut self, command: RenderCommand) {
        self.commands.push(command);
    }
}

#[derive(Debug)]
pub enum RenderCommand {
    Rect {
        zindex: i32,
        boundary: Rect,
        fill: Option<Fill>,
        border_radius: Option<BorderRadius>,
        border: Option<Border>,
    },
    Oval {
        zindex: i32,
        boundary: Rect,
        fill: Option<Fill>,
        border: Option<BorderSide>,
    },
    Text {
        zindex: i32,
        x: f32,
        y: f32,
        text_id: TextId,
        tint_color: Option<ColorRgba>,
    },
    Svg {
        zindex: i32,
        boundary: Rect,
        asset_id: &'static str,
        tint_color: Option<ColorRgba>,
    },
    PushClipRect(Rect),
    PopClip,
}

impl RenderCommand {
    fn zindex(&self) -> Option<i32> {
        match self {
            RenderCommand::Rect { zindex, .. } => Some(*zindex),
            RenderCommand::Oval { zindex, .. } => Some(*zindex),
            RenderCommand::Text { zindex, .. } => Some(*zindex),
            RenderCommand::Svg { zindex, .. } => Some(*zindex),
            RenderCommand::PushClipRect(rect) => None,
            RenderCommand::PopClip => None,
        }
    }
}

#[derive(Debug)]
pub enum Fill {
    None,
    Color(ColorRgba),
    Gradient(Gradient),
}

pub trait PixelExtension<T> {
    fn px(self, ctx: &RenderContext) -> T;
}

impl PixelExtension<f32> for f32 {
    fn px(self, ctx: &RenderContext) -> f32 {
        self * ctx.view.scale_factor.ceil()
    }
}

impl PixelExtension<Vec2> for Vec2 {
    fn px(self, ctx: &RenderContext) -> Vec2 {
        Vec2::new(self.x.px(ctx), self.y.px(ctx))
    }
}

impl PixelExtension<Rect> for Rect {
    fn px(self, ctx: &RenderContext) -> Rect {
        self * ctx.view.scale_factor.ceil()
    }
}

impl PixelExtension<BorderRadius> for BorderRadius {
    fn px(self, ctx: &RenderContext) -> BorderRadius {
        BorderRadius {
            top_left: self.top_left * ctx.view.scale_factor,
            top_right: self.top_right * ctx.view.scale_factor,
            bottom_left: self.bottom_left * ctx.view.scale_factor,
            bottom_right: self.bottom_right * ctx.view.scale_factor,
        }
    }
}

impl PixelExtension<BorderSide> for BorderSide {
    fn px(self, ctx: &RenderContext) -> BorderSide {
        BorderSide {
            width: self.width * ctx.view.scale_factor,
            color: self.color,
        }
    }
}

impl PixelExtension<Border> for Border {
    fn px(self, ctx: &RenderContext) -> Border {
        Border {
            top: self.top.map(|border_side| border_side.px(ctx)),
            right: self.right.map(|border_side| border_side.px(ctx)),
            bottom: self.bottom.map(|border_side| border_side.px(ctx)),
            left: self.left.map(|border_side| border_side.px(ctx)),
        }
    }
}

#[profiling::function]
pub fn cache_string<F>(ctx: &mut BuildContext, symbol: StringId, create_text_id: F) -> TextId
where
    F: FnOnce(&mut BuildContext) -> TextId,
{
    match ctx.strings.get(&symbol) {
        Some(text_id) => *text_id,
        None => {
            let text_id = create_text_id(ctx);
            ctx.strings.insert(symbol, text_id);

            text_id
        }
    }
}

pub fn render(
    state: &mut UiState,
    text: &mut TextsResources,
    fonts: &mut FontResources,
    assets: &Assets,
    string_interner: &mut StringInterner,
    strings: &mut HashMap<StringId, TextId>,
    force_redraw: bool,
) -> bool {
    let mut need_to_redraw = false;

    // let layout_time = std::time::Instant::now();
    {
        profiling::scope!("Tech Paws UI - Layout");

        layout(
            &mut state.layout_state,
            &state.view,
            &state.layout_commands,
            &mut state.widget_placements,
            text,
            fonts,
            assets,
        );

        for layout_text in &state.layout_state.texts {
            let text = text.get_mut(layout_text.text_id);

            text.with_buffer_mut(|buffer| {
                buffer.set_size(&mut fonts.font_system, Some(layout_text.width), None);
            });
        }

        layout(
            &mut state.layout_state,
            &state.view,
            &state.layout_commands,
            &mut state.widget_placements,
            text,
            fonts,
            assets,
        );
    }
    // println!(
    //     "LAYOUT TIME FOR {} COMMANDS: {:?}",
    //     state.layout_commands.len(),
    //     layout_time.elapsed()
    // );

    tracy_client::plot!(
        "Tech Paws UI - Layout commands",
        state.layout_commands.len() as f64
    );

    // let interaction_time = std::time::Instant::now();

    {
        profiling::scope!("Tech Paws UI - Interaction");

        need_to_redraw = need_to_redraw
            || handle_interaction(
                &mut state.user_input,
                &mut state.interaction_state,
                &mut state.widgets_states,
                &state.view,
                text,
                fonts,
                &state.widget_placements,
            );

        need_to_redraw = need_to_redraw || state.interaction_state != state.last_interaction_state;
        state.last_interaction_state = state.interaction_state.clone();
    }

    // println!("INTERACTION TIME: {:?}", interaction_time.elapsed());

    if force_redraw || need_to_redraw {
        profiling::scope!("Tech Paws UI - Collect Render Commands");
        // println!("REDRAW");

        // let render_time = std::time::Instant::now();

        for placement in &state.widget_placements {
            let mut render_context = RenderContext {
                interaction: &state.interaction_state,
                input: &state.user_input,
                view: &state.view,
                text,
                fonts,
                string_interner,
                strings,
                layout_direction: state.layout_direction,
                commands: &mut state.render_state.commands,
            };

            // if placement.widget_ref.widget_type == WidgetType::of::<widgets::button::ButtonWidget>()
            // {
            //     widgets::button::render(
            //         &mut render_context,
            //         placement,
            //         state
            //             .widgets_states
            //             .get_mut::<widgets::button::State>(placement.widget_ref.id)
            //             .unwrap(),
            //     );
            // }

            if placement.widget_ref.widget_type == WidgetType::of::<widgets::text::TextWidget>() {
                widgets::text::render(
                    &mut render_context,
                    placement,
                    // state
                    //     .widgets_states
                    //     .get_mut::<widgets::text::State>(placement.widget_ref.id)
                    //     .unwrap(),
                    state
                        .widgets_states
                        .text
                        .get(placement.widget_ref.id)
                        .unwrap(),
                );
            }

            if placement.widget_ref.widget_type
                == WidgetType::of::<widgets::colored_box::ColoredBox>()
            {
                widgets::colored_box::render(
                    &mut render_context,
                    placement,
                    state
                        .widgets_states
                        .colored_box
                        .get(placement.widget_ref.id)
                        .unwrap(),
                    // state
                    //     .widgets_states
                    //     .get_mut::<widgets::colored_box::State>(placement.widget_ref.id)
                    //     .unwrap(),
                );
            }

            if placement.widget_ref.widget_type
                == WidgetType::of::<widgets::decorated_box::DecoratedBox>()
            {
                widgets::decorated_box::render(
                    &mut render_context,
                    placement,
                    state
                        .widgets_states
                        .decorated_box
                        .get(placement.widget_ref.id)
                        .unwrap(),
                    // state
                    //     .widgets_states
                    //     .get_mut::<widgets::decorated_box::State>(placement.widget_ref.id)
                    //     .unwrap(),
                );
            }

            if placement.widget_ref.widget_type == WidgetType::of::<widgets::svg::SvgWidget>() {
                widgets::svg::render(
                    &mut render_context,
                    placement,
                    state
                        .widgets_states
                        .svg
                        .get(placement.widget_ref.id)
                        .unwrap(),
                    // state
                    //     .widgets_states
                    //     .get_mut::<widgets::svg::State>(placement.widget_ref.id)
                    //     .unwrap(),
                );
            }

            if placement.widget_ref.widget_type == WidgetType::of::<DebugBoundary>() {
                render_debug_boundary(&mut render_context, placement);
            }
        }

        tracy_client::plot!(
            "Tech Paws UI - Render Commands",
            state.widget_placements.len() as f64
        );
        // println!(
        //     "RENDER COMMAND CREATED FOR {} PLACEMENTS: {:?}",
        //     state.widget_placements.len(),
        //     render_time.elapsed()
        // );
    }

    // let clean_up_time = std::time::Instant::now();

    state.widgets_states.sweep(&mut state.interaction_state);
    state.user_input.clear_frame_events();

    // println!("CLEAN UP TIME: {:?}", clean_up_time.elapsed());

    // let commands_sort_time = std::time::Instant::now();
    // state
    // .render_state
    // .commands
    // .sort_by_key(|cmd| cmd.zindex().unwrap_or(i32::MAX));
    // println!("COMMANDS SORT TIME: {:?}", commands_sort_time.elapsed());

    {
        profiling::scope!("Tech Paws UI - Reset phase allocator");
        state.phase_allocator.reset();
    }

    force_redraw || need_to_redraw
}

fn render_debug_boundary(ctx: &mut RenderContext, placement: &WidgetPlacement) {
    let size = placement.rect.size().px(ctx);
    let position = placement.rect.position().px(ctx);

    ctx.push_command(RenderCommand::Rect {
        zindex: placement.zindex,
        boundary: placement.rect.shrink(2.).px(ctx),
        fill: None,
        border_radius: None,
        border: Some(Border::all(BorderSide::new(
            2.,
            ColorRgba::from_hex(0xFFFF0000),
        ))),
    });
}

fn render_performance() {}
