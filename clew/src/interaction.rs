use std::collections::HashSet;

use glam::Vec2;

use crate::{
    View, WidgetId, WidgetType,
    io::UserInput,
    layout::LayoutItem,
    point_with_rect_hit_test,
    scroll_area::ScrollAreaWidget,
    text::{FontResources, TextsResources},
    widgets::{self, gesture_detector::GestureDetector},
};

#[derive(Default, Clone, PartialEq)]
pub struct InteractionState {
    pub(crate) hover: HashSet<WidgetId>,
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
    // widgets_states: &mut WidgetsStates,
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
            // TODO: Remove this and put gesture detector to foregrounds instead of backgrounds
            && placement.widget_ref.widget_type == WidgetType::of::<GestureDetector>()
            && (!interaction_state.block_hover
                || interaction_state.active.is_none()
                || interaction_state.active == Some(placement.widget_ref.id))
            && point_with_rect_hit_test(mouse_point, placement.rect)
        {
            interaction_state.hot = Some(placement.widget_ref.id);
            break;
        }
    }

    // let mut need_to_redraw = false;

    for layout_item in layout_items.iter() {
        if let LayoutItem::Placement(placement) = layout_item {
            // if placement.widget_ref.widget_type == WidgetType::of::<widgets::button::ButtonWidget>() {
            //     widgets::button::handle_interaction(
            //         placement.widget_ref.id,
            //         user_input,
            //         interaction_state,
            //         widgets_states
            //             .get_mut::<widgets::button::State>(placement.widget_ref.id)
            //             .unwrap(),
            //     );

            //     need_to_redraw = need_to_redraw
            //         || widgets_states.update_last::<widgets::button::State>(placement.widget_ref.id);
            // }

            // if placement.widget_ref.widget_type
            //     == WidgetType::of::<widgets::gesture_detector::GestureDetector>()
            // {
            //     widgets::gesture_detector::handle_interaction(
            //         placement.widget_ref.id,
            //         user_input,
            //         view,
            //         interaction_state,
            //         // widgets_states
            //         //     .get_mut::<widgets::gesture_detector::State>(placement.widget_ref.id)
            //         //     .unwrap(),
            //         widgets_states
            //             .gesture_detector
            //             .get_mut(placement.widget_ref.id)
            //             .unwrap(),
            //     );

            //     need_to_redraw = need_to_redraw
            //         || widgets_states
            //             .update_last::<widgets::gesture_detector::State>(placement.widget_ref.id);
            // }

            if placement.widget_ref.widget_type
                == WidgetType::of::<widgets::scroll_area::ScrollAreaWidget>()
            {
                // widgets::scroll_area::handle_interaction(
                //     placement.widget_ref.id,
                //     user_input,
                //     interaction_state,
                //     widgets_states
                //         .scroll_area
                //         .get_mut(placement.widget_ref.id)
                //         .unwrap(),
                //     widgets_states
                //         .layout_measures
                //         .get_mut(placement.widget_ref.id)
                //         .unwrap(),
                // );

                // need_to_redraw = need_to_redraw
                //     || widgets_states
                //         .update_last::<widgets::gesture_detector::State>(placement.widget_ref.id);
            }

            // if placement.widget_ref.widget_type
            //     == WidgetType::of::<widgets::decorated_box::DecoratedBox>()
            // {
            //     need_to_redraw = need_to_redraw
            //         || widgets_states
            //             .update_last::<widgets::decorated_box::State>(placement.widget_ref.id);
            // }

            // if placement.widget_ref.widget_type
            //     == WidgetType::of::<widgets::colored_box::ColoredBox>()
            // {
            //     need_to_redraw = need_to_redraw
            //         || widgets_states
            //             .update_last::<widgets::colored_box::State>(placement.widget_ref.id);
            // }
        }
    }

    // need_to_redraw
    true
}
