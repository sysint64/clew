use std::any::Any;
use std::hash::Hash;

use glam::Vec2;

use crate::{
    AlignX, AlignY, Border, BorderRadius, BorderSide, ColorRgba, Constraints, EdgeInsets, Size,
    SizeConstraint, WidgetId, WidgetRef, WidgetType, impl_id, impl_size_methods,
    impl_width_methods,
    layout::{ContainerKind, DeriveWrapSize, LayoutCommand, WidgetPlacement},
    render::{Fill, PixelExtension, RenderCommand, RenderContext, cache_string},
    state::WidgetState,
    text::StringId,
};

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

    fn to_child(self) -> ColoredBoxChildBuilder {
        ColoredBoxChildBuilder {
            id: self.id,
            color: self.color,
            zindex: None,
            size: Size::default(),
            padding: EdgeInsets::ZERO,
            constraints: Constraints {
                min_width: 0.,
                min_height: 0.,
                max_width: f32::INFINITY,
                max_height: f32::INFINITY,
            },
        }
    }

    pub fn size<T: Into<Size>>(self, size: T) -> ColoredBoxChildBuilder {
        self.to_child().size(size)
    }

    pub fn width<T: Into<SizeConstraint>>(mut self, size: T) -> ColoredBoxChildBuilder {
        self.to_child().width(size)
    }

    pub fn height<T: Into<SizeConstraint>>(mut self, size: T) -> ColoredBoxChildBuilder {
        self.to_child().height(size)
    }

    pub fn fill_max_width(mut self) -> ColoredBoxChildBuilder {
        self.to_child().fill_max_width()
    }

    pub fn fill_max_height(mut self) -> ColoredBoxChildBuilder {
        self.to_child().fill_max_height()
    }

    pub fn fill_max_size(mut self) -> ColoredBoxChildBuilder {
        self.to_child().fill_max_size()
    }

    pub fn constraints(mut self, constraints: Constraints) -> ColoredBoxChildBuilder {
        self.to_child().constraints(constraints)
    }

    pub fn max_width(mut self, value: f32) -> ColoredBoxChildBuilder {
        self.to_child().max_width(value)
    }

    pub fn max_height(mut self, value: f32) -> ColoredBoxChildBuilder {
        self.to_child().max_height(value)
    }

    pub fn min_width(mut self, value: f32) -> ColoredBoxChildBuilder {
        self.to_child().min_width(value)
    }

    pub fn min_height(mut self, value: f32) -> ColoredBoxChildBuilder {
        self.to_child().min_height(value)
    }

    pub fn build<F>(&self, context: &mut BuildContext, callback: F)
    where
        F: FnOnce(&mut BuildContext),
    {
        let id = self.id.with_seed(context.id_seed);

        let last_zindex = context.current_zindex;
        context.current_zindex = self.zindex.unwrap_or(context.current_zindex);
        context.current_zindex += 1;

        let size = Size::new(SizeConstraint::Wrap, SizeConstraint::Wrap);
        let widget_ref = WidgetRef::new(WidgetType::of::<ColoredBox>(), id);

        context.decorators.push(widget_ref);
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
        let decorators = std::mem::take(context.decorators);

        context.push_layout_command(LayoutCommand::Child {
            widget_ref,
            decorators,
            padding: self.padding,
            constraints: self.constraints,
            size: self.size,
            zindex: self.zindex.unwrap_or(context.current_zindex),
            derive_wrap_size: DeriveWrapSize::Constraints,
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
    let id = placement.widget_ref.id;
    let size = placement.rect.size().px(ctx);
    let position = placement.rect.position().px(ctx);

    ctx.push_command(RenderCommand::Rect {
        zindex: placement.zindex,
        boundary: placement.rect.px(ctx),
        fill: Some(Fill::Color(state.color)),
        border_radius: None,
        border: None,
    });
}
