use std::any::Any;

use crate::{
    Constraints, EdgeInsets, Size, SizeConstraint, WidgetId, WidgetRef, WidgetType, impl_id,
    impl_size_methods,
    interaction::InteractionState,
    io::UserInput,
    layout::{ContainerKind, LayoutCommand, LayoutMeasure},
    state::WidgetState,
};
use std::hash::Hash;

use super::builder::BuildContext;

pub struct ScrollAreaWidget;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    scroll_direction: ScrollDirection,
}

#[derive(Clone, PartialEq)]
pub struct ScrollAreaResponse {
    pub offset_x: f32,
    pub offset_y: f32,
    pub overflow_x: bool,
    pub overflow_y: bool,
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

    pub fn scroll_direction(mut self, scroll_direction: ScrollDirection) -> Self {
        self.scroll_direction = scroll_direction;

        self
    }

    #[profiling::function]
    pub fn build<F>(self, context: &mut BuildContext, callback: F)
    where
        F: FnOnce(&mut BuildContext),
    {
        let id = self.id.with_seed(context.id_seed);
        let widget_ref = WidgetRef::new(WidgetType::of::<ScrollAreaWidget>(), id);

        let last_zindex = context.current_zindex;
        context.current_zindex = self.zindex.unwrap_or(context.current_zindex);
        let mut widget_refs = std::mem::take(context.decorators);
        widget_refs.push(widget_ref);

        let (offset_x, offset_y, response) = {
            let state = context
                .widgets_states
                .scroll_area
                .get_or_insert(id, || State {
                    offset_x: 0.,
                    offset_y: 0.,
                    overflow_x: false,
                    overflow_y: false,
                    scroll_direction: self.scroll_direction,
                });

            state.scroll_direction = self.scroll_direction;

            (
                state.offset_x,
                state.offset_y,
                ScrollAreaResponse {
                    offset_x: state.offset_x,
                    offset_y: state.offset_y,
                    overflow_x: state.overflow_x,
                    overflow_y: state.overflow_y,
                },
            )
        };

        context.push_layout_command(LayoutCommand::BeginContainer {
            decorators: widget_refs,
            zindex: 0,
            padding: self.padding,
            kind: ContainerKind::Measure { id },
            size: self.size,
            constraints: self.constraints,
        });

        context.push_layout_command(LayoutCommand::BeginOffset { offset_x, offset_y });
        context.with_user_data(response, callback);
        context.push_layout_command(LayoutCommand::EndOffset);

        context.push_layout_command(LayoutCommand::EndContainer);

        context.current_zindex = last_zindex;

        context
            .widgets_states
            .scroll_area
            .accessed_this_frame
            .insert(id);
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

pub fn handle_interaction(
    id: WidgetId,
    input: &UserInput,
    interaction: &mut InteractionState,
    widget_state: &mut State,
    layout_measure: &LayoutMeasure,
) {
    if widget_state.scroll_direction == ScrollDirection::Vertical
        || widget_state.scroll_direction == ScrollDirection::Both
    {
        if input.mouse_wheel_delta_y != 0. {
            widget_state.offset_y += input.mouse_wheel_delta_y as f32;
        }

        widget_state.offset_y = widget_state.offset_y.clamp(
            f32::min(0., -(layout_measure.wrap_height - layout_measure.height)),
            0.,
        );
    }

    if widget_state.scroll_direction == ScrollDirection::Horizontal
        || widget_state.scroll_direction == ScrollDirection::Both
    {
        if input.mouse_wheel_delta_x != 0. {
            widget_state.offset_x += input.mouse_wheel_delta_x as f32;
        }

        widget_state.offset_x = widget_state.offset_x.clamp(
            f32::min(0., -(layout_measure.wrap_width - layout_measure.width)),
            0.,
        );
    }
}
