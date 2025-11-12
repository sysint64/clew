use std::{sync::Arc, time::Duration};

use tech_paws_ui::{
    AlignX, AlignY, SizeConstraint,
    render::Renderer,
    text::FontResources,
    widgets::{
        builder::BuildContext,
        button::{button, button_id},
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
    fn init_assets(&mut self, fonts: &mut FontResources) {
        fonts.load_font("Inter", include_bytes!("../../assets/fonts/Inter.ttf"));
        fonts.load_font(
            "Source Han Serif",
            include_bytes!("../../assets/fonts/SourceHanSerif-Regular.otf"),
        );
        fonts.load_font(
            "Noto Emoji",
            include_bytes!("../../assets/fonts/NotoEmoji-VariableFont_wght.ttf"),
        );

        log::info!("Loaded fonts");
    }

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
                println!("Increment");
                self.value += 1;
            }
            CounterEvent::Decrement => {
                println!("Decrement");
                self.value -= 1;
            }
        }

        true
    }

    fn build(&mut self, ctx: &mut BuildContext) {
        vstack().fill_max_size().build(ctx, |ctx| {
            if button_id("counter", &format!("Counter: {}", self.value))
                .align_x(AlignX::Center)
                .align_y(AlignY::Center)
                .build(ctx)
                .clicked()
            {
                ctx.spawn(async move {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    println!("LETS GO!");

                    CounterEvent::Increment
                });
            };
        });
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
