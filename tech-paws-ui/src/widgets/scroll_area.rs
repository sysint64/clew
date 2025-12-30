use std::any::Any;

use crate::{
    Constraints, EdgeInsets, Size, SizeConstraint, WidgetId, impl_id, impl_size_methods,
    layout::{ContainerKind, LayoutCommand},
    state::WidgetState,
};
use std::hash::Hash;

use super::builder::BuildContext;

pub struct ScrollViewWidget;

#[derive(Clone, Copy, Debug)]
pub enum ScrollDirection {
    Horizontal,
    Vertical,
    Both,
}

pub struct ScrollAreaBuilder {
    id: WidgetId,
    size: Size,
    constraints: Constraints,
    zindex: Option<i32>,
    padding: EdgeInsets,
    scroll_direction: ScrollDirection,
}

#[derive(Clone, PartialEq)]
pub struct State {
    offset_x: f32,
    offset_y: f32,
    overflow_x: bool,
    overflow_y: bool,
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

impl ScrollAreaBuilder {
    impl_id!();
    impl_size_methods!();

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;

        self
    }

    #[profiling::function]
    pub fn build<F>(self, context: &mut BuildContext, callback: F)
    where
        F: FnOnce(&mut BuildContext),
    {
        let id = self.id.with_seed(context.id_seed);

        let last_zindex = context.current_zindex;
        context.current_zindex = self.zindex.unwrap_or(context.current_zindex);
        let widget_refs = std::mem::take(context.decorators);

        context.push_layout_command(LayoutCommand::BeginOffset {
            offset_x: 0.,
            offset_y: -100.,
        });
        callback(context);
        context.push_layout_command(LayoutCommand::EndOffset);

        context.current_zindex = last_zindex;

        context
            .widgets_states
            .scroll_area
            .accessed_this_frame
            .insert(id);

        let state = context
            .widgets_states
            .scroll_area
            .get_or_insert(id, || State {
                offset_x: 0.,
                offset_y: 0.,
                overflow_x: false,
                overflow_y: false,
            });
    }
}

#[track_caller]
pub fn scroll_area() -> ScrollAreaBuilder {
    ScrollAreaBuilder {
        id: WidgetId::auto(),
        size: Size::default(),
        constraints: Constraints::default(),
        zindex: None,
        padding: EdgeInsets::ZERO,
        scroll_direction: ScrollDirection::Vertical,
    }
}
