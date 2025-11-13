use std::any::TypeId;

use super::builder::BuildContext;

pub struct ComponentBuilder<'a, A, V: Component<A, Event>, Event> {
    app: &'a mut A,
    view: &'a mut V,
    phantom_data: std::marker::PhantomData<Event>,
}

pub trait Component<A, Event = ()> {
    fn on_event(&mut self, _app: &mut A, _event: &Event) -> bool {
        false
    }

    fn build(&mut self, app: &mut A, ctx: &mut BuildContext);
}

impl<'a, A, V: Component<A, Event>, Event: 'static> ComponentBuilder<'a, A, V, Event> {
    pub fn build(&mut self, context: &mut BuildContext) {
        // Skip event processing for () type
        if TypeId::of::<Event>() != TypeId::of::<()>() {
            for event_box in context.event_queue.iter() {
                if let Some(event) = event_box.downcast_ref::<Event>() {
                    self.view.on_event(self.app, event);
                }
            }
        }

        self.view.build(self.app, context);
    }
}

pub fn component<'a, A, V: Component<A, Event>, Event>(
    app: &'a mut A,
    component: &'a mut V,
) -> ComponentBuilder<'a, A, V, Event> {
    ComponentBuilder {
        app,
        view: component,
        phantom_data: std::marker::PhantomData,
    }
}
