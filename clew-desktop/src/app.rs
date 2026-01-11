use std::any::{Any, TypeId};
use std::sync::Arc;
use std::time::Instant;

use clew::PhysicalSize;
use clew::assets::Assets;
use clew::io::Cursor;
use clew::render::Renderer;
use clew::text::{FontResources, StringInterner};
use clew::widgets::builder::{ApplicationEvent, ApplicationEventLoopProxy, BuildContext};

use crate::window_manager::WindowManager;
use crate::window_manager::WindowState;
#[cfg(target_os = "macos")]
use winit::platform::macos::EventLoopBuilderExtMacOS;

pub trait ApplicationDelegate<Event> {
    fn init_assets(&mut self, _assets: &mut Assets) {}

    fn on_start(&mut self, window_manager: &mut WindowManager<Self, Event>)
    where
        Self: std::marker::Sized;

    fn on_event(&mut self, _window_manager: &mut WindowManager<Self, Event>, _event: &Event)
    where
        Self: std::marker::Sized,
    {
    }

    fn create_renderer(window: Arc<winit::window::Window>) -> Box<dyn Renderer>;
}

pub struct Application<'a, T: ApplicationDelegate<Event>, Event = ()> {
    app: T,
    window_manager: WindowManager<'a, T, Event>,
    fonts: FontResources,
    assets: Assets<'a>,
    string_interner: StringInterner,
    last_cursor: Cursor,
    broadcast_event_queue: Vec<Arc<dyn Any + Send>>,
    broadcast_async_tx: tokio::sync::mpsc::UnboundedSender<Box<dyn Any + Send>>,
    broadcast_async_rx: tokio::sync::mpsc::UnboundedReceiver<Box<dyn Any + Send>>,
    event_loop_proxy: Arc<WinitEventLoopProxy>,
    force_redraw: bool,
    needs_redraw: bool,
}

pub struct WinitEventLoopProxy {
    proxy: winit::event_loop::EventLoopProxy<ApplicationEvent>,
}

impl ApplicationEventLoopProxy for WinitEventLoopProxy {
    fn send_event(&self, event: ApplicationEvent) {
        let _ = self.proxy.send_event(event);
    }
}

#[allow(clippy::too_many_arguments)]
fn render<'a, T: ApplicationDelegate<Event>, Event: 'static>(
    app: &mut T,
    fonts: &mut FontResources,
    assets: &Assets,
    string_interner: &mut StringInterner,
    broadcast_event_queue: &mut Vec<Arc<dyn Any + Send>>,
    broadcast_async_tx: &mut tokio::sync::mpsc::UnboundedSender<Box<dyn Any + Send>>,
    window_state: &mut WindowState<'a, T, Event>,
    event_loop_proxy: Arc<WinitEventLoopProxy>,
    force_redraw: bool,
) -> bool {
    window_state.ui_state.before_render();

    for event_box in window_state.ui_state.current_event_queue.iter() {
        // Skip event processing for () type
        if TypeId::of::<Event>() != TypeId::of::<()>()
            && let Some(event) = event_box.downcast_ref::<Event>()
        {
            window_state.window.on_event(app, event);
        }
    }

    for event_box in broadcast_event_queue.iter() {
        window_state
            .ui_state
            .current_event_queue
            .push(event_box.clone());
    }

    broadcast_event_queue.clear();
    window_state.animations_stepped_this_frame.clear();

    let mut build_context = BuildContext {
        child_nth: 0,
        last_child_nth: 0,
        ignore_pointer: false,
        layout_commands: &mut window_state.ui_state.layout_commands,
        widgets_states: &mut window_state.ui_state.widgets_states,
        event_queue: &mut window_state.ui_state.current_event_queue,
        next_event_queue: &mut window_state.ui_state.next_event_queue,
        text: &mut window_state.texts,
        fonts,
        view: &window_state.ui_state.view,
        string_interner,
        async_tx: &mut window_state.ui_state.async_tx,
        broadcast_event_queue,
        broadcast_async_tx,
        event_loop_proxy,
        id_seed: None,
        user_data: None,
        scoped_user_data: None,
        strings: &mut window_state.strings,
        phase_allocator: &mut window_state.ui_state.phase_allocator,
        backgrounds: &mut window_state.ui_state.backgrounds,
        input: &window_state.ui_state.user_input,
        interaction: &mut window_state.ui_state.interaction_state,
        delta_time: window_state.delta_time_timer.elapsed().as_secs_f32(),
        animations_stepped_this_frame: &mut window_state.animations_stepped_this_frame,
        foregrounds: &mut window_state.ui_state.foregrounds,
        non_interactable: &mut window_state.ui_state.non_interactable,
        layout: None,
        child_nth_stack: Vec::new(),
        decoration_defer: Vec::new(),
        decoration_defer_start_stack: Vec::new(),
    };

    window_state.delta_time_timer = Instant::now();
    window_state.window.build(app, &mut build_context);

    clew::render(
        &mut window_state.ui_state,
        &mut window_state.texts,
        fonts,
        assets,
        string_interner,
        &mut window_state.strings,
        force_redraw,
    )
}

