use std::any::Any;

use glam::Vec2;

use crate::{
    AlignX, AlignY, ColorRgba, Constraints, EdgeInsets, Size, SizeConstraint, WidgetId, WidgetRef,
    WidgetType, impl_size_methods, impl_width_methods,
    layout::{DeriveWrapSize, LayoutCommand, WidgetPlacement},
    render::{PixelExtension, RenderCommand, RenderContext, cache_string},
    state::WidgetState,
};

use super::builder::BuildContext;

pub struct SvgWidget;

pub struct SvgBuilder {
    id: WidgetId,
    asset_id: &'static str,
    size: Size,
    constraints: Constraints,
    zindex: Option<i32>,
    color: Option<ColorRgba>,
    padding: EdgeInsets,
    margin: EdgeInsets,
}

#[derive(Clone, PartialEq)]
pub struct State {
    pub(crate) asset_id: &'static str,
    pub(crate) color: Option<ColorRgba>,
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

impl SvgBuilder {
    impl_size_methods!();

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;

        self
    }

    pub fn margin(mut self, margin: EdgeInsets) -> Self {
        self.margin = margin;

        self
    }

    pub fn color(mut self, color: ColorRgba) -> Self {
        self.color = Some(color);

        self
    }

    pub fn build(&self, context: &mut BuildContext) {
        let id = self.id.with_seed(context.id_seed);

        let widget_ref = WidgetRef::new(WidgetType::of::<SvgWidget>(), id);
        let decorators = std::mem::take(context.decorators);

        context.push_layout_command(LayoutCommand::Child {
            widget_ref,
            backgrounds: decorators,
            padding: self.padding,
            margin: self.margin,
            constraints: self.constraints,
            size: self.size,
            zindex: self.zindex.unwrap_or(context.current_zindex),
            derive_wrap_size: DeriveWrapSize::Svg(self.asset_id),
        });

        let state = context.widgets_states.svg.set(
            id,
            State {
                asset_id: self.asset_id,
                color: self.color,
            },
        );
    }
}

#[track_caller]
pub fn svg(asset_id: &'static str) -> SvgBuilder {
    SvgBuilder {
        id: WidgetId::auto(),
        asset_id,
        color: None,
        size: Size::default(),
        zindex: None,
        constraints: Constraints {
            min_width: 0.,
            min_height: 0.,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
        },
        padding: EdgeInsets::ZERO,
        margin: EdgeInsets::ZERO,
    }
}

pub fn render(ctx: &mut RenderContext, placement: &WidgetPlacement, state: &State) {
    let id = placement.widget_ref.id;
    let size = placement.rect.size().px(ctx);
    let position = placement.rect.position().px(ctx);

    ctx.push_command(RenderCommand::Svg {
        zindex: placement.zindex,
        boundary: placement.rect.px(ctx),
        asset_id: state.asset_id,
        tint_color: state.color,
    });
}
