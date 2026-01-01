use std::{
    any::Any,
    collections::{HashMap, HashSet},
    sync::Arc,
};

use bitvec::vec::BitVec;
use rustc_hash::{FxHashMap, FxHashSet};
use slab::Slab;
use smallvec::SmallVec;

use crate::{
    LayoutDirection, View, WidgetId, WidgetRef,
    interaction::InteractionState,
    io::UserInput,
    layout::{LayoutCommand, LayoutItem, LayoutMeasure, LayoutState, WidgetPlacement},
    render::RenderState,
    widgets::{colored_box, decorated_box, gesture_detector, scroll_area, svg, text},
};

pub trait WidgetState: Any + Send + 'static {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

pub struct UiState {
    pub view: View,
    pub render_state: RenderState,
    pub layout_commands: Vec<LayoutCommand>,
    pub phase_allocator: bumpalo::Bump,
    pub(crate) layout_state: LayoutState,
    pub current_event_queue: Vec<Arc<dyn Any + Send>>,
    pub next_event_queue: Vec<Arc<dyn Any + Send>>,
    pub widgets_states: WidgetsStates,
    pub(crate) widget_placements: Vec<WidgetPlacement>,
    pub(crate) layout_items: Vec<LayoutItem>,
    pub interaction_state: InteractionState,
    pub last_interaction_state: InteractionState,
    pub user_input: UserInput,
    pub decorators: SmallVec<[WidgetRef; 8]>,
    // TODO(sysint64): Maybe move it to build context
    pub layout_direction: LayoutDirection,
    pub async_tx: tokio::sync::mpsc::UnboundedSender<Box<dyn Any + Send>>,
    pub async_rx: tokio::sync::mpsc::UnboundedReceiver<Box<dyn Any + Send>>,
}

#[derive(Default)]
pub struct WidgetsStates {
    // pub data: FxHashMap<WidgetId, Box<dyn WidgetState>>,
    // pub last: FxHashMap<WidgetId, Box<dyn WidgetState>>,
    pub layout_measures: TypedWidgetStates<LayoutMeasure>,

    pub decorated_box: TypedWidgetStates<decorated_box::State>,
    pub scroll_area: TypedWidgetStates<scroll_area::State>,
    pub text: TypedWidgetStates<text::State>,
    pub gesture_detector: TypedWidgetStates<gesture_detector::State>,
    pub colored_box: TypedWidgetStates<colored_box::State>,
    pub svg: TypedWidgetStates<svg::State>,
    pub components: TypedWidgetStates<Box<dyn Any>>,
    pub custom: TypedWidgetStates<Option<Box<dyn WidgetState>>>,
}

pub struct TypedWidgetStates<T> {
    id_to_index: FxHashMap<WidgetId, u32>,
    states: Vec<T>,
    ids: Vec<WidgetId>,
    pub accessed_this_frame: FxHashSet<WidgetId>,
}

impl<T> Default for TypedWidgetStates<T> {
    fn default() -> Self {
        Self {
            id_to_index: FxHashMap::default(),
            states: Vec::new(),
            ids: Vec::new(),
            accessed_this_frame: FxHashSet::default(),
        }
    }
}

impl<T> TypedWidgetStates<T> {
    pub fn get_or_insert(&mut self, id: WidgetId, create: impl FnOnce() -> T) -> &mut T {
        let index = *self.id_to_index.entry(id).or_insert_with(|| {
            let idx = self.states.len() as u32;
            self.states.push(create());
            self.ids.push(id);
            idx
        });
        &mut self.states[index as usize]
    }

    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut T> {
        self.id_to_index
            .get(&id)
            .map(|&idx| &mut self.states[idx as usize])
    }

    pub fn get(&self, id: WidgetId) -> Option<&T> {
        self.id_to_index
            .get(&id)
            .map(|&idx| &self.states[idx as usize])
    }

