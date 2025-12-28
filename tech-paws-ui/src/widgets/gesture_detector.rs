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
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    clicked: bool,
    is_active: bool,
    is_hot: bool,
    is_focused: bool,
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
}

#[derive(Clone)]
pub struct GestureDetectorResponse {
    clicked: bool,
    is_active: bool,
    is_hot: bool,
    is_focused: bool,
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
            .get_or_insert(id, || State {
                clicked: false,
                is_active: false,
                is_hot: false,
                is_focused: false,
            });

        let response = GestureDetectorResponse {
            clicked: state.clicked,
            is_active: state.is_active,
            is_hot: state.is_hot,
            is_focused: state.is_focused,
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
    }
}

pub fn handle_interaction(
    id: WidgetId,
    input: &UserInput,
    interaction: &mut InteractionState,
    widget_state: &mut State,
) {
    widget_state.clicked = false;

    if interaction.is_active(&id) {
        if input.mouse_released {
            if interaction.is_hot(&id) {
                interaction.set_inactive(&id);
                interaction.focused = Some(id);
                widget_state.clicked = true;
            } else {
                interaction.set_inactive(&id);
            }
        }
    } else if input.mouse_left_pressed && interaction.is_hot(&id) {
        interaction.focused = Some(id);
        interaction.set_active(&id);
    }

    widget_state.is_active = interaction.is_active(&id);
    widget_state.is_hot = interaction.is_hot(&id);
    widget_state.is_focused = interaction.is_focused(&id);
}
