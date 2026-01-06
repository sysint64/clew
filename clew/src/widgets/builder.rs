use std::{
    any::Any,
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::Arc,
};

use rustc_hash::{FxHashSet, FxHasher};
use smallvec::SmallVec;

use crate::{
    Animation, Clip, Constraints, EdgeInsets, Size, Value, View, ViewId, WidgetId, WidgetRef,
    interaction::InteractionState,
    io::UserInput,
    layout::{ContainerKind, LayoutCommand},
    state::WidgetsStates,
    text::{FontResources, StringId, StringInterner, TextId, TextsResources},
};

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
    pub backgrounds: &'a mut SmallVec<[WidgetRef; 8]>,
    pub foregrounds: &'a mut SmallVec<[WidgetRef; 8]>,
    pub non_interactable: &'a mut FxHashSet<WidgetId>,
    pub phase_allocator: &'a bumpalo::Bump,
    pub input: &'a UserInput,
    pub interaction: &'a mut InteractionState,
    pub delta_time: f32,
    pub animations_stepped_this_frame: &'a mut HashSet<usize>,
}

pub trait Resolve<V> {
    fn resolve(&mut self, ctx: &mut BuildContext) -> V;
}

impl<V, A> Resolve<V> for A
where
    A: Animation + Value<V>,
{
    /// Advances the animation by the current frame's delta time (if not already
    /// advanced this frame) and returns the resolved value for this frame.
    ///
    /// Calling this multiple times in the same frame will not advance time
    /// multiple times.
    fn resolve(&mut self, ctx: &mut BuildContext) -> V {
        ctx.step_animation(self);

        self.value()
    }
}

