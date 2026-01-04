use std::time::Duration;

use clew as ui;
use clew_desktop::{
    app::{Application, ApplicationDelegate},
    window::Window,
    window_manager::{WindowDescriptor, WindowManager},
};
use clew_vello::VelloRenderer;
use pollster::FutureExt;

struct AnimationsApplication;

impl ApplicationDelegate<()> for AnimationsApplication {
    fn on_start(&mut self, window_manager: &mut WindowManager<Self, ()>)
    where
        Self: std::marker::Sized,
    {
        window_manager.spawn_window(
            MainWindow::new(),
            WindowDescriptor {
                title: "Animations".to_string(),
                width: 800,
                height: 600,
                resizable: true,
                fill_color: ui::ColorRgb::from_hex(0x121212),
            },
        );
    }

    fn create_renderer(window: std::sync::Arc<winit::window::Window>) -> Box<dyn ui::Renderer> {
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

struct MainWindow {
    offset_y: ui::Tween<f32, f32>,
    mx: ui::Damp<f32, f32>,
    my: ui::Damp<f32, f32>,
}

impl MainWindow {
    fn new() -> Self {
        Self {
            offset_y: ui::Tween::new(0.)
                .duration(Duration::from_secs(1))
                .curve(ui::curves::f32::ease_out_elastic),
            mx: ui::Damp::new(0.).speed(10.),
            my: ui::Damp::new(0.).speed(10.),
        }
    }
}

impl Window<AnimationsApplication, ()> for MainWindow {
    fn build(&mut self, _: &mut AnimationsApplication, ctx: &mut ui::BuildContext) {
        self.mx.approach(ctx.input.mouse_x / ctx.view.scale_factor);
        self.my.approach(ctx.input.mouse_y / ctx.view.scale_factor);

        ctx.step_animation(&mut self.offset_y);
        ctx.step_animation(&mut self.mx);
        ctx.step_animation(&mut self.my);

        ui::zstack().fill_max_size().build(ctx, |ctx| {
            ui::zstack()
                .fill_max_size()
                .align_x(ui::AlignX::Center)
                .align_y(ui::AlignY::Center)
                .offset_y(self.offset_y.value())
                .build(ctx, |ctx| {
                    ui::vstack()
                        .spacing(12.)
                        .cross_axis_alignment(ui::CrossAxisAlignment::Center)
                        .build(ctx, |ctx| {
                            ui::text(&format!("Offset: {}", self.offset_y.value())).build(ctx);
                            ui::text(&format!("Tween In Status: {:?}", self.offset_y.status()))
                                .build(ctx);
                            ui::text(&format!("MX In Status: {:?}", self.mx.status())).build(ctx);
                            ui::text(&format!("MY In Status: {:?}", self.my.status())).build(ctx);

                            if clew_widgets::button("Move Up").build(ctx).clicked() {
                                self.offset_y.tween_to(-100.);
                            }

                            if clew_widgets::button("Move Down").build(ctx).clicked() {
                                self.offset_y.tween_to(100.);
                            }

                            if clew_widgets::button("Go Home").build(ctx).clicked() {
                                self.offset_y.tween_to(0.);
                            }
                        });
                });

            ui::decorated_box()
                .shape(ui::BoxShape::Oval)
                .color(ui::ColorRgba::from_hex(0xFFFF0000))
                .width(32.)
                .height(32.)
                .offset(self.mx.value() - 16., self.my.value() - 16.)
                .build(ctx);
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
    Application::run_application(AnimationsApplication)?;

    Ok(())
}
