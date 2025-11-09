use std::{
    any::Any,
    collections::{HashMap, HashSet},
};

use crate::{WidgetId, interaction::InteractionState};

pub trait WidgetState: Any + Send {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any + Send> WidgetState for T {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct WidgetsStates {
    pub data: HashMap<WidgetId, Box<dyn WidgetState>>,
    pub accessed_this_frame: HashSet<WidgetId>,
}

impl WidgetsStates {
    pub fn get_or_insert<T: WidgetState, F>(&mut self, id: WidgetId, create: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        self.accessed_this_frame.insert(id);

        if !self.data.contains_key(&id) {
            self.data.insert(id, Box::new(create()));
        }

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
