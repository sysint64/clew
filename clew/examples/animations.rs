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
    keyframes: ui::Keyframes<f32>,
}

impl MainWindow {
    fn new() -> Self {
        Self {
            offset_y: ui::Tween::new(0.)
                .duration(Duration::from_secs(1))
                .curve(ui::curves::f32::ease_out_elastic),
            mx: ui::Damp::new(0.).speed(10.),
            my: ui::Damp::new(0.).speed(10.),
            keyframes: ui::Keyframes::new(0.0)
                .default_curve(ui::curves::f32::ease_in_out_quad)
                .repeat(ui::Repeat::Once),
        }
    }

    // Convenience: build a fun little sequence
    fn configure_keyframes_once(&mut self) {
        self.keyframes = ui::Keyframes::new(self.keyframes.value())
            .default_curve(ui::curves::f32::ease_in_out_quad)
            .repeat(ui::Repeat::Once)
            // Up quickly
            .tween(Duration::from_millis(220), -120.0)
            // Hold for a beat (will hold previous value, then snap to -120 at end;
            // if you prefer "snap then hold", flip the Hold logic in Keyframes)
            .hold(Duration::from_millis(120), -120.0)
            // Drop down with bounce curve override
            .tween_with_curve(
                Duration::from_millis(520),
                80.0,
                ui::curves::f32::ease_out_bounce,
            )
            // Back to 0
            .tween(Duration::from_millis(260), 0.0);

        self.keyframes.play();
    }

    fn configure_keyframes_loop(&mut self) {
        self.keyframes = ui::Keyframes::new(self.keyframes.value())
            .default_curve(ui::curves::f32::smooth_step)
            .repeat(ui::Repeat::Loop)
            .tween(Duration::from_millis(350), -60.0)
            .tween(Duration::from_millis(350), 60.0)
            .tween(Duration::from_millis(350), 0.0);

        self.keyframes.play();
    }

    fn configure_keyframes_pingpong_6(&mut self) {
        self.keyframes = ui::Keyframes::new(self.keyframes.value())
            .default_curve(ui::curves::f32::ease_in_out_sine)
            .repeat(ui::Repeat::PingPongNCycles(6))
            .tween(Duration::from_millis(300), -90.0)
            .tween(Duration::from_millis(300), 90.0)
            .tween(Duration::from_millis(300), 0.0);

        self.keyframes.play();
    }
}

impl Window<AnimationsApplication, ()> for MainWindow {
    fn build(&mut self, _: &mut AnimationsApplication, ctx: &mut ui::BuildContext) {
        self.mx.approach(ctx.input.mouse_x / ctx.view.scale_factor);
        self.my.approach(ctx.input.mouse_y / ctx.view.scale_factor);

        ctx.step_animation(&mut self.offset_y);
        ctx.step_animation(&mut self.keyframes); // NEW
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
                            ui::text(&format!("Tween Offset: {}", self.offset_y.value()))
                                .build(ctx);
                            ui::text(&format!("Tween Status: {:?}", self.offset_y.status()))
                                .build(ctx);

                            ui::text(&format!("Keyframes Offset: {}", self.keyframes.value()))
                                .build(ctx);
                            ui::text(&format!("Keyframes Status: {:?}", self.keyframes.status()))
                                .build(ctx);

                            ui::text(&format!("MX Status: {:?}", self.mx.status())).build(ctx);
                            ui::text(&format!("MY Status: {:?}", self.my.status())).build(ctx);

                            if clew_widgets::button("Move Up (Tween)").build(ctx).clicked() {
                                self.offset_y.tween_to(-100.);
                            }

                            if clew_widgets::button("Move Down (Tween)")
                                .build(ctx)
                                .clicked()
                            {
                                self.offset_y.tween_to(100.);
                            }

                            if clew_widgets::button("Go Home (Tween)").build(ctx).clicked() {
                                self.offset_y.tween_to(0.);
                            }

                            if clew_widgets::button("Play Keyframes (Once)")
                                .build(ctx)
                                .clicked()
                            {
                                self.configure_keyframes_once();
                            }

                            if clew_widgets::button("Loop Keyframes").build(ctx).clicked() {
                                self.configure_keyframes_loop();
                            }

                            if clew_widgets::button("PingPong Keyframes x6")
                                .build(ctx)
                                .clicked()
                            {
                                self.configure_keyframes_pingpong_6();
                            }

                            if clew_widgets::button("Stop Keyframes (Set 0)")
                                .build(ctx)
                                .clicked()
                            {
                                self.keyframes.set(0.0);
                            }
                        });
                });

            if self.keyframes.status() == ui::AnimationStatus::Started {
                println!("Started keyframes");
            }

            if self.keyframes.status() == ui::AnimationStatus::Ended {
                println!("Ended keyframes");
            }

            if self.offset_y.status() == ui::AnimationStatus::Started {
                println!("Started offsets");
            }

            if self.offset_y.status() == ui::AnimationStatus::Ended {
                println!("Ended offsets");
            }

            ui::decorated_box()
                .shape(ui::BoxShape::Oval)
                .color(ui::ColorRgba::from_hex(0xFFFF0000))
                .width(32.)
                .height(32.)
                .offset(self.mx.value() - 16., self.my.value() - 16.)
                .build(ctx);

            ui::decorated_box()
                .shape(ui::BoxShape::Rect)
                .color(ui::ColorRgba::from_hex(0xFF3B82F6))
                .width(80.)
                .height(80.)
                .offset(40., 140. + self.keyframes.value())
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
