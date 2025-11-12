use std::{collections::HashMap, rc::Rc, sync::Arc};

use tech_paws_ui::{
    EdgeInsets, PhysicalSize, View, render::Renderer, state::UiState, text::TextsResources,
};

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

pub(crate) struct WindowState<'a, App, Event> {
    pub(crate) window: Box<dyn Window<App, Event>>,
    pub(crate) winit_window: Arc<winit::window::Window>,
    pub(crate) texts: TextsResources<'a>,
    pub(crate) ui_state: UiState,
    pub(crate) renderer: Box<dyn Renderer>,
}

pub struct WindowManager<'a, App, Event> {
    pub(crate) windows: HashMap<winit::window::WindowId, WindowState<'a, App, Event>>,
    event_loop: Option<*const winit::event_loop::ActiveEventLoop>,
    renderer_factory: fn(Arc<winit::window::Window>) -> Box<dyn Renderer>,
}

impl<'a, App, Event> WindowManager<'a, App, Event> {
    pub fn new(renderer_factory: fn(Arc<winit::window::Window>) -> Box<dyn Renderer>) -> Self {
        Self {
            windows: HashMap::new(),
            event_loop: None,
            renderer_factory,
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
                    let scale_factor = winit_window.scale_factor();
                    let inner_size = winit_window.inner_size();
                    let renderer = (self.renderer_factory)(winit_window.clone());
                    let ui_state = UiState::new(View {
                        size: PhysicalSize::new(inner_size.width, inner_size.height),
                        scale_factor: scale_factor as f32,
                        safe_area: EdgeInsets::ZERO,
                    });

                    self.windows.insert(
                        id,
                        WindowState {
                            window: Box::new(window),
                            winit_window,
                            texts: TextsResources::new(),
                            ui_state,
                            renderer,
                        },
                    );

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

    pub fn get_window(&self, id: winit::window::WindowId) -> Option<&WindowState<'a, App, Event>> {
        self.windows.get(&id)
    }

    pub fn get_mut_window(
        &mut self,
        id: winit::window::WindowId,
    ) -> Option<&mut WindowState<'a, App, Event>> {
        self.windows.get_mut(&id)
    }

    pub fn request_redraw(&self, id: winit::window::WindowId) {
        if let Some(window) = self.windows.get(&id) {
            window.winit_window.request_redraw();
        }
    }

    pub fn request_redraw_all(&self) {
        for window in self.windows.values() {
            window.winit_window.request_redraw();
        }
    }
}