    pub fn replace(&mut self, id: WidgetId, state: T) {
        if let Some(&idx) = self.id_to_index.get(&id) {
            self.states[idx as usize] = state;
        } else {
            let idx = self.states.len() as u32;
            self.id_to_index.insert(id, idx);
            self.states.push(state);
            self.ids.push(id);
        }
    }

    pub fn set(&mut self, id: WidgetId, state: T) -> usize {
        if let Some(&idx) = self.id_to_index.get(&id) {
            self.states[idx as usize] = state;

            idx as usize
        } else {
            let idx = self.states.len() as u32;
            self.id_to_index.insert(id, idx);
            self.states.push(state);
            self.ids.push(id);

            idx as usize
        }
    }

    pub fn sweep(&mut self, interaction: &mut InteractionState) {
        let mut i = 0;

        while i < self.states.len() {
            if self.accessed_this_frame.contains(&self.ids[i]) {
                i += 1;
            } else {
                // Swap-remove from both parallel arrays
                self.id_to_index.remove(&self.ids[i]);

                self.states.swap_remove(i);
                self.ids.swap_remove(i);

                // Update the index of the element that was swapped in
                if i < self.ids.len() {
                    self.id_to_index.insert(self.ids[i], i as u32);
                }
            }
        }

        self.accessed_this_frame.clear();
    }

    pub fn clear(&mut self) {
        self.id_to_index.clear();
        self.states.clear();
        self.ids.clear();
        self.accessed_this_frame.clear();
    }
}

impl UiState {
    pub fn before_render(&mut self) {
        self.layout_commands.clear();
        self.render_state.commands.clear();
        self.widget_placements.clear();
        self.layout_items.clear();

        std::mem::swap(&mut self.current_event_queue, &mut self.next_event_queue);
        self.next_event_queue.clear();

        // Collect async events
        while let Ok(event) = self.async_rx.try_recv() {
            self.current_event_queue.push(event.into());
        }
    }

