use glam::Vec2;
use rustc_hash::FxHashSet;

use crate::{
    View, WidgetId,
    io::UserInput,
    layout::LayoutItem,
    point_with_rect_hit_test,
    text::{FontResources, TextsResources},
};

#[derive(Default, Clone, PartialEq)]
pub struct InteractionState {
    pub(crate) hover: FxHashSet<WidgetId>,
    pub(crate) hot: Option<WidgetId>,
    pub(crate) active: Option<WidgetId>,
    pub(crate) focused: Option<WidgetId>,
    pub(crate) was_focused: Option<WidgetId>,
    pub(crate) block_hover: bool,
}

impl InteractionState {
    pub fn is_hover(&self, id: &WidgetId) -> bool {
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

    pub(crate) fn _was_focused(&self, id: &WidgetId) -> bool {
        self.was_focused == Some(*id)
    }

    pub(crate) fn set_active(&mut self, id: &WidgetId) {
        self.active = Some(*id);
    }

    pub(crate) fn set_inactive(&mut self, id: &WidgetId) {
        if self.is_active(id) {
            self.active = None;
            self.block_hover = false;
        }
    }
}

pub fn handle_interaction(
    user_input: &mut UserInput,
    interaction_state: &mut InteractionState,
    non_interactable: &FxHashSet<WidgetId>,
    view: &View,
    _text: &mut TextsResources,
    _fonts: &mut FontResources,
    layout_items: &[LayoutItem],
) -> bool {
    if user_input.mouse_left_pressed {
        user_input.mouse_left_click_count = user_input.mouse_left_click_tracker.on_click(
            user_input.mouse_x,
            user_input.mouse_y,
            view.scale_factor,
        );
    }

    let unscaled_mouse_x = user_input.mouse_x / view.scale_factor;
    let unscaled_mouse_y = user_input.mouse_y / view.scale_factor;

    let mouse_point = Vec2::new(unscaled_mouse_x, unscaled_mouse_y);

    interaction_state.hot = None;
    interaction_state.hover.clear();

    for layout_item in layout_items.iter() {
        if let LayoutItem::Placement(placement) = layout_item
            && point_with_rect_hit_test(mouse_point, placement.rect)
        {
            interaction_state.hover.insert(placement.widget_ref.id);
        }
    }

    for layout_item in layout_items.iter().rev() {
        if let LayoutItem::Placement(placement) = layout_item
            && !non_interactable.contains(&placement.widget_ref.id)
            && (!interaction_state.block_hover
                || interaction_state.active.is_none()
                || interaction_state.active == Some(placement.widget_ref.id))
            && point_with_rect_hit_test(mouse_point, placement.rect)
        {
            interaction_state.hot = Some(placement.widget_ref.id);
            break;
        }
    }

    true
}
