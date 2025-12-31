use std::any::Any;
use std::hash::Hash;

use glam::Vec2;
use smallvec::{SmallVec, smallvec};

use crate::{
    AlignX, AlignY, Border, BorderRadius, BorderSide, BoxShape, ColorRgba, Constraints, EdgeInsets,
    Gradient, LinearGradient, RadialGradient, Size, SizeConstraint, WidgetId, WidgetRef,
    WidgetType, impl_id, impl_size_methods, impl_width_methods,
    layout::{ContainerKind, DeriveWrapSize, LayoutCommand, WidgetPlacement},
    render::{Fill, PixelExtension, RenderCommand, RenderContext, cache_string},
    state::WidgetState,
    text::StringId,
};
// use bumpalo::{Bump, collections::Vec};

use super::builder::BuildContext;

pub struct DecoratedBox;

pub struct DecoratedBoxBuilder {
    id: WidgetId,
    zindex: Option<i32>,
    color: Option<ColorRgba>,
    gradients: SmallVec<[Gradient; 4]>,
    border_radius: Option<BorderRadius>,
    border: Option<Border>,
    shape: BoxShape,
    offset_x: f32,
    offset_y: f32,
}

pub struct DecoratedBoxChildBuilder {
    id: WidgetId,
    size: Size,
    constraints: Constraints,
    padding: EdgeInsets,
    zindex: Option<i32>,
    color: Option<ColorRgba>,
    gradients: SmallVec<[Gradient; 4]>,
    border_radius: Option<BorderRadius>,
    border: Option<Border>,
    shape: BoxShape,
    offset_x: f32,
    offset_y: f32,
}

#[derive(Clone, PartialEq)]
pub struct State {
    shape: BoxShape,
    color: Option<ColorRgba>,
    gradients: SmallVec<[Gradient; 4]>,
    border_radius: Option<BorderRadius>,
    border: Option<Border>,
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

impl DecoratedBoxBuilder {
    impl_id!();

    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset_x = x;
        self.offset_y = y;

        self
    }

    pub fn offset_x(mut self, offset: f32) -> Self {
        self.offset_x = offset;

        self
    }

    pub fn offset_y(mut self, offset: f32) -> Self {
        self.offset_y = offset;

        self
    }

    pub fn color(mut self, color: ColorRgba) -> Self {
        self.color = Some(color);

        self
    }

    pub fn border_radius(mut self, border_radius: BorderRadius) -> Self {
        self.border_radius = Some(border_radius);

        self
    }

    pub fn border(mut self, border: Border) -> Self {
        self.border = Some(border);

        self
    }

    pub fn add_gradient(mut self, gradient: Gradient) -> Self {
        self.gradients.push(gradient);

        self
    }

    pub fn add_linear_gradient(mut self, gradient: LinearGradient) -> Self {
        self.gradients.push(Gradient::Linear(gradient));

        self
    }

    pub fn add_radial_gradient(mut self, gradient: RadialGradient) -> Self {
        self.gradients.push(Gradient::Radial(gradient));

        self
    }

    pub fn shape(mut self, shape: BoxShape) -> Self {
        self.shape = shape;

        self
    }

    fn to_child(self) -> DecoratedBoxChildBuilder {
        DecoratedBoxChildBuilder {
            id: self.id,
            zindex: None,
            size: Size::default(),
            padding: EdgeInsets::ZERO,
            constraints: Constraints {
                min_width: 0.,
                min_height: 0.,
                max_width: f32::INFINITY,
                max_height: f32::INFINITY,
            },
            color: self.color,
            gradients: self.gradients,
            border_radius: self.border_radius,
            border: self.border,
            shape: self.shape,
            offset_x: self.offset_x,
            offset_y: self.offset_y,
        }
    }

    pub fn size<T: Into<Size>>(self, size: T) -> DecoratedBoxChildBuilder {
        self.to_child().size(size)
    }

    pub fn width<T: Into<SizeConstraint>>(mut self, size: T) -> DecoratedBoxChildBuilder {
        self.to_child().width(size)
    }

    pub fn height<T: Into<SizeConstraint>>(mut self, size: T) -> DecoratedBoxChildBuilder {
        self.to_child().height(size)
    }

    pub fn fill_max_width(mut self) -> DecoratedBoxChildBuilder {
        self.to_child().fill_max_width()
    }

    pub fn fill_max_height(mut self) -> DecoratedBoxChildBuilder {
        self.to_child().fill_max_height()
    }

    pub fn fill_max_size(mut self) -> DecoratedBoxChildBuilder {
        self.to_child().fill_max_size()
    }

    pub fn constraints(mut self, constraints: Constraints) -> DecoratedBoxChildBuilder {
        self.to_child().constraints(constraints)
    }

    pub fn max_width(mut self, value: f32) -> DecoratedBoxChildBuilder {
        self.to_child().max_width(value)
    }