    pub fn new(view: View) -> Self {
        let (async_tx, async_rx) = tokio::sync::mpsc::unbounded_channel();

        let phase_allocator = bumpalo::Bump::with_capacity(16 * 1024 * 1024);

        Self {
            view,
            render_state: Default::default(),
            phase_allocator,
            layout_commands: Vec::new(),
            current_event_queue: Vec::new(),
            next_event_queue: Vec::new(),
            widgets_states: WidgetsStates::default(),
            layout_state: LayoutState::default(),
            widget_placements: Vec::new(),
            layout_items: Vec::new(),
            decorators: SmallVec::new(),
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
    #[profiling::function]
    pub fn get_or_insert_custom<T: WidgetState, F>(&mut self, id: WidgetId, create: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        let index = *self.custom.id_to_index.entry(id).or_insert_with(|| {
            let idx = self.custom.states.len() as u32;
            self.custom.states.push(Some(Box::new(create())));
            self.custom.ids.push(id);
            idx
        });

        self.custom.states[index as usize]
            .as_mut()
            .unwrap()
            .as_any_mut()
            .downcast_mut::<T>()
            .unwrap()

        // self.data
        //     .entry(id)
        //     .or_insert_with(|| Box::new(create()))
        //     .as_any_mut()
        //     .downcast_mut::<T>()
        //     .unwrap()
    }

    // pub fn take_or_create<T: WidgetState, F>(&mut self, id: WidgetId, create: F) -> (u32, T)
    // where
    //     F: Fn() -> T,
    // {
    //     let index = *self.custom.id_to_index.entry(id).or_insert_with(|| {
    //         let idx = self.custom.states.len() as u32;
    //         self.custom.states.push(Box::new(create()));
    //         self.custom.ids.push(id);
    //         idx
    //     });

    //     let boxed = std::mem::replace(&mut self.custom.states[index as usize], Box::new(create()));
    //     let concrete = *boxed
    //         .into_any()
    //         .downcast::<T>()
    //         .expect("Type mismatch in widget state");

    //     (index, concrete)
    // }

    // pub fn restore<T: WidgetState>(&mut self, index: u32, state: T) {
    //     self.custom.states[index as usize] = Box::new(state);
    // }

    pub fn take_or_create<T: WidgetState, F>(&mut self, id: WidgetId, create: F) -> (u32, Box<T>)
    where
        F: FnOnce() -> T,
    {
        let index = *self.custom.id_to_index.entry(id).or_insert_with(|| {
            let idx = self.custom.states.len() as u32;
            self.custom.states.push(Some(Box::new(create())));
            self.custom.ids.push(id);
            idx
        });

        let boxed = self.custom.states[index as usize]
            .take()
            .expect("State already taken");

        let concrete: Box<T> = boxed
            .into_any()
            .downcast::<T>()
            .expect("Type mismatch in widget state");

        (index, concrete)
    }

    pub fn restore<T: WidgetState>(&mut self, index: u32, state: Box<T>) {
        self.custom.states[index as usize] = Some(state as Box<dyn WidgetState>);
    }

    // #[profiling::function]
    // pub fn replace<T: WidgetState>(&mut self, id: WidgetId, state: T) {
    //     match self.data.entry(id) {
    //         std::collections::hash_map::Entry::Occupied(mut entry) => {
    //             // Try to reuse existing allocation
    //             if let Some(existing) = entry.get_mut().as_any_mut().downcast_mut::<T>() {
    //                 *existing = state;
    //             } else {
    //                 entry.insert(Box::new(state));
    //             }
    //         }
    //         std::collections::hash_map::Entry::Vacant(entry) => {
    //             entry.insert(Box::new(state));
    //         }
    //     }

    //     // self.data.insert(id, Box::new(state));

    //     // self.data.entry(id).or_insert(|| Box::new(create()));
    //     // self.accessed_this_frame.insert(id);
    //     // self.data.entry(id).or_insert_with(|| Box::new(create()));

    //     // self.data
    //     //     .get_mut(&id)
    //     //     .unwrap()
    //     //     .as_any_mut()
    //     //     .downcast_mut::<T>()
    //     //     .unwrap()
    // }

    // #[profiling::function]
    // pub fn get_mut<T: WidgetState>(&mut self, id: WidgetId) -> Option<&mut T> {
    //     self.data
    //         .get_mut(&id)
    //         .and_then(|b| b.as_any_mut().downcast_mut::<T>())
    // }

    #[profiling::function]
    pub fn update_last<T>(&mut self, id: WidgetId) -> bool
    where
        T: WidgetState + Clone + PartialEq,
    {
        true

        // let current_state = self
        //     .data
        //     .get(&id)
        //     .and_then(|b| b.as_any().downcast_ref::<T>())
        //     .unwrap();

        // let last_state = self
        //     .last
        //     .get_mut(&id)
        //     .and_then(|b| b.as_any_mut().downcast_mut::<T>());

        // if let Some(last_state) = last_state {
        //     if last_state != current_state {
        //         *last_state = current_state.clone();

        //         true
        //     } else {
        //         false
        //     }
        // } else {
        //     self.last.insert(id, Box::new(current_state.clone()));

        //     true
        // }
    }

    // pub fn contains(&self, id: WidgetId) -> bool {
    //     self.data.contains_key(&id)
    // }

    #[profiling::function]
    pub fn sweep(&mut self, interaction: &mut InteractionState) {
        self.decorated_box.clear();
        self.colored_box.clear();
        self.svg.clear();
        self.gesture_detector.sweep(interaction);
        self.custom.sweep(interaction);
        self.text.sweep(interaction);
        self.scroll_area.sweep(interaction);
        self.layout_measures.clear();

        // self.data
        //     .retain(|id, _| self.accessed_this_frame.contains(id));

        // if let Some(id) = interaction.focused {
        //     if !self.accessed_this_frame.contains(&id) {
        //         interaction.focused = None;
        //     }
        // }

        // self.accessed_this_frame.clear();
    }
}
