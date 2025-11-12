use std::{
    any::Any,
    collections::{HashMap, HashSet},
};

use crate::{
    LayoutDirection, View, WidgetId,
    event_queue::EventQueue,
    interaction::InteractionState,
    io::UserInput,
    layout::{LayoutCommand, LayoutState, WidgetPlacement},
    render::RenderState,
};

pub trait WidgetState: Any + Send {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct UiState {
    pub view: View,
    pub render_state: RenderState,
    pub layout_commands: Vec<LayoutCommand>,
    pub layout_state: LayoutState,
    pub event_queue: EventQueue,
    pub widgets_states: WidgetsStates,
    pub widget_placements: Vec<WidgetPlacement>,
    pub interaction_state: InteractionState,
    pub user_input: UserInput,
    // TODO(sysint64): Maybe move it to build context
    pub layout_direction: LayoutDirection,
}

#[derive(Default)]
pub struct WidgetsStates {
    pub data: HashMap<WidgetId, Box<dyn WidgetState>>,
    pub accessed_this_frame: HashSet<WidgetId>,
}

impl UiState {
    pub fn before_render(&mut self) {
        self.layout_commands.clear();
        self.render_state.commands.clear();
        self.widget_placements.clear();
    }

    pub fn new(view: View) -> Self {
        Self {
            view,
            render_state: Default::default(),
            layout_commands: Vec::new(),
            event_queue: EventQueue::new(),
            widgets_states: WidgetsStates::default(),
            layout_state: LayoutState::default(),
            widget_placements: Vec::new(),
            interaction_state: InteractionState::default(),
            user_input: UserInput::default(),
            layout_direction: LayoutDirection::LTR,
        }
    }
}

impl WidgetsStates {
    pub fn get_or_insert<T: WidgetState, F>(&mut self, id: WidgetId, create: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        self.accessed_this_frame.insert(id);
        self.data.entry(id).or_insert_with(|| Box::new(create()));

        self.data
            .get_mut(&id)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<T>()
            .unwrap()
    }

    pub fn get_mut<T: WidgetState>(&mut self, id: WidgetId) -> Option<&mut T> {
        self.data
            .get_mut(&id)
            .and_then(|b| b.as_any_mut().downcast_mut::<T>())
    }

    pub fn contains(&self, id: WidgetId) -> bool {
        self.data.contains_key(&id)
    }

    pub fn sweep(&mut self, interaction: &mut InteractionState) {
        self.data
            .retain(|id, _| self.accessed_this_frame.contains(id));

        if let Some(id) = interaction.focused {
            if !self.accessed_this_frame.contains(&id) {
                interaction.focused = None;
            }
        }

        self.accessed_this_frame.clear();
    }
}
