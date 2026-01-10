use std::collections::HashMap;

use crate::{
    Border, BorderRadius, BorderSide, ClipShape, ColorRgb, ColorRgba, DebugBoundary, Gradient,
    LayoutDirection, Rect, Vec2, View, WidgetType,
    assets::Assets,
    interaction::{InteractionState, handle_interaction},
    io::UserInput,
    layout::{LayoutItem, WidgetPlacement, layout},
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
    fn upload_svg(&mut self, _name: &'static str, _tree: &usvg::Tree) {}

    fn on_scale_factor_update(&mut self, _scale_factor: f32) {}

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
    unsorted_commands: &'a mut Vec<RenderCommandUnsorted>,
    commands: &'a mut Vec<RenderCommand>,
}

impl RenderContext<'_, '_> {
    pub fn push_command(&mut self, zindex: i32, command: RenderCommand) {
        self.commands.push(command);
    }
}

#[derive(Debug, Clone)]
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
    PushClip {
        zindex: i32,
        rect: Rect,
        shape: ClipShape,
    },
    PopClip,
    BeginGroup {
        zindex: i32,
    },
    EndGroup,
}

#[derive(Debug, Clone)]
pub enum RenderCommandUnsorted {
    RenderCommand { zindex: i32, command: RenderCommand },
    BeginGroup { zindex: i32 },
    EndGroup,
}

impl RenderCommand {
    pub fn zindex(&self) -> i32 {
        match self {
            RenderCommand::Rect { zindex, .. } => *zindex,
            RenderCommand::Oval { zindex, .. } => *zindex,
            RenderCommand::Text { zindex, .. } => *zindex,
            RenderCommand::Svg { zindex, .. } => *zindex,
            RenderCommand::PushClip { zindex, .. } => *zindex,
            RenderCommand::BeginGroup { zindex } => *zindex,
            RenderCommand::PopClip | RenderCommand::EndGroup => {
                unreachable!("End markers should not be sorted independently")
            }
        }
    }
}

#[derive(Debug, Clone)]
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

