use std::{sync::Arc, time::Duration};

use tech_paws_ui::identifiable::Identifiable;
use tech_paws_ui::{
    AlignX, AlignY, ColorRgb,
    render::Renderer,
    text::FontResources,
    widgets::{
        builder::BuildContext,
        button::button,
        for_each::for_each,
        view::{Component, component},
        vstack::vstack,
    },
};
use tech_paws_ui_derive::Identifiable;
use tech_paws_ui_desktop::{
    app::{Application, ApplicationDelegate},
    window::Window,
    window_manager::{WindowDescriptor, WindowManager},
};
use tech_paws_ui_tiny_skia::TinySkiaRenderer;

pub struct DemoApplication {
    counter: u32,
}

#[allow(clippy::new_without_default)]
impl DemoApplication {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CounterEvent {
    Increment,
    Decrement,
}

impl ApplicationDelegate<CounterEvent> for DemoApplication {
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

    fn on_event(
        &mut self,
        _: &mut WindowManager<DemoApplication, CounterEvent>,
        event: &CounterEvent,
    ) {
        match event {
            CounterEvent::Increment => {
                println!("Increment");
                self.counter += 1;
            }
            CounterEvent::Decrement => {
                println!("Decrement");
                self.counter -= 1;
            }
        }
    }

    fn on_start(&mut self, window_manager: &mut WindowManager<DemoApplication, CounterEvent>) {
        window_manager.spawn_window(
            MainWindow::new(),
            WindowDescriptor {
                title: "TODO List".to_string(),
                width: 800,
                height: 600,
                resizable: true,
                fill_color: ColorRgb::from_hex(0x121212),
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

#[allow(clippy::new_without_default)]
impl MainWindow {
    pub fn new() -> Self {
        Self {
            counter: Counter {
                books: vec![
                    Book {
                        id: 12,
                        key: 21,
                        title: "Book 1".to_string(),
                    },
                    Book {
                        id: 113,
                        key: 311,
                        title: "Book 2".to_string(),
                    },
                    Book {
                        id: 114,
                        key: 411,
                        title: "Book 2".to_string(),
                    },
                ],
            },
        }
    }
}

impl Window<DemoApplication, CounterEvent> for MainWindow {
    fn on_event(&mut self, app: &mut DemoApplication, event: &CounterEvent) {
        match event {
            CounterEvent::Increment => {
                println!("Increment");
                app.counter += 1;
            }
            CounterEvent::Decrement => {
                println!("Decrement");
                app.counter -= 1;
            }
        }
    }

    fn build(&mut self, app: &mut DemoApplication, ctx: &mut BuildContext) {
        component(app, &mut self.counter).build(ctx);
    }
}

struct Counter {
    books: Vec<Book>,
}

#[derive(Identifiable)]
struct Book {
    id: u64,
    #[id]
    key: u64,
    title: String,
}

enum CounterComponentEvent {
    HelloWorld,
}

impl Component<DemoApplication, CounterComponentEvent> for Counter {
    fn on_event(&mut self, _: &mut DemoApplication, _: &CounterComponentEvent) -> bool {
        println!("Hello World!");

        true
    }

    fn build(&mut self, app: &mut DemoApplication, ctx: &mut BuildContext) {
        vstack()
            .align_x(AlignX::Center)
            .align_y(AlignY::Center)
            .build(ctx, |ctx| {
                for_each(0..2).build(ctx, |ctx, i| {
                    for_each(&mut self.books).build(ctx, |ctx, book| {
                        if button(&format!("{i}: {}", book.title)).build(ctx).clicked() {
                            book.title = "Changed!".to_string();
                            println!("Clicked to book with id id: {}", book.id());
                        }
                    });
                });

                if button(&format!("Counter: {}", app.counter))
                    .id("counter")
                    .build(ctx)
                    .clicked()
                {
                    ctx.broadcast(CounterEvent::Increment);
                    ctx.emit(CounterComponentEvent::HelloWorld);
                    ctx.spawn(async move {
                        tokio::time::sleep(Duration::from_secs(2)).await;

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
