use crate::{
    View, WidgetId,
    interaction::InteractionState,
    io::{Cursor, UserInput},
    text::{FontResources, TextsResources},
};

use super::State;

pub(crate) fn handle_interaction(
    id: WidgetId,
    user_input: &mut UserInput,
    view: &View,
    interaction: &mut InteractionState,
    widget_state: &mut State,
    text: &mut TextsResources,
    fonts: &mut FontResources,
) {
    if interaction.is_hot(&id) || interaction.is_active(&id) {
        user_input.cursor = Cursor::Text;
    }

    if interaction.is_active(&id) {
        if user_input.mouse_released {
            if interaction.is_hot(&id) {
                interaction.set_inactive(&id);
                interaction.focused = Some(id);
                // events.push(UiEvent::FocusWindow);
            } else {
                interaction.set_inactive(&id);
            }
        }
    } else if user_input.mouse_left_pressed && interaction.is_hot(&id) {
        interaction.set_active(&id);
        interaction.focused = Some(id);
        // events.push(UiEvent::FocusWindow);
    }


}
