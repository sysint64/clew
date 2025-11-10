use std::{sync::Arc, time::Duration};

use tech_paws_ui::widgets::{
    builder::BuildContext,
    button::{self, button_id},
    view::{Component, component},
    vstack::vstack,
};
use tech_paws_ui_desktop::{
    app::{Application, ApplicationDelegate},
    window::Window,
    window_manager::{WindowDescriptor, WindowManager},
};
use tech_paws_ui_tiny_skia::TinySkiaRenderer;

pub struct DemoApplication {
    pub todo_list: Vec<String>,
}

impl DemoApplication {
    pub fn new() -> Self {
        Self {
            todo_list: vec!["Task 1".to_string(), "Task 2".to_string()],
        }
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

    fn create_renderer(window: Arc<winit::window::Window>) -> Box<dyn tech_paws_ui::render::Renderer> {
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
pub enum Message {
    Increment,
    Decrement,
}

impl Component<Message> for Counter {
    fn on_event(&mut self, event: &Message) -> bool {
        match event {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }

        true
    }

    fn build(&mut self, ctx: &mut BuildContext) {
        vstack().build(ctx, |ctx| {
            if button_id("counter", &format!("Counter: {}", self.value))
                .build(ctx)
                .clicked()
            {
                ctx.emit(Message::Increment);
                let value = self.value;
                ctx.spawn(async move {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    println!("Current counter: {value}");

                    Message::Decrement
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
