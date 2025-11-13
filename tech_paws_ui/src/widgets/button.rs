use glam::Vec2;

use super::builder::BuildContext;
use crate::{
    AlignX, AlignY, Border, BorderRadius, BorderSide, ColorRgba, Constraints, EdgeInsets, Gradient,
    LinearGradient, Size, SizeConstraint, WidgetId, WidgetRef, WidgetType, impl_id,
    impl_position_methods, impl_width_methods,
    interaction::InteractionState,
    io::UserInput,
    layout::{ContainerKind, LayoutCommand, WidgetPlacement},
    render::{Fill, PixelExtension, RenderCommand, RenderContext, cache_string},
    state::WidgetState,
    text::StringId,
};
use std::{any::Any, hash::Hash};

pub struct ButtonBuilder<'a> {
    id: WidgetId,
    text: &'a str,
    width: SizeConstraint,
    constraints: Constraints,
    align_x: Option<AlignX>,
    align_y: Option<AlignY>,
    zindex: Option<i32>,
    padding: Option<EdgeInsets>,
}

pub struct ButtonResponse {
    clicked: bool,
}

impl ButtonResponse {
    pub fn clicked(&self) -> bool {
        self.clicked
    }
}

#[derive(Clone, PartialEq)]
pub struct State {
    pub(crate) text: StringId,
    pub(crate) clicked: bool,
}

pub struct ButtonWidget;

impl WidgetState for State {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<'a> ButtonBuilder<'a> {
    impl_id!();
    impl_width_methods!();
    impl_position_methods!();

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = Some(padding);

        self
    }

    pub fn build(&self, context: &mut BuildContext) -> ButtonResponse {
        let id = self.id.with_seed(context.id_seed);

        let text = context.string_interner.get_or_intern(self.text);
        let size = Size::new(self.width, SizeConstraint::Fixed(20.0));
        let widget_ref = WidgetRef::new(WidgetType::of::<ButtonWidget>(), id);

        if let Some(padding) = self.padding {
            let mut padding_containts = self.constraints;
            padding_containts.expand(padding);

            context.push_layout_command(LayoutCommand::BeginContainer {
                kind: ContainerKind::Padding { padding },
                size,
                constraints: self.constraints,
            });

            context.with_align(self.align_x, self.align_y, |context| {
                context.push_layout_command(LayoutCommand::Fixed {
                    widget_ref,
                    constraints: self.constraints,
                    size,
                    zindex: self.zindex.unwrap_or(context.current_zindex),
                });
            });

            context.push_layout_command(LayoutCommand::EndContainer);
        } else {
            context.with_align(self.align_x, self.align_y, |context| {
                context.push_layout_command(LayoutCommand::Fixed {
                    widget_ref,
                    constraints: self.constraints,
                    size,
                    zindex: self.zindex.unwrap_or(context.current_zindex),
                });
            });
        }

        context.widgets_states.accessed_this_frame.insert(id);

        let state = context
            .widgets_states
            .get_or_insert::<State, _>(id, || State {
                clicked: false,
                text,
            });

        state.text = text;

        ButtonResponse {
            clicked: state.clicked,
        }
    }
}

#[track_caller]
pub fn button(text: &str) -> ButtonBuilder<'_> {
    ButtonBuilder {
        id: WidgetId::auto_with_seed(text),
        text,
        width: SizeConstraint::Wrap,
        align_x: None,
        align_y: None,
        padding: None,
        zindex: None,
        constraints: Constraints {
            min_width: Some(100.),
            min_height: Some(20.),
            max_width: None,
            max_height: Some(20.),
        },
    }
}

#[track_caller]
pub fn button_id(id: impl Hash, text: &str) -> ButtonBuilder {
    ButtonBuilder {
        id: WidgetId::auto_with_seed(id),
        text,
        width: SizeConstraint::Wrap,
        align_x: None,
        align_y: None,
        padding: None,
        zindex: None,
        constraints: Constraints {
            min_width: Some(100.),
            min_height: Some(20.),
            max_width: None,
            max_height: Some(20.),
        },
    }
}

pub fn handle_interaction(
    id: WidgetId,
    input: &UserInput,
    interaction: &mut InteractionState,
    widget_state: &mut State,
) {
    widget_state.clicked = false;

    if interaction.is_active(&id) {
        if input.mouse_released {
            if interaction.is_hot(&id) {
                interaction.set_inactive(&id);
                interaction.focused = Some(id);
                widget_state.clicked = true;
            } else {
                interaction.set_inactive(&id);
            }
        }
    } else if input.mouse_left_pressed && interaction.is_hot(&id) {
        interaction.focused = Some(id);
        interaction.set_active(&id);
    }
}

pub fn render(ctx: &mut RenderContext, placement: &WidgetPlacement, state: &State) {
    let id = placement.widget_ref.id;
    let size = placement.rect.size().px(ctx);
    let position = placement.rect.position().px(ctx);

    let border_color = if ctx.interaction.is_focused(&id) {
        ColorRgba::from_hex(0xFF357CCE)
    } else if ctx.interaction.is_active(&id) && ctx.interaction.is_hot(&id) {
        ColorRgba::from_hex(0xFF414141)
    } else if ctx.interaction.is_hot(&id) {
        ColorRgba::from_hex(0xFF616161)
    } else {
        ColorRgba::from_hex(0xFF414141)
    };

    let fill = if ctx.interaction.is_active(&id) && ctx.interaction.is_hot(&id) {
        Fill::Gradient(Gradient::Linear(LinearGradient::vertical(vec![
            ColorRgba::from_hex(0xFF1C1C1C),
            ColorRgba::from_hex(0xFF212121),
        ])))
    } else if ctx.interaction.is_hot(&id) {
        Fill::Gradient(Gradient::Linear(LinearGradient::vertical(vec![
            ColorRgba::from_hex(0xFF383838),
            ColorRgba::from_hex(0xFF2E2E2E),
        ])))
    } else {
        Fill::Gradient(Gradient::Linear(LinearGradient::vertical(vec![
            ColorRgba::from_hex(0xFF2F2F2F),
            ColorRgba::from_hex(0xFF272727),
        ])))
    };

    ctx.push_command(RenderCommand::Rect {
        boundary: placement.rect.offset(0., 1.).px(ctx),
        fill: Fill::Color(ColorRgba::from_hex(0xFF272727)),
        border_radius: BorderRadius::all(3.0.px(ctx)),
        border: Border::all(BorderSide::new(0.0, border_color)),
    });

    ctx.push_command(RenderCommand::Rect {
        boundary: placement.rect.px(ctx),
        fill,
        border_radius: BorderRadius::all(3.0.px(ctx)),
        border: Border::all(BorderSide::new(1.0.px(ctx), border_color)),
    });

    let text_id = cache_string(ctx, state.text, |ctx| {
        let text = ctx.string_interner.resolve(state.text).unwrap();
        ctx.text
            .add_text(ctx.view, ctx.fonts, 12., 12., |fonts, text_res| {
                text_res.set_text(fonts, text)
            })
    });

    let text_size = ctx.text.get_mut(text_id).layout();
    let text_position = position
        + Vec2::new(
            AlignX::Center.position(ctx.layout_direction, size.x, text_size.x),
            AlignX::Center.position(ctx.layout_direction, size.y, text_size.y),
        );

    ctx.push_command(RenderCommand::Text {
        x: text_position.x,
        y: text_position.y,
        text_id,
        tint_color: Some(ColorRgba::from_hex(0xFFFFFFFF)),
    });
}
