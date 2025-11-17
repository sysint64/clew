use std::any::Any;

use glam::Vec2;

use crate::{
    AlignX, AlignY, ColorRgba, Constraints, Size, SizeConstraint, WidgetId, WidgetRef, WidgetType,
    impl_width_methods,
    layout::{LayoutCommand, WidgetPlacement},
    render::{PixelExtension, RenderCommand, RenderContext, cache_string},
    state::WidgetState,
    text::StringId,
};

use super::builder::BuildContext;

pub struct TextWidget;

pub struct TextBuilder<'a> {
    id: WidgetId,
    text: &'a str,
    width: SizeConstraint,
    constraints: Constraints,
    zindex: Option<i32>,
    color: ColorRgba,
    text_align_x: AlignX,
    text_align_y: AlignY,
}

#[derive(Clone, PartialEq)]
pub struct State {
    pub(crate) text: StringId,
    pub(crate) color: ColorRgba,
    pub(crate) text_align_x: AlignX,
    pub(crate) text_align_y: AlignY,
}

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

impl<'a> TextBuilder<'a> {
    impl_width_methods!();

    pub fn color(mut self, color: ColorRgba) -> Self {
        self.color = color;

        self
    }

    pub fn text_align_x(mut self, text_align_x: AlignX) -> Self {
        self.text_align_x = text_align_x;

        self
    }

    pub fn text_align_y(mut self, text_align_y: AlignY) -> Self {
        self.text_align_y = text_align_y;

        self
    }

    pub fn build(&self, context: &mut BuildContext) {
        let id = self.id.with_seed(context.id_seed);

        let text = context.string_interner.get_or_intern(self.text);
        let size = Size::new(self.width, SizeConstraint::Fixed(20.0));
        let widget_ref = WidgetRef::new(WidgetType::of::<TextWidget>(), id);

        println!("{}", context.current_zindex);

        context.push_layout_command(LayoutCommand::Fixed {
            widget_ref,
            constraints: self.constraints,
            size,
            zindex: self.zindex.unwrap_or(context.current_zindex),
        });

        context.widgets_states.accessed_this_frame.insert(id);

        let state = context
            .widgets_states
            .get_or_insert::<State, _>(id, || State {
                text,
                color: self.color,
                text_align_x: self.text_align_x,
                text_align_y: self.text_align_y,
            });

        state.text = text;
    }
}

#[track_caller]
pub fn text(text: &str) -> TextBuilder<'_> {
    TextBuilder {
        id: WidgetId::auto(),
        text,
        color: ColorRgba::from_hex(0xFFFFFFFF),
        width: SizeConstraint::Wrap,
        zindex: None,
        constraints: Constraints {
            min_width: 100.,
            min_height: 20.,
            max_width: f32::INFINITY,
            max_height: 20.,
        },
        text_align_x: AlignX::Start,
        text_align_y: AlignY::Top,
    }
}

pub fn render(ctx: &mut RenderContext, placement: &WidgetPlacement, state: &State) {
    let id = placement.widget_ref.id;
    let size = placement.rect.size().px(ctx);
    let position = placement.rect.position().px(ctx);

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
            state
                .text_align_x
                .position(ctx.layout_direction, size.x, text_size.x),
            state.text_align_y.position(size.y, text_size.y),
        );

    ctx.push_command(RenderCommand::Text {
        zindex: placement.zindex,
        x: text_position.x,
        y: text_position.y,
        text_id,
        tint_color: Some(state.color),
    });
}
