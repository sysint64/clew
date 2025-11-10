use std::collections::HashMap;
use std::sync::Arc;

use tech_paws_ui::PhysicalSize;
use tech_paws_ui::event_queue::EventQueue;
use tech_paws_ui::render::Renderer;
use tech_paws_ui::widgets::builder::BuildContext;
use tech_paws_ui::{state::WidgetsStates, task_spawner::TaskSpawner};

use crate::{window::Window, window_manager::WindowManager};
use tokio::sync::mpsc;
#[cfg(target_os = "macos")]
use winit::platform::macos::EventLoopBuilderExtMacOS;

pub trait ApplicationDelegate<Event> {
    fn on_start(&mut self, window_manager: &mut WindowManager<Self, Event>)
    where
        Self: std::marker::Sized;

    fn on_event(&mut self, event: Event) {}

    fn create_renderer(window: Arc<winit::window::Window>) -> Box<dyn Renderer>;
}

pub struct Application<'a, T: ApplicationDelegate<Event>, Event = ()> {
    app: T,
    window_manager: WindowManager<'a, T, Event>,
    task_spawner: TaskSpawner,
    redraw_rx: mpsc::UnboundedReceiver<()>,
}

fn render<T: ApplicationDelegate<Event>, Event>(
    app: &mut T,
    task_spawner: &mut TaskSpawner,
    window: &mut Box<dyn Window<T, Event>>,
) {
    let mut layout_commands = Vec::new();
    let mut widgets_states = WidgetsStates::default();
    let mut queue = EventQueue::new();
    let mut build_context = BuildContext {
        current_zindex: 0,
        layout_commands: &mut layout_commands,
        widgets_states: &mut widgets_states,
        task_spawner: task_spawner,
        event_queue: &mut queue,
    };

    window.build(app, &mut build_context);
}

impl<'a, T: ApplicationDelegate<Event>, Event> winit::application::ApplicationHandler
    for Application<'a, T, Event>
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window_manager
            .with_event_loop(event_loop, |window_manager| {
                self.app.on_start(window_manager);
            });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(size) => {
                let ui_state = self.window_manager.get_mut_ui_state(window_id).unwrap();
                ui_state.view.size = PhysicalSize::new(size.width, size.height);
                self.window_manager.request_redraw(window_id);
            }
            winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let ui_state = self.window_manager.get_mut_ui_state(window_id).unwrap();
                ui_state.view.scale_factor = scale_factor as f32;
                ui_state
                    .render_state
                    .texts
                    .update_view(&ui_state.view, &mut ui_state.render_state.fonts);
                self.window_manager.request_redraw(window_id);
            }
            winit::event::WindowEvent::RedrawRequested => {
                self.window_manager
                    .get_winit_window(window_id)
                    .unwrap()
                    .request_redraw();

                let window = self.window_manager.get_mut_window(window_id).unwrap();
                render(&mut self.app, &mut self.task_spawner, window);

                let (ui_state, renderer) =
                    self.window_manager.get_ui_state_and_renderer_mut(window_id);
                let ui_state = ui_state.unwrap();
                let renderer = renderer.unwrap();

                renderer.process_commands(&ui_state.view, &ui_state.render_state, &[]);
            }
            _ => (),
        }
    }
}

impl<T: ApplicationDelegate<Event>, Event> Application<'_, T, Event> {
    pub fn run_application(delegate: T) -> anyhow::Result<()> {
        let (redraw_tx, redraw_rx) = mpsc::unbounded_channel();

        let mut application = Application {
            app: delegate,
            window_manager: WindowManager::new(T::create_renderer),
            task_spawner: TaskSpawner::new(redraw_tx),
            redraw_rx,
        };

        #[cfg(target_os = "macos")]
        {
            let event_loop = winit::event_loop::EventLoop::with_user_event()
                .with_activation_policy(winit::platform::macos::ActivationPolicy::Regular)
                .build()?;

            event_loop.run_app(&mut application)?;
        }

        #[cfg(not(target_os = "macos"))]
        {
            let event_loop = winit::event_loop::EventLoop::with_user_event().build()?;

            event_loop.run_app(&mut application)?;
        }

        Ok(())
    }
}
