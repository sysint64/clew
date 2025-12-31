use std::hash::Hash;
use std::{any::TypeId, marker::PhantomData};

use super::{builder::BuildContext, scope::scope};
use crate::{WidgetId, impl_id, state::WidgetState};

pub struct WidgetBuilder<T: WidgetState + Widget> {
    id: WidgetId,
    phantom_data: PhantomData<T>,
}

pub trait Widget: 'static {
    type Event;

    fn on_event(&mut self, _event: &Self::Event) -> bool {
        false
    }

    fn build(&mut self, ctx: &mut BuildContext);
}

impl<T: WidgetState + Widget + Default> WidgetBuilder<T> {
    impl_id!();

    pub fn build(&mut self, context: &mut BuildContext) {
        let id = self.id.with_seed(context.id_seed);
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

        scope(id).build(context, |context| {
            state.build(context);
        });

        context.widgets_states.restore(idx, state);
    }
}

#[track_caller]
pub fn widget<T: WidgetState + Widget>() -> WidgetBuilder<T> {
    WidgetBuilder {
        id: WidgetId::auto(),
        phantom_data: PhantomData,
    }
}
