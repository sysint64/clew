use pollster::FutureExt;
use tech_paws_ui::{self as ui, state::WidgetState};
use tech_paws_ui_derive::WidgetState;
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
            MainWindow {
                counter: CounterWidget { counter: 0 },
            },
            WindowDescriptor {
                title: "Counter".to_string(),
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

pub struct MainWindow {
    counter: CounterWidget,
}

impl Window<CounterApplication, ()> for MainWindow {
    fn build(&mut self, _: &mut CounterApplication, ctx: &mut ui::BuildContext) {
        ui::vstack().build(ctx, |ctx| {
            ui::widget::<CounterWidget>().build(ctx);
            ui::widget().state(&mut self.counter).build(ctx);
        });

        // ui::zstack()
        //     .fill_max_size()
        //     .align_x(ui::AlignX::Center)
        //     .align_y(ui::AlignY::Center)
        //     .build(ctx, |ctx| {
        //         ui::vstack()
        //             .spacing(12.)
        //             .cross_axis_alignment(ui::CrossAxisAlignment::Center)
        //             .build(ctx, |ctx| {
        //                 ui::text(
        //                     &bumpalo::format!(in &ctx.phase_allocator, "Counter: {}", self.counter),
        //                 )
        //                 .build(ctx);

        //                 ui::hstack().build(ctx, |ctx| {
        //                     if clew_widgets::button("+").build(ctx).clicked() {
        //                         self.counter += 1;
        //                     }

        //                     if clew_widgets::button("-").build(ctx).clicked() {
        //                         self.counter -= 1;
        //                     }
        //                 });
        //             });
        //     });
    }
}

#[derive(Default, WidgetState)]
pub struct CounterWidget {
    counter: i32,
}

pub enum CounterEvent {
    Increment,
    Decrement,
}

impl ui::Widget for CounterWidget {
    type Event = CounterEvent;

    fn on_event(&mut self, event: &Self::Event) -> bool {
        match event {
            CounterEvent::Increment => self.counter += 1,
            CounterEvent::Decrement => self.counter -= 1,
        }

        true
    }

    fn build(&mut self, ctx: &mut ui::BuildContext) {
        ui::zstack()
            .fill_max_size()
            .align_x(ui::AlignX::Center)
            .align_y(ui::AlignY::Center)
            .build(ctx, |ctx| {
                ui::vstack()
                    .spacing(12.)
                    .cross_axis_alignment(ui::CrossAxisAlignment::Center)
                    .build(ctx, |ctx| {
                        ui::text(
                            &bumpalo::format!(in &ctx.phase_allocator, "Counter: {}", self.counter),
                        )
                        .build(ctx);

                        ui::hstack().build(ctx, |ctx| {
                            if clew_widgets::button("+").build(ctx).clicked() {
                                ctx.emit(CounterEvent::Increment);
                            }

                            if clew_widgets::button("-").build(ctx).clicked() {
                                ctx.emit(CounterEvent::Decrement);
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
