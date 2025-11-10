use std::{collections::HashMap, rc::Rc, sync::Arc};

use crate::window::Window;

#[derive(Debug, Clone)]
pub struct WindowDescriptor {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
}

impl Default for WindowDescriptor {
    fn default() -> Self {
        Self {
            title: "Window".to_string(),
            width: 800,
            height: 600,
            resizable: true,
        }
    }
}

pub struct WindowId {
    winit_window_id: winit::window::WindowId,
}

impl WindowId {
    fn new(id: winit::window::WindowId) -> Self {
        Self {
            winit_window_id: id,
        }
    }
}

pub struct WindowManager<App, Event> {
    windows: HashMap<winit::window::WindowId, Box<dyn Window<App, Event>>>,
    winit_windows: HashMap<winit::window::WindowId, Arc<winit::window::Window>>,
    event_loop: Option<*const winit::event_loop::ActiveEventLoop>,
}

impl<App, Event> WindowManager<App, Event> {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            winit_windows: HashMap::new(),
            event_loop: None,
        }
    }

    pub fn with_event_loop<F>(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        callback: F,
    ) where
        F: FnOnce(&mut WindowManager<App, Event>),
    {
        self.event_loop = Some(event_loop);
        callback(self);
        self.event_loop = None;
    }

    /// Create a new window with the given descriptor
    pub fn spawn_window<T: Window<App, Event> + 'static>(
        &mut self,
        window: T,
        descriptor: WindowDescriptor,
    ) {
        if let Some(event_loop) = self.event_loop {
            let attributes = winit::window::WindowAttributes::default()
                .with_title(descriptor.title)
                .with_inner_size(winit::dpi::LogicalSize::new(
                    descriptor.width,
                    descriptor.height,
                ))
                .with_resizable(descriptor.resizable);

            let event_loop = unsafe { &*event_loop };
            match event_loop.create_window(attributes) {
                Ok(winit_window) => {
                    let winit_window = Arc::new(winit_window);
                    let id = winit_window.id();
                    self.winit_windows.insert(id, winit_window);
                    self.windows.insert(id, Box::new(window));

                    log::debug!("Created window: {id:?}");
                }
                Err(e) => {
                    log::error!("Failed to create window: {e}");
                }
            }
        } else {
            log::error!("Event loop has not been set");
        }
    }

    pub fn get_window(&self, id: winit::window::WindowId) -> Option<&Box<dyn Window<App, Event>>> {
        self.windows.get(&id)
    }

    pub fn get_mut_window(
        &mut self,
        id: winit::window::WindowId,
    ) -> Option<&mut Box<dyn Window<App, Event>>> {
        self.windows.get_mut(&id)
    }

    pub(crate) fn get_winit_window(
        &self,
        id: winit::window::WindowId,
    ) -> Option<&Arc<winit::window::Window>> {
        self.winit_windows.get(&id)
    }

    pub fn request_redraw(&self, id: winit::window::WindowId) {
        if let Some(window) = self.winit_windows.get(&id) {
            window.request_redraw();
        }
    }

    pub fn request_redraw_all(&self) {
        for window in self.winit_windows.values() {
            window.request_redraw();
        }
    }
}
