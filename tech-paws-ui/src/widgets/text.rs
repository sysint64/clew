use std::any::Any;

use glam::Vec2;
use smallvec::{SmallVec, smallvec};

use crate::{
    AlignX, AlignY, ColorRgba, Constraints, EdgeInsets, Size, SizeConstraint, TextAlign, WidgetId,
    WidgetRef, WidgetType, impl_id, impl_size_methods, impl_width_methods,
    layout::{DeriveWrapSize, LayoutCommand, WidgetPlacement},
    render::{PixelExtension, RenderCommand, RenderContext, cache_string},
    state::WidgetState,
    text::{StringId, TextId},
};

use super::builder::BuildContext;
use std::hash::Hash;

pub struct TextWidget;

pub struct TextBuilder<'a> {
    id: WidgetId,
    text: &'a str,
    size: Size,
    constraints: Constraints,
    zindex: Option<i32>,
    color: ColorRgba,
    backgrounds: SmallVec<[WidgetRef; 8]>,
    text_align_x: AlignX,
    vertical_align: AlignY,
    text_align: TextAlign,
    padding: EdgeInsets,
    margin: EdgeInsets,
}

#[derive(Clone, PartialEq)]
pub struct State {
    pub(crate) text_id: TextId,
    pub(crate) text_data: String,
    pub(crate) color: ColorRgba,
    pub(crate) text_align: TextAlign,
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

    #[inline]
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl<'a> TextBuilder<'a> {
    impl_id!();
    impl_size_methods!();

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;

        self
    }

    pub fn margin(mut self, margin: EdgeInsets) -> Self {
        self.margin = margin;

        self
    }

    pub fn background(mut self, decorator: WidgetRef) -> Self {
        self.backgrounds.push(decorator);

        self
    }

    pub fn color(mut self, color: ColorRgba) -> Self {
        self.color = color;

        self
    }

    pub fn text_align(mut self, text_align: TextAlign) -> Self {
        self.text_align = text_align;

        self
    }

    pub fn text_align_x(mut self, text_align_x: AlignX) -> Self {
        self.text_align_x = text_align_x;

        self
    }

    pub fn text_align_y(mut self, text_align_y: AlignY) -> Self {
        self.vertical_align = text_align_y;

        self
    }

    #[profiling::function]
    pub fn build(mut self, context: &mut BuildContext) {
        let id = self.id.with_seed(context.id_seed);

        let widget_ref = WidgetRef::new(WidgetType::of::<TextWidget>(), id);
        let state = context.widgets_states.text.get(id);
        let last_text_align = state.map(|it| it.text_align).unwrap_or(TextAlign::Left);

        let (text_data, text_id) = if let Some(state) = state {
            if state.text_data != self.text {
                context.text.update_text(state.text_id, |text| {
                    text.set_text(context.fonts, self.text);
                });

                // Reset wrap size calculation during layout.
                if !self.size.width.constrained() {
                    let text = context.text.get_mut(state.text_id);
                    text.with_buffer_mut(|buffer| {
                        buffer.set_size(&mut context.fonts.font_system, None, None);
                    });
                }

                (Some(self.text.to_string()), state.text_id)
            } else {
                (None, state.text_id)
            }
        } else {
            let text_id =
                context
                    .text
                    .add_text(context.view, context.fonts, 12., 12., |fonts, text_res| {
                        text_res.set_text(fonts, self.text)
                    });

            (Some(self.text.to_string()), text_id)
        };

        if last_text_align != self.text_align {
            let text = context.text.get_mut(text_id);
            text.with_buffer_mut(|buffer| {
                for line in buffer.lines.iter_mut() {
                    line.set_align(match self.text_align {
                        TextAlign::Auto => None,
                        TextAlign::Left => Some(cosmic_text::Align::Left),
                        TextAlign::Right => Some(cosmic_text::Align::Right),
                        TextAlign::End => Some(cosmic_text::Align::End),
                        TextAlign::Center => Some(cosmic_text::Align::Center),
                        TextAlign::Justified => Some(cosmic_text::Align::Justified),
                    });
                }
            });
        }

        let mut backgrounds = std::mem::take(context.decorators);
        backgrounds.append(&mut self.backgrounds);

        context.push_layout_command(LayoutCommand::Child {
            widget_ref,
            backgrounds,
            padding: self.padding,
            margin: self.margin,
            constraints: self.constraints,
            size: self.size,
            zindex: self.zindex.unwrap_or(context.current_zindex),
            derive_wrap_size: DeriveWrapSize::Text(text_id),
        });

        context.widgets_states.text.accessed_this_frame.insert(id);

        let state = context.widgets_states.text.get_or_insert(id, || State {
            text_id: text_id,
            text_data: text_data.clone().unwrap(),
            color: self.color,
            text_align: self.text_align,
            text_align_x: self.text_align_x,
            text_align_y: self.vertical_align,
        });

        if let Some(text_data) = text_data {
            state.text_data = text_data;
        }

        state.color = self.color;
        state.text_align = self.text_align;
    }
}

#[track_caller]
pub fn text(text: &str) -> TextBuilder<'_> {
    TextBuilder {
        id: WidgetId::auto(),
        text,
        color: ColorRgba::from_hex(0xFFFFFFFF),
        backgrounds: smallvec![],
        size: Size::default(),
        zindex: None,
        constraints: Constraints {
            min_width: 100.,
            min_height: 20.,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
        },
        text_align_x: AlignX::Start,
        vertical_align: AlignY::Top,
        text_align: TextAlign::Left,
        padding: EdgeInsets::ZERO,
        margin: EdgeInsets::ZERO,
    }
}

pub fn render(ctx: &mut RenderContext, placement: &WidgetPlacement, state: &State) {
    let id = placement.widget_ref.id;
    let size = placement.rect.size().px(ctx);
    let position = placement.rect.position().px(ctx);

    let text = ctx.text.get_mut(state.text_id);
    let text_size = text.layout();
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
        text_id: state.text_id,
        tint_color: Some(state.color),
    });
}
