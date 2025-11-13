use std::{any::Any, sync::Arc};

use crate::{
    AlignX, AlignY, View, ViewId,
    layout::LayoutCommand,
    state::WidgetsStates,
    text::{FontResources, StringInterner, TextsResources},
};

#[derive(Debug)]
pub enum ApplicationEvent {
    Wake { view_id: ViewId },
}

pub trait ApplicationEventLoopProxy: Send + Sync {
    fn send_event(&self, event: ApplicationEvent);
}

pub struct BuildContext<'a, 'b> {
    pub current_zindex: i32,
    pub layout_commands: &'a mut Vec<LayoutCommand>,
    pub widgets_states: &'a mut WidgetsStates,
    pub event_queue: &'a mut Vec<Arc<dyn Any + Send>>,
    pub next_event_queue: &'a mut Vec<Arc<dyn Any + Send>>,
    pub broadcast_event_queue: &'a mut Vec<Arc<dyn Any + Send>>,
    pub text: &'a mut TextsResources<'b>,
    pub fonts: &'a mut FontResources,
    pub view: &'a View,
    pub string_interner: &'a mut StringInterner,
    pub async_tx: &'a mut tokio::sync::mpsc::UnboundedSender<Box<dyn Any + Send>>,
    pub broadcast_async_tx: &'a mut tokio::sync::mpsc::UnboundedSender<Box<dyn Any + Send>>,
    pub event_loop_proxy: Arc<dyn ApplicationEventLoopProxy>,
}

impl BuildContext<'_, '_> {
    pub fn push_layout_command(&mut self, command: LayoutCommand) {
        self.layout_commands.push(command);
    }

    pub fn with_align<F>(&mut self, align_x: Option<AlignX>, align_y: Option<AlignY>, callback: F)
    where
        F: FnOnce(&mut BuildContext),
    {
        if align_x.is_some() || align_y.is_some() {
            self.push_layout_command(LayoutCommand::BeginAlign {
                align_x: align_x.unwrap_or(AlignX::Left),
                align_y: align_y.unwrap_or(AlignY::Top),
            });
            callback(self);
            self.push_layout_command(LayoutCommand::EndAlign);
        } else {
            callback(self);
        }
    }

    pub fn emit<E: Any + Send + 'static>(&mut self, event: E) {
        self.next_event_queue.push(Arc::new(event));
    }

    pub fn spawn<E: Any + Send + 'static, F>(&self, future: F)
    where
        F: Future<Output = E> + Send + 'static,
    {
        let tx = self.async_tx.clone();
        let event_loop_proxy = self.event_loop_proxy.clone();
        let view_id = self.view.id;

        tokio::spawn(async move {
            let event = future.await;
            let _ = tx.send(Box::new(event));
            event_loop_proxy.send_event(ApplicationEvent::Wake { view_id });
        });
    }

    pub fn broadcast<E: Any + Send + 'static>(&mut self, event: E) {
        self.broadcast_event_queue.push(Arc::new(event));
    }

    pub fn spawn_broadcast<E: Any + Send + 'static, F>(&self, future: F)
    where
        F: Future<Output = E> + Send + 'static,
    {
        let tx = self.broadcast_async_tx.clone();
        let event_loop_proxy = self.event_loop_proxy.clone();
        let view_id = self.view.id;

        tokio::spawn(async move {
            let event = future.await;
            let _ = tx.send(Box::new(event));
            event_loop_proxy.send_event(ApplicationEvent::Wake { view_id });
        });
    }
}

#[macro_export]
macro_rules! impl_size_methods {
    () => {
        pub fn size(mut self, size: Size) -> Self {
            self.size = size;
            self
        }

        pub fn width<T: Into<SizeConstraint>>(mut self, size: T) -> Self {
            self.size.width = size.into();
            self
        }

        pub fn height<T: Into<SizeConstraint>>(mut self, size: T) -> Self {
            self.size.height = size.into();
            self
        }

        pub fn fill_max_width(mut self) -> Self {
            self.size.width = SizeConstraint::Fill(1.);
            self
        }

        pub fn fill_max_height(mut self) -> Self {
            self.size.height = SizeConstraint::Fill(1.);
            self
        }

        pub fn fill_max_size(mut self) -> Self {
            self.size.width = SizeConstraint::Fill(1.);
            self.size.height = SizeConstraint::Fill(1.);
            self
        }

        pub fn constraints(mut self, constraints: Constraints) -> Self {
            self.constraints = constraints;
            self
        }

        pub fn max_width(mut self, value: f32) -> Self {
            self.constraints.max_width = Some(value);
            self
        }

        pub fn max_height(mut self, value: f32) -> Self {
            self.constraints.max_height = Some(value);
            self
        }

        pub fn min_width(mut self, value: f32) -> Self {
            self.constraints.min_width = Some(value);
            self
        }

        pub fn min_height(mut self, value: f32) -> Self {
            self.constraints.min_height = Some(value);
            self
        }
    };
}

#[macro_export]
macro_rules! impl_id {
    () => {
        #[track_caller]
        pub fn id(mut self, id: impl Hash) -> Self {
            self.id = WidgetId::auto_with_seed(id);

            self
        }
    };
}

#[macro_export]
macro_rules! impl_width_methods {
    () => {
        pub fn width<T: Into<SizeConstraint>>(mut self, size: T) -> Self {
            self.width = size.into();
            self
        }

        pub fn fill_max_width(mut self) -> Self {
            self.width = SizeConstraint::Fill(1.);
            self
        }

        pub fn max_width(mut self, value: f32) -> Self {
            self.constraints.max_width = Some(value);
            self
        }

        pub fn min_width(mut self, value: f32) -> Self {
            self.constraints.min_width = Some(value);
            self
        }
    };
}

#[macro_export]
macro_rules! impl_position_methods {
    () => {
        pub fn align_x(mut self, align: AlignX) -> Self {
            self.align_x = Some(align);
            self
        }

        pub fn align_y(mut self, align: AlignY) -> Self {
            self.align_y = Some(align);
            self
        }

        pub fn zindex(mut self, zindex: i32) -> Self {
            self.zindex = Some(zindex);
            self
        }
    };
}
