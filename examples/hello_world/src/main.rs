use std::{sync::Arc, time::Duration};

use tech_paws_ui::{
    AlignX, AlignY, SizeConstraint,
    render::Renderer,
    widgets::{
        builder::BuildContext,
        button::button,
        hstack::hstack,
        view::{Component, component},
        vstack::vstack,
    },
};
use tech_paws_ui_desktop::{
    app::{Application, ApplicationDelegate},
    window::Window,
    window_manager::{WindowDescriptor, WindowManager},
};
use tech_paws_ui_tiny_skia::TinySkiaRenderer;

pub struct DemoApplication {}

impl DemoApplication {
    pub fn new() -> Self {
        Self {}
    }
}

impl ApplicationDelegate<()> for DemoApplication {
    fn on_start(&mut self, window_manager: &mut WindowManager<DemoApplication, ()>) {
        window_manager.spawn_window(
            MainWindow::new(),
            WindowDescriptor {
                title: "TODO List".to_string(),
                width: 800,
                height: 600,
                resizable: true,
            },
        );
    }

    fn create_renderer(window: Arc<winit::window::Window>) -> Box<dyn Renderer> {
        Box::new(TinySkiaRenderer::new(window.clone(), window.clone()))
    }
}

pub struct MainWindow {
    counter: Counter,
}

impl MainWindow {
    pub fn new() -> Self {
        Self {
            counter: Counter { value: 0 },
        }
    }
}

impl Window<DemoApplication, ()> for MainWindow {
    fn build(&mut self, app: &mut DemoApplication, ctx: &mut BuildContext) {
        component(&mut self.counter).build(ctx);
    }
}

struct Counter {
    value: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum CounterEvent {
    Increment,
    Decrement,
}

impl Component<CounterEvent> for Counter {
    fn on_event(&mut self, event: &CounterEvent) -> bool {
        match event {
            CounterEvent::Increment => {
                self.value += 1;
            }
            CounterEvent::Decrement => {
                self.value -= 1;
            }
        }

        true
    }

    fn build(&mut self, ctx: &mut BuildContext) {
        vstack().spacing(32.).fill_max_size().build(ctx, |ctx| {
            hstack().fill_max_size().build(ctx, |ctx| {
                if button("Button 1")
                    .align_y(AlignY::Center)
                    .build(ctx)
                    .clicked()
                {
                    log::info!("Button 1 clicked");
                }

                if button("Button 4").fill_max_width().build(ctx).clicked() {
                    log::info!("Button 4 clicked");
                }

                vstack().width(SizeConstraint::Fill(2.)).build(ctx, |ctx| {
                    if button("vstack 1").fill_max_width().build(ctx).clicked() {
                        log::info!("Button 2 clicked");
                    }
                    if button("vstack 2").fill_max_width().build(ctx).clicked() {
                        log::info!("Button 3 clicked");
                    }
                    if button("vstack 2").fill_max_width().build(ctx).clicked() {
                        log::info!("Button 3 clicked");
                    }
                });

                if button("Button 4").build(ctx).clicked() {
                    log::info!("Button 4 clicked");
                }
            });

            vstack().fill_max_size().build(ctx, |ctx| {
                button("vstack 2").align_x(AlignX::End).build(ctx);
            });

            hstack().fill_max_width().build(ctx, |ctx| {
                if button("Button 1").build(ctx).clicked() {
                    log::info!("Button 1 clicked");
                }

                if button("Button 4")
                    .fill_max_width()
                    .build(ctx)
                    .clicked()
                {
                    log::info!("Button 4 clicked");
                }

                vstack().fill_max_width().build(ctx, |ctx| {
                    if button("vstack 1").fill_max_width().build(ctx).clicked() {
                        log::info!("Button 2 clicked");
                    }
                    if button("vstack 2").fill_max_width().build(ctx).clicked() {
                        log::info!("Button 3 clicked");
                    }
                });

                if button("Button 4").build(ctx).clicked() {
                    log::info!("Button 4 clicked");
                }
            });
        });

        // vstack().fill_max_size().build(ctx, |ctx| {
        //     if button_id("counter", &format!("Counter: {}", self.value))
        //         .align_x(AlignX::Center)
        //         .align_y(AlignY::Center)
        //         .build(ctx)
        //         .clicked()
        //     {
        //         ctx.emit(CounterEvent::Increment);
        //         let value = self.value;
        //         ctx.spawn(async move {
        //             tokio::time::sleep(Duration::from_secs(2)).await;
        //             println!("Current counter: {value}");

        //             CounterEvent::Decrement
        //         });
        //     };
        // });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    log::info!("Starting app");
    Application::run_application(DemoApplication::new())?;

    Ok(())
}
