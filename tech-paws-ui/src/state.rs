use std::{
    any::Any,
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    LayoutDirection, View, WidgetId,
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
    pub(crate) layout_state: LayoutState,
    pub current_event_queue: Vec<Arc<dyn Any + Send>>,
    pub next_event_queue: Vec<Arc<dyn Any + Send>>,
    pub widgets_states: WidgetsStates,
    pub(crate) widget_placements: Vec<WidgetPlacement>,
    pub interaction_state: InteractionState,
    pub last_interaction_state: InteractionState,
    pub user_input: UserInput,
    // TODO(sysint64): Maybe move it to build context
    pub layout_direction: LayoutDirection,
    pub async_tx: tokio::sync::mpsc::UnboundedSender<Box<dyn Any + Send>>,
    pub async_rx: tokio::sync::mpsc::UnboundedReceiver<Box<dyn Any + Send>>,
}

#[derive(Default)]
pub struct WidgetsStates {
    pub data: HashMap<WidgetId, Box<dyn WidgetState>>,
    pub last: HashMap<WidgetId, Box<dyn WidgetState>>,
    pub accessed_this_frame: HashSet<WidgetId>,
}

impl UiState {
    pub fn before_render(&mut self) {
        self.layout_commands.clear();
        self.render_state.commands.clear();
        self.widget_placements.clear();

        std::mem::swap(&mut self.current_event_queue, &mut self.next_event_queue);
        self.next_event_queue.clear();

        // Collect async events
        while let Ok(event) = self.async_rx.try_recv() {
            self.current_event_queue.push(event.into());
        }
    }

    pub fn new(view: View) -> Self {
        let (async_tx, async_rx) = tokio::sync::mpsc::unbounded_channel();

        Self {
            view,
            render_state: Default::default(),
            layout_commands: Vec::new(),
            current_event_queue: Vec::new(),
            next_event_queue: Vec::new(),
            widgets_states: WidgetsStates::default(),
            layout_state: LayoutState::default(),
            widget_placements: Vec::new(),
            interaction_state: InteractionState::default(),
            last_interaction_state: InteractionState::default(),
            user_input: UserInput::default(),
            layout_direction: LayoutDirection::LTR,
            async_tx,
            async_rx,
        }
    }
}

impl WidgetsStates {
    pub fn get_or_insert<T: WidgetState, F>(&mut self, id: WidgetId, create: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        puffin::profile_function!();

        self.data.entry(id).or_insert_with(|| Box::new(create()));

        self.data
            .get_mut(&id)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<T>()
            .unwrap()
    }

    pub fn replace<T: WidgetState>(&mut self, id: WidgetId, state: T) {
        self.data.insert(id, Box::new(state));

        // self.data.entry(id).or_insert(|| Box::new(create()));
        // self.accessed_this_frame.insert(id);
        // self.data.entry(id).or_insert_with(|| Box::new(create()));

        // self.data
        //     .get_mut(&id)
        //     .unwrap()
        //     .as_any_mut()
        //     .downcast_mut::<T>()
        //     .unwrap()
    }

    pub fn get_mut<T: WidgetState>(&mut self, id: WidgetId) -> Option<&mut T> {
        self.data
            .get_mut(&id)
            .and_then(|b| b.as_any_mut().downcast_mut::<T>())
    }

    pub fn update_last<T>(&mut self, id: WidgetId) -> bool
    where
        T: WidgetState + Clone + PartialEq,
    {
        let current_state = self
            .data
            .get(&id)
            .and_then(|b| b.as_any().downcast_ref::<T>())
            .unwrap();

        let last_state = self
            .last
            .get_mut(&id)
            .and_then(|b| b.as_any_mut().downcast_mut::<T>());

        if let Some(last_state) = last_state {
            if last_state != current_state {
                *last_state = current_state.clone();

                true
            } else {
                false
            }
        } else {
            self.last.insert(id, Box::new(current_state.clone()));

            true
        }
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
