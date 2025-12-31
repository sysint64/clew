use crate::{
    Constraints, Size, SizeConstraint, WidgetId, WidgetRef, WidgetType, impl_id,
    interaction::InteractionState,
    io::UserInput,
    layout::{ContainerKind, LayoutCommand},
    state::WidgetState,
};
use std::{any::Any, hash::Hash};

use super::builder::BuildContext;

pub struct GestureDetectorBuilder {
    id: WidgetId,
    focusable: bool,
    clickable: bool,
    dragable: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct State {
    clicked: bool,
    is_active: bool,
    is_hot: bool,
    is_focused: bool,
    clickable: bool,
    dragable: bool,
    focusable: bool,
    drag_start_x: f32,
    drag_start_y: f32,
    drag_x: f32,
    drag_y: f32,
    drag_delta_x: f32,
    drag_delta_y: f32,
    drag_state: DragState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DragState {
    #[default]
    None,
    Start,
    Update,
    End,
}

pub struct GestureDetector;

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

#[derive(Clone)]
pub struct GestureDetectorResponse {
    clicked: bool,
    is_active: bool,
    is_hot: bool,
    is_focused: bool,
    drag_start_x: f32,
    drag_start_y: f32,
    drag_x: f32,
    drag_y: f32,
    drag_delta_x: f32,
    drag_delta_y: f32,
    drag_state: DragState,
}

impl GestureDetectorResponse {
    #[inline]
    pub fn clicked(&self) -> bool {
        self.clicked
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    #[inline]
    pub fn is_hot(&self) -> bool {
        self.is_hot
    }

    #[inline]
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }
}

impl GestureDetectorBuilder {
    impl_id!();

    pub fn clickable(mut self, value: bool) -> Self {
        self.clickable = value;

        self
    }

    pub fn focusable(mut self, value: bool) -> Self {
        self.focusable = value;

        self
    }

    pub fn dragable(mut self, value: bool) -> Self {
        self.dragable = value;

        self
    }

    #[profiling::function]
    pub fn build<F>(self, context: &mut BuildContext, callback: F) -> GestureDetectorResponse
    where
        F: FnOnce(&mut BuildContext),
    {
        let id = self.id.with_seed(context.id_seed);
        let widget_ref = WidgetRef::new(WidgetType::of::<GestureDetector>(), id);

        let state = context
            .widgets_states
            .gesture_detector
            .get_or_insert(id, State::default);

        state.clickable = self.clickable;
        state.dragable = self.dragable;
        state.focusable = self.focusable;

        let response = GestureDetectorResponse {
            clicked: state.clicked,
            is_active: state.is_active,
            is_hot: state.is_hot,
            is_focused: state.is_focused,
            drag_start_x: state.drag_start_x,
            drag_start_y: state.drag_start_y,
            drag_x: state.drag_x,
            drag_y: state.drag_y,
            drag_delta_x: state.drag_delta_x,
            drag_delta_y: state.drag_delta_y,
            drag_state: state.drag_state,
        };

        context.decorators.push(widget_ref);
        context.with_user_data(response.clone(), callback);

        context
            .widgets_states
            .gesture_detector
            .accessed_this_frame
            .insert(id);

        response
    }
}

#[track_caller]
pub fn gesture_detector() -> GestureDetectorBuilder {
    GestureDetectorBuilder {
        id: WidgetId::auto(),
        clickable: false,
        dragable: false,
        focusable: false,
    }
}

pub fn handle_interaction(
    id: WidgetId,
    input: &UserInput,
    interaction: &mut InteractionState,
    widget_state: &mut State,
) {
    widget_state.clicked = false;

    if widget_state.clickable {
        if interaction.is_active(&id) {
            if input.mouse_released {
                if interaction.is_hot(&id) {
                    interaction.set_inactive(&id);
                    widget_state.clicked = true;

                    if widget_state.focusable {
                        interaction.focused = Some(id);
                    }
                } else {
                    interaction.set_inactive(&id);
                }
            }
        } else if input.mouse_left_pressed && interaction.is_hot(&id) {
            if widget_state.focusable {
                interaction.focused = Some(id);
            }

            interaction.set_active(&id);
        }
    }

    if widget_state.dragable {}

    widget_state.is_active = interaction.is_active(&id);
    widget_state.is_hot = interaction.is_hot(&id);
    widget_state.is_focused = interaction.is_focused(&id);
}