impl BuildContext<'_, '_> {
    /// Advances an animation by the current frame's delta time.
    ///
    /// This method updates the animation's internal state and status
    /// based on the elapsed time since the previous frame.
    ///
    /// Each animation is guaranteed to be stepped at most once per frame.
    /// Calling this method multiple times with the same animation in the
    /// same frame has no additional effect.
    ///
    /// This is typically called explicitly for long-lived animations, or
    /// indirectly via `resolve(ctx)` when retrieving an animated value.
    pub fn step_animation<T: Animation>(&mut self, animation: &mut T) {
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

    pub fn scope<F, T>(&mut self, key: impl Hash, callback: F) -> T
    where
        F: FnOnce(&mut BuildContext) -> T,
    {
        let mut hasher = FxHasher::default();
        key.hash(&mut hasher);

        self.with_id_seed(hasher.finish(), callback)
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

    pub fn build_with_common<F, T>(&mut self, common: &mut WidgetCommon, callback: F) -> T
    where
        F: FnOnce(&mut BuildContext) -> T,
    {
        let has_offset = common.flags.contains(WidgetCommonFlags::OFFSET);

        if has_offset {
            self.push_layout_command(LayoutCommand::BeginOffset {
                offset_x: common.offset_x,
                offset_y: common.offset_y,
            });
        }

        let needs_container = common.flags.intersects(
            WidgetCommonFlags::SIZE
                .union(WidgetCommonFlags::CONSTRAINTS)
                .union(WidgetCommonFlags::ZINDEX)
                .union(WidgetCommonFlags::PADDING)
                .union(WidgetCommonFlags::MARGIN)
                .union(WidgetCommonFlags::BACKGROUNDS)
                .union(WidgetCommonFlags::FOREGROUNDS)
                .union(WidgetCommonFlags::CLIP),
        );

        let value;

        if needs_container {
            let mut backgrounds = std::mem::take(self.backgrounds);
            backgrounds.append(&mut common.backgrounds);

            let mut foregrounds = std::mem::take(self.foregrounds);
            foregrounds.append(&mut common.foregrounds);

            let last_zindex = self.current_zindex;
            self.current_zindex += 1;

            self.push_layout_command(LayoutCommand::BeginContainer {
                backgrounds,
                foregrounds,
                zindex: last_zindex,
                padding: common.padding,
                margin: common.margin,
                kind: ContainerKind::Passthrough,
                size: common.size,
                constraints: common.constraints,
                clip: common.clip,
            });

            value = self.scope(common.id, callback);

            self.push_layout_command(LayoutCommand::EndContainer);
            self.current_zindex = last_zindex;
        } else {
            value = self.scope(common.id, callback);
        }

        if has_offset {
            self.push_layout_command(LayoutCommand::EndOffset);
        }

        value
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

bitflags::bitflags! {
    #[derive(Default, Clone, Copy)]
    pub struct WidgetCommonFlags: u16 {
        const ID = 1 << 0;
        const SIZE = 1 << 1;
        const CONSTRAINTS = 1 << 2;
        const ZINDEX = 1 << 3;
        const PADDING = 1 << 4;
        const MARGIN = 1 << 5;
        const BACKGROUNDS = 1 << 6;
        const FOREGROUNDS = 1 << 7;
        const OFFSET = 1 << 8;
        const CLIP = 1 << 9;
    }
}

pub struct WidgetCommon {
    pub id: WidgetId,
    pub size: Size,
    pub constraints: Constraints,
    pub zindex: Option<i32>,
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,
    pub backgrounds: SmallVec<[WidgetRef; 8]>,
    pub foregrounds: SmallVec<[WidgetRef; 8]>,
    pub offset_x: f32,
    pub offset_y: f32,
    pub clip: Clip,
    pub flags: WidgetCommonFlags,
}

impl Default for WidgetCommon {
    #[track_caller]
    fn default() -> Self {
        Self {
            id: WidgetId::auto(),
            size: Default::default(),
            constraints: Default::default(),
            zindex: Default::default(),
            padding: Default::default(),
            margin: Default::default(),
            backgrounds: Default::default(),
            foregrounds: Default::default(),
            offset_x: Default::default(),
            offset_y: Default::default(),
            clip: Clip::None,
            flags: WidgetCommonFlags::empty(),
        }
    }
}

pub struct Layout {
    pub size: Size,
    pub constraints: Constraints,
}

impl WidgetCommon {
    pub fn take_layout(&mut self) -> Layout {
        self.flags.remove(WidgetCommonFlags::SIZE);
        self.flags.remove(WidgetCommonFlags::CONSTRAINTS);

        Layout {
            size: self.size,
            constraints: self.constraints,
        }
    }
}

impl WidgetBuilder for WidgetCommon {
    fn common_mut(&mut self) -> &mut WidgetCommon {
        self
    }
}

pub trait WidgetBuilder {
    fn common_mut(&mut self) -> &mut WidgetCommon;

    #[track_caller]
    fn id(mut self, id: impl std::hash::Hash) -> Self
    where
        Self: Sized,
    {
        self.common_mut().id = ::clew::WidgetId::auto_with_seed(id);
        self.common_mut().flags |= WidgetCommonFlags::ID;
        self
    }

    fn size<T: Into<::clew::Size>>(mut self, size: T) -> Self
    where
        Self: Sized,
    {
        self.common_mut().size = size.into();
        self.common_mut().flags |= WidgetCommonFlags::SIZE;
        self
    }

    fn width<T: Into<::clew::SizeConstraint>>(mut self, size: T) -> Self
    where
        Self: Sized,
    {
        self.common_mut().size.width = size.into();
        self.common_mut().flags |= WidgetCommonFlags::SIZE;
        self
    }

    fn height<T: Into<::clew::SizeConstraint>>(mut self, size: T) -> Self
    where
        Self: Sized,
    {
        self.common_mut().size.height = size.into();
        self.common_mut().flags |= WidgetCommonFlags::SIZE;
        self
    }

    fn fill_max_width(mut self) -> Self
    where
        Self: Sized,
    {
        self.common_mut().size.width = ::clew::SizeConstraint::Fill(1.);
        self.common_mut().flags |= WidgetCommonFlags::SIZE;
        self
    }

    fn fill_max_height(mut self) -> Self
    where
        Self: Sized,
    {
        self.common_mut().size.height = ::clew::SizeConstraint::Fill(1.);
        self.common_mut().flags |= WidgetCommonFlags::SIZE;
        self
    }

    fn fill_max_size(mut self) -> Self
    where
        Self: Sized,
    {
        self.common_mut().size.width = ::clew::SizeConstraint::Fill(1.);
        self.common_mut().size.height = ::clew::SizeConstraint::Fill(1.);
        self.common_mut().flags |= WidgetCommonFlags::SIZE;
        self
    }

    fn constraints(mut self, constraints: ::clew::Constraints) -> Self
    where
        Self: Sized,
    {
        self.common_mut().constraints = constraints;
        self.common_mut().flags |= WidgetCommonFlags::CONSTRAINTS;
        self
    }

    fn max_width(mut self, value: f32) -> Self
    where
        Self: Sized,
    {
        self.common_mut().constraints.max_width = value;
        self.common_mut().flags |= WidgetCommonFlags::CONSTRAINTS;
        self
    }

    fn max_height(mut self, value: f32) -> Self
    where
        Self: Sized,
    {
        self.common_mut().constraints.max_height = value;
        self.common_mut().flags |= WidgetCommonFlags::CONSTRAINTS;
        self
    }

    fn min_width(mut self, value: f32) -> Self
    where
        Self: Sized,
    {
        self.common_mut().constraints.min_width = value;
        self.common_mut().flags |= WidgetCommonFlags::CONSTRAINTS;
        self
    }

    fn min_height(mut self, value: f32) -> Self
    where
        Self: Sized,
    {
        self.common_mut().constraints.min_height = value;
        self.common_mut().flags |= WidgetCommonFlags::CONSTRAINTS;
        self
    }

    fn clip(mut self, clip: ::clew::Clip) -> Self
    where
        Self: Sized,
    {
        self.common_mut().clip = clip;
        self.common_mut().flags |= WidgetCommonFlags::CLIP;
        self
    }

    fn offset(mut self, x: f32, y: f32) -> Self
    where
        Self: Sized,
    {
        self.common_mut().offset_x = x;
        self.common_mut().offset_y = y;
        self.common_mut().flags |= WidgetCommonFlags::OFFSET;
        self
    }

    fn offset_x(mut self, offset: f32) -> Self
    where
        Self: Sized,
    {
        self.common_mut().offset_x = offset;
        self.common_mut().flags |= WidgetCommonFlags::OFFSET;
        self
    }

    fn offset_y(mut self, offset: f32) -> Self
    where
        Self: Sized,
    {
        self.common_mut().offset_y = offset;
        self.common_mut().flags |= WidgetCommonFlags::OFFSET;
        self
    }

    fn zindex(mut self, zindex: i32) -> Self
    where
        Self: Sized,
    {
        self.common_mut().zindex = Some(zindex);
        self.common_mut().flags |= WidgetCommonFlags::ZINDEX;
        self
    }

    fn padding(mut self, padding: ::clew::EdgeInsets) -> Self
    where
        Self: Sized,
    {
        self.common_mut().padding = padding;
        self.common_mut().flags |= WidgetCommonFlags::PADDING;
        self
    }

    fn margin(mut self, margin: ::clew::EdgeInsets) -> Self
    where
        Self: Sized,
    {
        self.common_mut().margin = margin;
        self.common_mut().flags |= WidgetCommonFlags::MARGIN;
        self
    }

    fn background(mut self, decorator: ::clew::WidgetRef) -> Self
    where
        Self: Sized,
    {
        self.common_mut().backgrounds.push(decorator);
        self.common_mut().flags |= WidgetCommonFlags::BACKGROUNDS;
        self
    }

    fn foreground(mut self, decorator: ::clew::WidgetRef) -> Self
    where
        Self: Sized,
    {
        self.common_mut().foregrounds.push(decorator);
        self.common_mut().flags |= WidgetCommonFlags::FOREGROUNDS;
        self
    }
}
