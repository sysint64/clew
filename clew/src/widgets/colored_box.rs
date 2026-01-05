use std::any::Any;
use std::hash::Hash;

use crate::{
    Clip, ColorRgba, Constraints, EdgeInsets, Size, SizeConstraint, WidgetId, WidgetRef,
    WidgetType, impl_id, impl_size_methods,
    layout::{DeriveWrapSize, LayoutCommand, WidgetPlacement},
    render::{Fill, PixelExtension, RenderCommand, RenderContext},
    state::WidgetState,
};

use smallvec::smallvec;
use super::builder::BuildContext;

pub struct ColoredBox;

pub struct ColoredBoxDecoratorBuilder {
    id: WidgetId,
    color: ColorRgba,
    zindex: Option<i32>,
}

pub struct ColoredBoxChildBuilder {
    id: WidgetId,
    size: Size,
    constraints: Constraints,
    color: ColorRgba,
    zindex: Option<i32>,
    padding: EdgeInsets,
    clip: Clip,
}

#[derive(Clone, PartialEq)]
pub struct State {
    pub(crate) color: ColorRgba,
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

impl ColoredBoxDecoratorBuilder {
    impl_id!();

    pub fn build<F>(&self, context: &mut BuildContext, callback: F)
    where
        F: FnOnce(&mut BuildContext),
    {
        let id = self.id.with_seed(context.id_seed);

        let last_zindex = context.current_zindex;
        context.current_zindex = self.zindex.unwrap_or(context.current_zindex);
        context.current_zindex += 1;

        let widget_ref = WidgetRef::new(WidgetType::of::<ColoredBox>(), id);

        context.backgrounds.push(widget_ref);
        callback(context);

        context.current_zindex = last_zindex;
        context
            .widgets_states
            .colored_box
            .set(id, State { color: self.color });
    }
}

impl ColoredBoxChildBuilder {
    impl_id!();
    impl_size_methods!();

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;

        self
    }

    pub fn build(&self, context: &mut BuildContext) {
        let id = self.id.with_seed(context.id_seed);
        let widget_ref = WidgetRef::new(WidgetType::of::<ColoredBox>(), id);
        let decorators = std::mem::take(context.backgrounds);

        context.push_layout_command(LayoutCommand::Leaf {
            widget_ref,
            backgrounds: decorators,
            foregrounds: smallvec![],
            padding: self.padding,
            margin: EdgeInsets::ZERO,
            constraints: self.constraints,
            size: self.size,
            zindex: self.zindex.unwrap_or(context.current_zindex),
            derive_wrap_size: DeriveWrapSize::Constraints,
            clip: self.clip,
        });

        context
            .widgets_states
            .colored_box
            .set(id, State { color: self.color });
    }
}

#[track_caller]
pub fn colored_box(color: ColorRgba) -> ColoredBoxDecoratorBuilder {
    ColoredBoxDecoratorBuilder {
        id: WidgetId::auto(),
        color,
        zindex: None,
    }
}

pub fn render(ctx: &mut RenderContext, placement: &WidgetPlacement, state: &State) {
    ctx.push_command(RenderCommand::Rect {
        zindex: placement.zindex,
        boundary: placement.rect.px(ctx),
        fill: Some(Fill::Color(state.color)),
        border_radius: None,
        border: None,
    });
}
