use glam::Vec2;

use super::builder::BuildContext;
use crate::{
    AlignX, AlignY, Border, BorderRadius, BorderSide, ColorRgba, Constraints, EdgeInsets, Gradient,
    LinearGradient, Size, SizeConstraint, WidgetId, WidgetRef, WidgetType, impl_id,
    impl_position_methods, impl_width_methods,
    interaction::InteractionState,
    io::UserInput,
    layout::{ContainerKind, DeriveWrapSize, LayoutCommand, WidgetPlacement},
    render::{Fill, PixelExtension, RenderCommand, RenderContext, cache_string},
    state::WidgetState,
    text::{StringId, TextId},
};
use std::{any::Any, hash::Hash, rc::Rc};

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
    pub(crate) text_id: TextId,
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
        let size = Size::new(self.width, SizeConstraint::Wrap);
        let widget_ref = WidgetRef::new(WidgetType::of::<ButtonWidget>(), id);
        let text_id = cache_string(context, text, |ctx| {
            let text = ctx.string_interner.resolve(text).unwrap();
            ctx.text
                .add_text(ctx.view, ctx.fonts, 12., 12., |fonts, text_res| {
                    text_res.set_text(fonts, text)
                })
        });

        if let Some(padding) = self.padding {
            let mut padding_containts = self.constraints;
            padding_containts.expand(padding);

            context.push_layout_command(LayoutCommand::BeginContainer {
                widget_ref: vec![],
                zindex: 0,
                kind: ContainerKind::Padding { padding },
                size,
                constraints: self.constraints,
            });

            context.with_align(self.align_x, self.align_y, |context| {
                context.push_layout_command(LayoutCommand::Child {
                    widget_refs: vec![widget_ref],
                    constraints: self.constraints,
                    size,
                    zindex: self.zindex.unwrap_or(context.current_zindex),
                    derive_wrap_size: DeriveWrapSize::Text {
                        padding: EdgeInsets::symmetric(8., 4.),
                        text_id,
                    },
                });
            });

            context.push_layout_command(LayoutCommand::EndContainer);
        } else {
            context.with_align(self.align_x, self.align_y, |context| {
                context.push_layout_command(LayoutCommand::Child {
                    widget_refs: vec![widget_ref],
                    constraints: self.constraints,
                    size,
                    derive_wrap_size: DeriveWrapSize::Text {
                        padding: EdgeInsets::symmetric(8., 4.),
                        text_id,
                    },
                    zindex: self.zindex.unwrap_or(context.current_zindex),
                });
            });
        }

        context.widgets_states.accessed_this_frame.insert(id);

        let mut state = context
            .widgets_states
            .get_or_insert::<State, _>(id, || State {
                clicked: false,
                text_id,
            });

        state.text_id = text_id;

        ButtonResponse {
            clicked: state.clicked,
        }
    }
}

#[track_caller]
pub fn button(text: &str) -> ButtonBuilder<'_> {
    ButtonBuilder {
        id: WidgetId::auto(),
        text,
        width: SizeConstraint::Wrap,
        // width: SizeConstraint::Fixed(100.),
        align_x: None,
        align_y: None,
        padding: None,
        zindex: None,
        constraints: Constraints {
            min_width: 20.,
            min_height: 20.,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
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
        zindex: placement.zindex,
        boundary: placement.rect.offset(0., 1.).px(ctx),
        fill: Some(Fill::Color(ColorRgba::from_hex(0xFF272727))),
        border_radius: Some(BorderRadius::all(3.0.px(ctx))),
        border: None,
    });

    ctx.push_command(RenderCommand::Rect {
        zindex: placement.zindex,
        boundary: placement.rect.px(ctx),
        fill: Some(fill),
        border_radius: Some(BorderRadius::all(3.0.px(ctx))),
        border: Some(Border::all(BorderSide::new(1.0.px(ctx), border_color))),
    });

    let text_size = ctx.text.get_mut(state.text_id).layout();
    let text_position = position
        + Vec2::new(
            AlignX::Center.position(ctx.layout_direction, size.x, text_size.x),
            AlignX::Center.position(ctx.layout_direction, size.y, text_size.y),
        );

    ctx.push_command(RenderCommand::Text {
        zindex: placement.zindex,
        x: text_position.x,
        y: text_position.y,
        text_id: state.text_id,
        tint_color: Some(ColorRgba::from_hex(0xFFFFFFFF)),
    });
}
