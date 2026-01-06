use std::hash::Hash;
use std::{any::TypeId, marker::PhantomData};

use clew_derive::WidgetBuilder;

use super::builder::WidgetCommon;
use super::{builder::BuildContext, scope::scope};
use crate::{WidgetId, state::WidgetState};

#[derive(WidgetBuilder)]
pub struct WidgetBuilder<T: WidgetState + Widget> {
    common: WidgetCommon,
    phantom_data: PhantomData<T>,
}

#[derive(WidgetBuilder)]
pub struct WidgetWithStateBuilder<'a, T: WidgetState + Widget> {
    common: WidgetCommon,
    state: &'a mut T,
}

pub trait Widget: 'static {
    type Event;

    fn on_event(&mut self, _event: &Self::Event) -> bool {
        false
    }

    fn build(&mut self, ctx: &mut BuildContext);
}

impl<T: WidgetState + Widget + Default> WidgetBuilder<T> {
    pub fn state<'a>(self, state: &'a mut T) -> WidgetWithStateBuilder<'a, T> {
        WidgetWithStateBuilder {
            common: self.common,
            state,
        }
    }

    pub fn build(&mut self, context: &mut BuildContext) {
        let id = self.common.id.with_seed(context.id_seed);
        let (idx, mut state) = context.widgets_states.take_or_create(id, T::default);

        // Skip event processing for () type
        if TypeId::of::<T::Event>() != TypeId::of::<()>() {
            for event_box in context.event_queue.iter() {
                if let Some(event) = event_box.downcast_ref::<T::Event>() {
                    state.on_event(event);
                }
            }
        }

        context.widgets_states.custom.accessed_this_frame.insert(id);
        context.build_with_common(&mut self.common, |ctx| state.build(ctx));

        context.widgets_states.restore(idx, state);
    }
}

impl<'a, T: WidgetState + Widget + Default> WidgetWithStateBuilder<'a, T> {
    pub fn build(&mut self, context: &mut BuildContext) {
        let id = self.common.id.with_seed(context.id_seed);

        // Skip event processing for () type
        if TypeId::of::<T::Event>() != TypeId::of::<()>() {
            for event_box in context.event_queue.iter() {
                if let Some(event) = event_box.downcast_ref::<T::Event>() {
                    self.state.on_event(event);
                }
            }
        }

        context.widgets_states.custom.accessed_this_frame.insert(id);
        context.build_with_common(&mut self.common, |ctx| self.state.build(ctx));
    }
}

#[track_caller]
pub fn widget<T: WidgetState + Widget>() -> WidgetBuilder<T> {
    WidgetBuilder {
        common: WidgetCommon::default(),
        phantom_data: PhantomData,
    }
}
