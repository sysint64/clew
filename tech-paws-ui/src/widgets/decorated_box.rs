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
    size: Size,
    constraints: Constraints,
    zindex: Option<i32>,
    padding: EdgeInsets,
    margin: EdgeInsets,
    offset_x: f32,
    offset_y: f32,

    color: Option<ColorRgba>,
    gradients: SmallVec<[Gradient; 4]>,
    border_radius: Option<BorderRadius>,
    border: Option<Border>,
    shape: BoxShape,
}

pub struct DecorationBuilder {
    id: WidgetId,
    color: Option<ColorRgba>,
    gradients: SmallVec<[Gradient; 4]>,
    border_radius: Option<BorderRadius>,
    border: Option<Border>,
    shape: BoxShape,
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

impl DecorationBuilder {
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

    pub fn build(self, context: &mut BuildContext) -> WidgetRef {
        let id = self.id.with_seed(context.id_seed);

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

        WidgetRef::new(WidgetType::of::<DecoratedBox>(), id)
    }
}

impl DecoratedBoxBuilder {
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

    #[profiling::function]
    pub fn build(self, context: &mut BuildContext) {
        let id = self.id.with_seed(context.id_seed);
        let widget_ref = WidgetRef::new(WidgetType::of::<DecoratedBox>(), id);
        let backgrounds = std::mem::take(context.decorators);

        if self.offset_x != 0. || self.offset_y != 0. {
            context.push_layout_command(LayoutCommand::BeginOffset {
                offset_x: self.offset_x,
                offset_y: self.offset_y,
            });
        }

        context.push_layout_command(LayoutCommand::Child {
            widget_ref,
            backgrounds,
            padding: self.padding,
            margin: self.margin,
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
        size: Size::default(),
        constraints: Constraints::default(),
        padding: EdgeInsets::ZERO,
        margin: EdgeInsets::ZERO,
    }
}

#[track_caller]
pub fn decoration() -> DecorationBuilder {
    DecorationBuilder {
        id: WidgetId::auto(),
        color: None,
        gradients: smallvec![],
        border_radius: None,
        border: None,
        shape: BoxShape::rect,
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
