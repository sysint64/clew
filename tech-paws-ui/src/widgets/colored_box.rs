use std::any::Any;

use glam::Vec2;

use crate::{
    AlignX, AlignY, Border, BorderRadius, BorderSide, ColorRgba, Constraints, Size, SizeConstraint,
    WidgetId, WidgetRef, WidgetType, impl_width_methods,
    layout::{ContainerKind, LayoutCommand, WidgetPlacement},
    render::{Fill, PixelExtension, RenderCommand, RenderContext, cache_string},
    state::WidgetState,
    text::StringId,
};

use super::builder::BuildContext;

pub struct ColoredBox;

pub struct ColoredBoxBuilder {
    id: WidgetId,
    color: ColorRgba,
    zindex: Option<i32>,
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
}

impl ColoredBoxBuilder {
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

        context.push_layout_command(LayoutCommand::BeginContainer {
            widget_ref: Some(widget_ref),
            zindex: context.current_zindex - 1,
            kind: ContainerKind::ZStack,
            size,
            constraints: Constraints::default(),
        });
        callback(context);
        context.push_layout_command(LayoutCommand::EndContainer);

        context.current_zindex = last_zindex;
        context.widgets_states.accessed_this_frame.insert(id);

        let state = context
            .widgets_states
            .get_or_insert::<State, _>(id, || State { color: self.color });
    }
}

#[track_caller]
pub fn colored_box(color: ColorRgba) -> ColoredBoxBuilder {
    ColoredBoxBuilder {
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
        fill: Fill::Color(state.color),
        border_radius: BorderRadius::all(0.0.px(ctx)),
        border: Border::all(BorderSide::new(0.0, ColorRgba::transparent())),
    });
}