    pub fn max_height(mut self, value: f32) -> DecoratedBoxChildBuilder {
        self.to_child().max_height(value)
    }

    pub fn min_width(mut self, value: f32) -> DecoratedBoxChildBuilder {
        self.to_child().min_width(value)
    }

    pub fn min_height(mut self, value: f32) -> DecoratedBoxChildBuilder {
        self.to_child().min_height(value)
    }

    #[profiling::function]
    pub fn build<F>(self, context: &mut BuildContext, callback: F)
    where
        F: FnOnce(&mut BuildContext),
    {
        let id = self.id.with_seed(context.id_seed);

        let last_zindex = context.current_zindex;
        context.current_zindex = self.zindex.unwrap_or(context.current_zindex);
        context.current_zindex += 1;

        let widget_ref = WidgetRef::new(WidgetType::of::<DecoratedBox>(), id);

        context.decorators.push(widget_ref);
        callback(context);

        context.current_zindex = last_zindex;
        context.widgets_states.decorated_box.set(
            id,
            State {
                color: self.color,
                shape: self.shape,
                gradients: self.gradients,
                border_radius: self.border_radius,
                border: self.border,
            },
        );
    }
}

impl DecoratedBoxChildBuilder {
    impl_id!();
    impl_size_methods!();

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;

        self
    }

    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset_x = x;
        self.offset_y = y;

        self
    }

    pub fn offset_x(mut self, offset: f32) -> Self {
        self.offset_x = offset;

        self
    }

    pub fn offset_y(mut self, offset: f32) -> Self {
        self.offset_y = offset;

        self
    }

    pub fn build(&self, context: &mut BuildContext) {
        let id = self.id.with_seed(context.id_seed);
        let widget_ref = WidgetRef::new(WidgetType::of::<DecoratedBox>(), id);
        let decorators = std::mem::take(context.decorators);

        if self.offset_x != 0. || self.offset_y != 0. {
            context.push_layout_command(LayoutCommand::BeginOffset {
                offset_x: self.offset_x,
                offset_y: self.offset_y,
            });
        }

        context.push_layout_command(LayoutCommand::Child {
            widget_ref,
            decorators,
            padding: self.padding,
            constraints: self.constraints,
            size: self.size,
            zindex: self.zindex.unwrap_or(context.current_zindex),
            derive_wrap_size: DeriveWrapSize::Constraints,
        });

        if self.offset_x != 0. || self.offset_y != 0. {
            context.push_layout_command(LayoutCommand::EndOffset);
        }

        context.widgets_states.decorated_box.set(
            id,
            State {
                color: self.color,
                shape: self.shape,
                gradients: self.gradients.clone(),
                border_radius: self.border_radius,
                border: self.border,
            },
        );
    }
}

#[track_caller]
pub fn decorated_box() -> DecoratedBoxBuilder {
    DecoratedBoxBuilder {
        id: WidgetId::auto(),
        zindex: None,
        color: None,
        gradients: smallvec![],
        border_radius: None,
        border: None,
        shape: BoxShape::rect,
        offset_x: 0.,
        offset_y: 0.,
    }
}

pub fn render(ctx: &mut RenderContext, placement: &WidgetPlacement, state: &State) {
    let id = placement.widget_ref.id;
    let size = placement.rect.size().px(ctx);
    let position = placement.rect.position().px(ctx);

    match state.shape {
        BoxShape::rect => {
            if let Some(color) = state.color {
                ctx.push_command(RenderCommand::Rect {
                    zindex: placement.zindex,
                    boundary: placement.rect.px(ctx),
                    fill: Some(Fill::Color(color)),
                    border_radius: state.border_radius.map(|it| it.px(ctx)),
                    border: state.border.map(|it| it.px(ctx)),
                });
            }

            for gradient in &state.gradients {
                ctx.push_command(RenderCommand::Rect {
                    zindex: placement.zindex,
                    boundary: placement.rect.px(ctx),
                    fill: Some(Fill::Gradient(gradient.clone())),
                    border_radius: state.border_radius.map(|it| it.px(ctx)),
                    border: state.border.map(|it| it.px(ctx)),
                });
            }
        }
        BoxShape::oval => {
            let border = state.border.map(|it| it.px(ctx)).map(|it| {
                it.top
                    .or(it.bottom)
                    .or(it.left)
                    .or(it.right)
                    .unwrap_or(BorderSide::default())
            });

            if let Some(color) = state.color {
                ctx.push_command(RenderCommand::Oval {
                    zindex: placement.zindex,
                    boundary: placement.rect.px(ctx),
                    fill: Some(Fill::Color(color)),
                    border: border,
                });
            }

            for gradient in &state.gradients {
                ctx.push_command(RenderCommand::Oval {
                    zindex: placement.zindex,
                    boundary: placement.rect.px(ctx),
                    fill: Some(Fill::Gradient(gradient.clone())),
                    border: border,
                });
            }
        }
    }
}
