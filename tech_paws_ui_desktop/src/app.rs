use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tech_paws_ui::event_queue::EventQueue;
use tech_paws_ui::io::Cursor;
use tech_paws_ui::render::{RenderCommand, Renderer, create_test_commands};
use tech_paws_ui::state::UiState;
use tech_paws_ui::text::{FontResources, StringId, StringInterner, TextId, TextsResources};
use tech_paws_ui::widgets::builder::BuildContext;
use tech_paws_ui::{PhysicalSize, View};
use tech_paws_ui::{state::WidgetsStates, task_spawner::TaskSpawner};

use crate::window_manager::WindowState;
use crate::{window::Window, window_manager::WindowManager};
use tokio::sync::mpsc;
#[cfg(target_os = "macos")]
use winit::platform::macos::EventLoopBuilderExtMacOS;

pub trait ApplicationDelegate<Event> {
    fn init_assets(&mut self, fonts: &mut FontResources) {}

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
    fonts: FontResources,
    string_interner: StringInterner,
    strings: HashMap<StringId, TextId>,
    last_cursor: Cursor,
}

fn render<'a, T: ApplicationDelegate<Event>, Event>(
    app: &mut T,
    fonts: &mut FontResources,
    string_interner: &mut StringInterner,
    strings: &mut HashMap<StringId, TextId>,
    task_spawner: &mut TaskSpawner,
    window_state: &mut WindowState<'a, T, Event>,
) {
    window_state.ui_state.before_render();

    let mut build_context = BuildContext {
        current_zindex: 0,
        layout_commands: &mut window_state.ui_state.layout_commands,
        widgets_states: &mut window_state.ui_state.widgets_states,
        task_spawner: task_spawner,
        event_queue: &mut window_state.ui_state.event_queue,
        text: &mut window_state.texts,
        fonts,
        view: &window_state.ui_state.view,
        string_interner,
    };

    window_state.window.build(app, &mut build_context);
    tech_paws_ui::render(
        &mut window_state.ui_state,
        &mut window_state.texts,
        fonts,
        string_interner,
        strings,
    );
}

impl<T: ApplicationDelegate<Event>, Event> winit::application::ApplicationHandler
    for Application<'_, T, Event>
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
        let window = self.window_manager.get_mut_window(window_id).unwrap();
        let input_cursor = window.ui_state.user_input.cursor;

        if self.last_cursor != input_cursor
        /* || ui_state.parameters.should_update_cursor_each_frame*/
        {
            let cursor = match input_cursor {
                Cursor::Default => winit::window::CursorIcon::Default,
                Cursor::Pointer => winit::window::CursorIcon::Pointer,
                Cursor::Text => winit::window::CursorIcon::Text,
                Cursor::EwResize => winit::window::CursorIcon::EwResize,
                Cursor::NsResize => winit::window::CursorIcon::NsResize,
                Cursor::NeswResize => winit::window::CursorIcon::NeswResize,
                Cursor::NwseResize => winit::window::CursorIcon::NwseResize,
            };

            window
                .winit_window
                .set_cursor(winit::window::Cursor::Icon(cursor));
            self.last_cursor = input_cursor;
        }

        match event {
            winit::event::WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(size) => {
                window.ui_state.view.size = PhysicalSize::new(size.width, size.height);
                self.window_manager.request_redraw(window_id);
            }
            winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                window.ui_state.view.scale_factor = scale_factor as f32;
                window
                    .texts
                    .update_view(&window.ui_state.view, &mut self.fonts);

                self.window_manager.request_redraw(window_id);
            }
            winit::event::WindowEvent::RedrawRequested => {
                window.winit_window.request_redraw();

                render(
                    &mut self.app,
                    &mut self.fonts,
                    &mut self.string_interner,
                    &mut self.strings,
                    &mut self.task_spawner,
                    window,
                );

                window.renderer.process_commands(
                    &window.ui_state.view,
                    &window.ui_state.render_state,
                    &mut self.fonts,
                    &mut window.texts,
                );
            }
            winit::event::WindowEvent::MouseInput {
                state: btn_state,
                button,
                ..
            } => {
                window.ui_state.user_input.mouse_pressed =
                    btn_state == winit::event::ElementState::Pressed;
                window.ui_state.user_input.mouse_released =
                    btn_state == winit::event::ElementState::Released;

                match button {
                    winit::event::MouseButton::Left => {
                        window.ui_state.user_input.mouse_left_pressed =
                            window.ui_state.user_input.mouse_pressed;
                        window.ui_state.user_input.mouse_left_released =
                            window.ui_state.user_input.mouse_released;
                    }
                    winit::event::MouseButton::Right => {
                        window.ui_state.user_input.mouse_right_pressed =
                            window.ui_state.user_input.mouse_pressed;
                        window.ui_state.user_input.mouse_right_released =
                            window.ui_state.user_input.mouse_released;
                    }
                    winit::event::MouseButton::Middle => {
                        window.ui_state.user_input.mouse_middle_pressed =
                            window.ui_state.user_input.mouse_pressed;
                        window.ui_state.user_input.mouse_middle_released =
                            window.ui_state.user_input.mouse_released;
                    }
                    _ => {}
                }
            }

            // Mouse wheel scrolling
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        window.ui_state.user_input.mouse_wheel_delta_x = x as f64 * 20.0; // Scale line delta
                        window.ui_state.user_input.mouse_wheel_delta_y = y as f64 * 20.0;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        window.ui_state.user_input.mouse_wheel_delta_x = pos.x;
                        window.ui_state.user_input.mouse_wheel_delta_y = pos.y;
                    }
                }
            }

            // Mouse movement
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                window.ui_state.user_input.mouse_x = position.x;
                window.ui_state.user_input.mouse_y = position.y;
            }

            // Focus events
            winit::event::WindowEvent::Focused(focused) => {
                if !focused {
                    // Clear input state when window loses focus
                    // input.keys_pressed.clear();
                    window.ui_state.user_input.mouse_left_pressed = false;
                    window.ui_state.user_input.mouse_right_pressed = false;
                    window.ui_state.user_input.mouse_middle_pressed = false;

                    window.winit_window.set_cursor(winit::window::Cursor::Icon(
                        winit::window::CursorIcon::Default,
                    ));
                    self.last_cursor = Cursor::Default;
                    window.ui_state.user_input.cursor = Cursor::Default;
                }
            }
            _ => (),
        }
    }
}

impl<T: ApplicationDelegate<Event>, Event> Application<'_, T, Event> {
    pub fn run_application(mut delegate: T) -> anyhow::Result<()> {
        let (redraw_tx, redraw_rx) = mpsc::unbounded_channel();

        let mut fonts = FontResources::new();
        delegate.init_assets(&mut fonts);

        let mut application = Application {
            app: delegate,
            window_manager: WindowManager::new(T::create_renderer),
            task_spawner: TaskSpawner::new(redraw_tx),
            redraw_rx,
            fonts,
            string_interner: StringInterner::new(),
            strings: HashMap::new(),
            last_cursor: Cursor::Default,
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