impl PixelExtension<ClipShape> for ClipShape {
    fn px(self, ctx: &RenderContext) -> ClipShape {
        match self {
            ClipShape::Rect => self,
            ClipShape::RoundedRect { border_radius } => ClipShape::RoundedRect {
                border_radius: border_radius.px(ctx),
            },
            ClipShape::Oval => self,
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

#[derive(Clone, Copy)]
enum GroupKind {
    Clip,
    Group,
}

impl GroupKind {
    fn matches_end(&self, cmd: &RenderCommand) -> bool {
        matches!(
            (self, cmd),
            (GroupKind::Clip, RenderCommand::PopClip) | (GroupKind::Group, RenderCommand::EndGroup)
        )
    }
}

fn group_start(cmd: &RenderCommand) -> Option<GroupKind> {
    match cmd {
        RenderCommand::PushClip { .. } => Some(GroupKind::Clip),
        RenderCommand::BeginGroup { .. } => Some(GroupKind::Group),
        _ => None,
    }
}

fn group_end(cmd: &RenderCommand) -> Option<GroupKind> {
    match cmd {
        RenderCommand::PopClip => Some(GroupKind::Clip),
        RenderCommand::EndGroup => Some(GroupKind::Group),
        _ => None,
    }
}

pub fn sort_render_commands(commands: &mut Vec<RenderCommand>) {
    let len = commands.len();
    sort_segment(commands, 0, len);
}

fn sort_segment(commands: &mut [RenderCommand], start: usize, end: usize) {
    let mut items: Vec<(usize, usize, i32)> = Vec::new();
    let mut i = start;

    while i < end {
        if let Some(kind) = group_start(&commands[i]) {
            let group_start_idx = i;
            let group_zindex = commands[i].zindex();
            let mut depth = 1;
            i += 1;

            while i < end && depth > 0 {
                if group_start(&commands[i]).is_some() {
                    depth += 1;
                } else if kind.matches_end(&commands[i]) {
                    depth -= 1;
                }
                i += 1;
            }

            items.push((group_start_idx, i, group_zindex));
        } else if group_end(&commands[i]).is_some() {
            break;
        } else {
            items.push((i, i + 1, commands[i].zindex()));
            i += 1;
        }
    }

    items.sort_by_key(|&(start, _, z)| (z, start));

    let original: Vec<RenderCommand> = commands[start..end].to_vec();
    let base = start;

    let mut write_pos = start;
    for (item_start, item_end, _) in &items {
        let src_start = item_start - base;
        let src_end = item_end - base;
        let len = src_end - src_start;

        commands[write_pos..write_pos + len].clone_from_slice(&original[src_start..src_end]);
        write_pos += len;
    }

    // Recursively sort inside each group
    let mut i = start;
    while i < end {
        if let Some(kind) = group_start(&commands[i]) {
            let content_start = i + 1;
            let mut depth = 1;
            i += 1;

            while i < end && depth > 0 {
                if group_start(&commands[i]).is_some() {
                    depth += 1;
                } else if kind.matches_end(&commands[i]) {
                    depth -= 1;
                }
                i += 1;
            }

            sort_segment(commands, content_start, i - 1);
        } else {
            i += 1;
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
        profiling::scope!("clew :: Layout");

        layout(
            &mut state.layout_state,
            &state.view,
            &state.layout_commands,
            &mut state.layout_items,
            &mut state.widgets_states.layout_measures,
            text,
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
            &mut state.layout_items,
            &mut state.widgets_states.layout_measures,
            text,
            assets,
        );
    }

    tracy_client::plot!(
        "clew :: Layout commands",
        state.layout_commands.len() as f64
    );

    {
        profiling::scope!("clew :: Interaction");

        need_to_redraw = need_to_redraw
            || handle_interaction(
                &mut state.user_input,
                &mut state.interaction_state,
                &state.non_interactable,
                // &mut state.widgets_states,
                &state.view,
                text,
                fonts,
                &state.layout_items,
            );

        need_to_redraw = need_to_redraw || state.interaction_state != state.last_interaction_state;
        state.last_interaction_state = state.interaction_state.clone();
    }

    if force_redraw || need_to_redraw {
        profiling::scope!("clew :: Collect Render Commands");

        for layout_item in &state.layout_items {
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

            match layout_item {
                LayoutItem::Placement(placement) => {
                    if placement.widget_ref.widget_type
                        == WidgetType::of::<widgets::text::TextWidget>()
                    {
                        widgets::text::render(
                            &mut render_context,
                            placement,
                            state
                                .widgets_states
                                .text
                                .get(placement.widget_ref.id)
                                .unwrap(),
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

                    if placement.widget_ref.widget_type
                        == WidgetType::of::<widgets::svg::SvgWidget>()
                    {
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
                LayoutItem::PushClip { rect, clip, zindex } => {
                    let shape = clip
                        .to_shape()
                        .expect("Cannot push clip without a shape")
                        .px(&render_context);

                    let rect = rect.px(&render_context);

                    state.render_state.commands.push(RenderCommand::PushClip {
                        rect,
                        shape,
                        zindex: *zindex,
                    })
                }
                LayoutItem::PopClip => {
                    state.render_state.commands.push(RenderCommand::PopClip);
                }
                LayoutItem::BeginGroup { zindex } => {
                    state
                        .render_state
                        .commands
                        .push(RenderCommand::BeginGroup { zindex: *zindex });
                }
                LayoutItem::EndGroup => {
                    state.render_state.commands.push(RenderCommand::EndGroup);
                }
            }
        }

        tracy_client::plot!("clew :: Layout Items", state.layout_items.len() as f64);

        tracy_client::plot!(
            "clew :: Render Commands",
            state.render_state.commands.len() as f64
        );
    }

    state.widgets_states.sweep();
    state.user_input.clear_frame_events();

    {
        profiling::scope!("clew :: Sort commands by zindex");

        // println!("Before sort:");
        // for (i, cmd) in state.render_state.commands.iter().enumerate() {
        //     println!("  {}: {:?}", i, cmd);
        // }

        sort_render_commands(&mut state.render_state.commands);

        // println!("After sort:");
        // for (i, cmd) in state.render_state.commands.iter().enumerate() {
        //     println!("  {}: {:?}", i, cmd);
        // }

        // sort_render_commands(&mut state.render_state.commands);
        // state
        //     .render_state
        //     .commands
        //     .sort_by_key(|cmd| cmd.zindex().unwrap_or(i32::MAX));
    }

    {
        profiling::scope!("clew :: Reset phase allocator");
        state.phase_allocator.reset();
    }

    force_redraw || need_to_redraw
}

fn render_debug_boundary(ctx: &mut RenderContext, placement: &WidgetPlacement) {
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
