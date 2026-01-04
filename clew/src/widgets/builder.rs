use std::{
    any::Any,
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::Arc,
};

use rustc_hash::FxHasher;
use smallvec::SmallVec;

use crate::{
    Animation, View, ViewId, WidgetRef,
    interaction::InteractionState,
    io::UserInput,
    layout::LayoutCommand,
    state::WidgetsStates,
    text::{FontResources, StringId, StringInterner, TextId, TextsResources},
};
// use bumpalo::{Bump, collections::Vec};

#[derive(Debug)]
pub enum ApplicationEvent {
    Wake { view_id: ViewId },
}

pub trait ApplicationEventLoopProxy: Send + Sync {
    fn send_event(&self, event: ApplicationEvent);
}

pub struct UserDataStack<'a> {
    data: &'a (dyn Any + Send),
    parent: Option<&'a UserDataStack<'a>>,
}

pub struct MutUserDataStack<'a> {
    data: &'a mut (dyn Any + Send),
    parent: Option<&'a mut MutUserDataStack<'a>>,
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
    pub strings: &'a mut HashMap<StringId, TextId>,
    pub async_tx: &'a mut tokio::sync::mpsc::UnboundedSender<Box<dyn Any + Send>>,
    pub broadcast_async_tx: &'a mut tokio::sync::mpsc::UnboundedSender<Box<dyn Any + Send>>,
    pub event_loop_proxy: Arc<dyn ApplicationEventLoopProxy>,
    pub id_seed: Option<u64>,
    // pub user_data: Vec<Box<dyn Any + Send>>,
    pub user_data: Option<&'a UserDataStack<'a>>,
    pub scoped_user_data: Option<&'a mut MutUserDataStack<'a>>,
    pub decorators: &'a mut SmallVec<[WidgetRef; 8]>,
    pub phase_allocator: &'a bumpalo::Bump,
    pub input: &'a UserInput,
    pub interaction: &'a mut InteractionState,
    pub delta_time: f32,
    pub animations_stepped_this_frame: &'a mut HashSet<usize>,
}

impl BuildContext<'_, '_> {
    pub fn step_animation<V, T: Animation<V>>(&mut self, animation: &mut T) {
        if animation.in_progress() {
            let id = animation as *mut T as usize;

            if self.animations_stepped_this_frame.insert(id) {
                animation.step(self.delta_time)
            }
        }
    }

    pub fn provide<F, T: Any + Send>(&mut self, data: T, callback: F)
    where
        F: FnOnce(&mut Self),
    {
        // Store as raw pointer to avoid lifetime issues
        let data_ref: &(dyn Any + Send) = &data;
        let node = UserDataStack {
            data: unsafe { &*(data_ref as *const _) },
            parent: self.user_data.take(),
        };

        self.user_data = Some(unsafe { &*(&node as *const _) });

        callback(self);

        // Restore parent, dropping our node's reference
        self.user_data = node.parent;
    }

    pub fn scoped<F, T: Any + Send>(&mut self, data: &mut T, callback: F)
    where
        F: FnOnce(&mut Self),
    {
        // Store as raw pointer to avoid lifetime issues
        let data_ref: &mut (dyn Any + Send) = data;
        let mut node = MutUserDataStack {
            data: unsafe { &mut *(data_ref as *mut _) },
            parent: self.scoped_user_data.take(),
        };

        self.scoped_user_data = Some(unsafe { &mut *(&mut node as *mut _) });

        callback(self);

        // Restore parent, dropping our node's reference
        self.scoped_user_data = node.parent;
    }

    pub fn of<T: 'static>(&self) -> Option<&T> {
        let mut current = self.user_data;
        while let Some(node) = current {
            if let Some(data) = node.data.downcast_ref::<T>() {
                return Some(data);
            }
            current = node.parent;
        }
        None
    }

    // pub fn of_mut<T: 'static>(&mut self) -> Option<&mut T> {
    //     let mut current = self.scoped_user_data;
    //     while let Some(node) = current {
    //         if let Some(data) = node.data.downcast_mut::<T>() {
    //             return Some(data);
    //         }
    //         current = node.parent;
    //     }
    //     None
    // }

    pub fn of_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let mut current = self.scoped_user_data.as_mut();
        while let Some(node) = current {
            if (*node.data).is::<T>() {
                return Some(unsafe { &mut *(node.data as *mut dyn Any as *mut T) });
            }
            current = node.parent.as_mut();
        }
        None
    }

    // pub fn with_user_data<F, T: Any + Send + 'static>(&mut self, data: T, callback: F)
    // where
    //     F: FnOnce(&mut BuildContext),
    // {
    //     self.user_data.push(Box::new(data));
    //     callback(self);
    //     self.user_data.pop();
    // }

    // pub fn of<T: 'static>(&self) -> Option<&T> {
    //     for data in self.user_data.iter().rev() {
    //         let data = data.downcast_ref::<T>();

    //         if data.is_some() {
    //             return data;
    //         }
    //     }

    //     None
    // }

    #[profiling::function]
    pub fn push_layout_command(&mut self, command: LayoutCommand) {
        self.layout_commands.push(command);
    }

    pub fn with_id_seed<F, T>(&mut self, seed: u64, callback: F) -> T
    where
        F: FnOnce(&mut BuildContext) -> T,
    {
        let last_id_seed = self.id_seed;

        // Combine with parent seed, to support nested scopes
        self.id_seed = Some(match last_id_seed {
            Some(parent) => {
                let mut hasher = FxHasher::default();
                parent.hash(&mut hasher);
                seed.hash(&mut hasher);
                hasher.finish()
            }
            None => seed,
        });

        let result = callback(self);
        self.id_seed = last_id_seed;

        result
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
        pub fn size<T: Into<Size>>(mut self, size: T) -> Self {
            self.size = size.into();
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
            self.constraints.max_width = value;
            self
        }

        pub fn max_height(mut self, value: f32) -> Self {
            self.constraints.max_height = value;
            self
        }

        pub fn min_width(mut self, value: f32) -> Self {
            self.constraints.min_width = value;
            self
        }

        pub fn min_height(mut self, value: f32) -> Self {
            self.constraints.min_height = value;
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
            self.constraints.max_width = value;
            self
        }

        pub fn min_width(mut self, value: f32) -> Self {
            self.constraints.min_width = value;
            self
        }
    };
}

#[macro_export]
macro_rules! impl_position_methods {
    () => {
        pub fn zindex(mut self, zindex: i32) -> Self {
            self.zindex = Some(zindex);
            self
        }
    };
}