impl<T: ApplicationDelegate<Event>, Event: 'static>
    winit::application::ApplicationHandler<ApplicationEvent> for Application<'_, T, Event>
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        self.window_manager
            .with_event_loop(event_loop, |window_manager| {
                self.app.on_start(window_manager);
            });
    }

    fn user_event(&mut self, _: &winit::event_loop::ActiveEventLoop, event: ApplicationEvent) {
        match event {
            ApplicationEvent::Wake { view_id } => {
                self.window_manager.request_view_redraw(view_id);
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        // Request redraw for all windows that need it
        for window in self.window_manager.windows.values() {
            // if self.needs_redraw {
            window.winit_window.request_redraw();
            // }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // Collect async events
        while let Ok(event) = self.broadcast_async_rx.try_recv() {
            self.broadcast_event_queue.push(event.into());
        }

        for event_box in self.broadcast_event_queue.iter() {
            if let Some(event) = event_box.downcast_ref::<Event>() {
                self.app.on_event(&mut self.window_manager, event);

                for window in self.window_manager.windows.values_mut() {
                    window.window.on_event(&mut self.app, event);
                }
            }
        }

        if !matches!(event, winit::event::WindowEvent::RedrawRequested) {
            self.broadcast_event_queue.clear();
        }

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
                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(size) => {
                window.ui_state.view.size = PhysicalSize::new(size.width, size.height);
                self.force_redraw = true;

                window.ui_state.user_input.mouse_left_pressed = false;
                window.ui_state.user_input.mouse_right_pressed = false;
                window.ui_state.user_input.mouse_middle_pressed = false;
                window.ui_state.user_input.mouse_left_released = false;
                window.ui_state.user_input.mouse_right_released = false;
                window.ui_state.user_input.mouse_middle_released = false;
                window.ui_state.user_input.mouse_pressed = false;
                window.ui_state.user_input.mouse_released = false;
                window.ui_state.user_input.mouse_x = -1.;
                window.ui_state.user_input.mouse_y = -1.;
                window.ui_state.user_input.mouse_wheel_delta_x = 0.;
                window.ui_state.user_input.mouse_wheel_delta_y = 0.;
                window.ui_state.user_input.mouse_left_click_count = 0;

                self.window_manager.request_redraw(window_id);
            }
            winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                window.ui_state.view.scale_factor = scale_factor as f32;
                window
                    .texts
                    .update_view(&window.ui_state.view, &mut self.fonts);

                self.force_redraw = true;
                self.window_manager.request_redraw(window_id);
            }
            winit::event::WindowEvent::RedrawRequested => {
                let need_to_redraw = render(
                    &mut self.app,
                    &mut self.fonts,
                    &self.assets,
                    &mut self.string_interner,
                    &mut self.broadcast_event_queue,
                    &mut self.broadcast_async_tx,
                    window,
                    self.event_loop_proxy.clone(),
                    self.force_redraw,
                );

                if need_to_redraw {
                    window.renderer.process_commands(
                        &window.ui_state.view,
                        &window.ui_state.render_state,
                        window.fill_color,
                        &mut self.fonts,
                        &mut window.texts,
                        &self.assets,
                    );

                    window.winit_window.request_redraw();
                    self.force_redraw = false;
                }
            }
            winit::event::WindowEvent::MouseInput {
                state: btn_state,
                button,
                ..
            } => {
                // window.winit_window.request_redraw();
                self.needs_redraw = true;

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
                self.needs_redraw = true;
                // window.winit_window.request_redraw();

                match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        // Scale line delta
                        window.ui_state.user_input.mouse_wheel_delta_x = x * 20.0;
                        window.ui_state.user_input.mouse_wheel_delta_y = y * 20.0;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        window.ui_state.user_input.mouse_wheel_delta_x = pos.x as f32;
                        window.ui_state.user_input.mouse_wheel_delta_y = pos.y as f32;
                    }
                }
            }

            // Mouse movement
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                // window.winit_window.request_redraw();
                self.needs_redraw = true;

                window.ui_state.user_input.mouse_x = position.x as f32;
                window.ui_state.user_input.mouse_y = position.y as f32;
            }

            // Focus events
            winit::event::WindowEvent::Focused(focused) => {
                window.winit_window.request_redraw();

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

impl<T: ApplicationDelegate<Event>, Event: 'static> Application<'_, T, Event> {
    pub fn run_application(mut delegate: T) -> anyhow::Result<()> {
        let (broadcast_async_tx, broadcast_async_rx) = tokio::sync::mpsc::unbounded_channel();

        let mut assets = Assets::new();

        delegate.init_assets(&mut assets);

        let fonts = assets.create_font_resources();

        #[cfg(target_os = "macos")]
        let event_loop = winit::event_loop::EventLoop::with_user_event()
            .with_activation_policy(winit::platform::macos::ActivationPolicy::Regular)
            .build()?;

        #[cfg(not(target_os = "macos"))]
        let event_loop = winit::event_loop::EventLoop::with_user_event().build()?;

        let event_proxy = event_loop.create_proxy();

        let mut application = Application {
            app: delegate,
            window_manager: WindowManager::new(T::create_renderer),
            fonts,
            string_interner: StringInterner::new(),
            last_cursor: Cursor::Default,
            broadcast_event_queue: Vec::new(),
            broadcast_async_rx,
            broadcast_async_tx,
            force_redraw: false,
            needs_redraw: false,
            event_loop_proxy: Arc::new(WinitEventLoopProxy { proxy: event_proxy }),
            assets,
        };

        event_loop.run_app(&mut application)?;

        Ok(())
    }
}
