use std::any::TypeId;

use super::builder::BuildContext;

pub struct ComponentBuilder<'a, V: Component<Event>, Event> {
    view: &'a mut V,
    phantom_data: std::marker::PhantomData<Event>,
}

pub trait Component<Event = ()> {
    fn on_event(&mut self, event: &Event) -> bool {
        false
    }

    fn build(&mut self, ctx: &mut BuildContext);
}

impl<'a, V: Component<Event>, Event: 'static> ComponentBuilder<'a, V, Event> {
    pub fn build(&mut self, context: &mut BuildContext) {
        // Skip event processing for () type
        if TypeId::of::<Event>() != TypeId::of::<()>() {
            for event_box in context.event_queue.iter() {
                if let Some(event) = event_box.downcast_ref::<Event>() {
                    self.view.on_event(event);
                }
            }
        }

        self.view.build(context);
    }
}

pub fn component<V: Component<Event>, Event>(view: &mut V) -> ComponentBuilder<V, Event> {
    ComponentBuilder {
        view,
        phantom_data: std::marker::PhantomData,
    }
}
