use clew_widgets::button;
use pollster::FutureExt;
use tech_paws_ui::{
    AlignX, AlignY, ColorRgb, CrossAxisAlignment,
    render::Renderer,
    widgets::{builder::BuildContext, hstack::hstack, text::text, vstack::vstack, zstack::zstack},
};
use tech_paws_ui_desktop::{
    app::{Application, ApplicationDelegate},
    window::Window,
    window_manager::{WindowDescriptor, WindowManager},
};
use tech_paws_ui_vello::VelloRenderer;

struct CounterApplication;

impl ApplicationDelegate<()> for CounterApplication {
    fn on_start(&mut self, window_manager: &mut WindowManager<Self, ()>)
    where
        Self: std::marker::Sized,
    {
        window_manager.spawn_window(
            MainWindow { counter: 0 },
            WindowDescriptor {
                title: "Counter".to_string(),
                width: 800,
                height: 600,
                resizable: true,
                fill_color: ColorRgb::from_hex(0x121212),
            },
        );
    }

    fn create_renderer(window: std::sync::Arc<winit::window::Window>) -> Box<dyn Renderer> {
        Box::new(
            VelloRenderer::new(
                window.clone(),
                window.inner_size().width,
                window.inner_size().height,
            )
            .block_on(),
        )
    }
}

pub struct MainWindow {
    counter: i32,
}

impl Window<CounterApplication, ()> for MainWindow {
    fn build(&mut self, _: &mut CounterApplication, ctx: &mut BuildContext) {
        zstack()
            .fill_max_size()
            .align_x(AlignX::Center)
            .align_y(AlignY::Center)
            .build(ctx, |ctx| {
                vstack()
                    .spacing(12.)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .build(ctx, |ctx| {
                        text(
                            &bumpalo::format!(in &ctx.phase_allocator, "Counter: {}", self.counter),
                        )
                        .build(ctx);

                        hstack().build(ctx, |ctx| {
                            if button("+").build(ctx).clicked() {
                                self.counter += 1;
                            }

                            if button("-").build(ctx).clicked() {
                                self.counter -= 1;
                            }
                        });
                    });
            });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracy_client::Client::start();

    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    log::info!("Starting app");
    Application::run_application(CounterApplication)?;

    Ok(())
}
