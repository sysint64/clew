use std::collections::HashSet;

use crate::{
    WidgetId,
    io::UserInput,
    state::UiState,
    text::{FontResources, TextsResources},
};

#[derive(Default)]
pub struct InteractionState {
    pub(crate) hover: HashSet<WidgetId>,
    pub(crate) hot: Option<WidgetId>,
    pub(crate) active: Option<WidgetId>,
    pub(crate) focused: Option<WidgetId>,
    pub(crate) was_focused: Option<WidgetId>,
}

impl InteractionState {
    pub fn _is_hover(&self, id: &WidgetId) -> bool {
        self.hover.contains(id)
    }

    pub(crate) fn is_hot(&self, id: &WidgetId) -> bool {
        self.hot == Some(*id)
    }

    pub(crate) fn is_active(&self, id: &WidgetId) -> bool {
        self.active == Some(*id)
    }

    pub(crate) fn is_focused(&self, id: &WidgetId) -> bool {
        self.focused == Some(*id)
    }

    pub(crate) fn was_focused(&self, id: &WidgetId) -> bool {
        self.was_focused == Some(*id)
    }

    pub(crate) fn set_active(&mut self, id: &WidgetId) {
        self.active = Some(*id);
    }

    pub(crate) fn set_inactive(&mut self, id: &WidgetId) {
        if self.is_active(id) {
            self.active = None;
        }
    }
}

// pub fn handle_interaction(
//     user_input: &mut UserInput,
//     view: &View,
//     text: &mut TextsResources,
//     fonts: &mut FontResources,
// ) {
//     if state.input.mouse_left_pressed {
//         state.input.mouse_left_click_count = state.input.mouse_left_click_tracker.on_click(
//             state.input.mouse_x as f32,
//             state.input.mouse_y as f32,
//             view.scale_factor,
//         );
//     }
// }
