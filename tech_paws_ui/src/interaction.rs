use std::collections::HashSet;

use glam::Vec2;

use crate::{
    LayoutWidget, View, WidgetId, WidgetType,
    io::UserInput,
    layout::WidgetPlacement,
    point_with_rect_hit_test,
    state::{UiState, WidgetsStates},
    text::{FontResources, TextsResources},
    widgets,
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

pub fn handle_interaction(
    user_input: &mut UserInput,
    interaction_state: &mut InteractionState,
    widgets_states: &mut WidgetsStates,
    view: &View,
    text: &mut TextsResources,
    fonts: &mut FontResources,
    widget_placements: &[WidgetPlacement],
) {
    if user_input.mouse_left_pressed {
        user_input.mouse_left_click_count = user_input.mouse_left_click_tracker.on_click(
            user_input.mouse_x as f32,
            user_input.mouse_y as f32,
            view.scale_factor,
        );
    }

    let unscaled_mouse_x = user_input.mouse_x / view.scale_factor as f64;
    let unscaled_mouse_y = user_input.mouse_y / view.scale_factor as f64;

    let mouse_point = Vec2::new(unscaled_mouse_x as f32, unscaled_mouse_y as f32);

    interaction_state.hot = None;
    interaction_state.hover.clear();

    for placement in widget_placements.iter() {
        if placement.widget_ref.widget_type == WidgetType::of::<LayoutWidget>() {
            continue;
        }

        if point_with_rect_hit_test(mouse_point, placement.rect) {
            interaction_state.hover.insert(placement.widget_ref.id);
        }
    }

    for placement in widget_placements.iter().rev() {
        if placement.widget_ref.widget_type == WidgetType::of::<LayoutWidget>() {
            continue;
        }

        if point_with_rect_hit_test(mouse_point, placement.rect) {
            interaction_state.hot = Some(placement.widget_ref.id);
            break;
        }
    }

    for placement in widget_placements.iter() {
        if placement.widget_ref.widget_type == WidgetType::of::<widgets::button::ButtonWidget>() {
            widgets::button::handle_interaction(
                placement.widget_ref.id,
                &user_input,
                interaction_state,
                widgets_states
                    .get_mut::<widgets::button::State>(placement.widget_ref.id)
                    .unwrap(),
            );
        }
    }
}
