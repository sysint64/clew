use clew_widgets::{HorizontalScrollBar, VerticalScrollBar};
use pollster::FutureExt;
use tech_paws_ui::{
    AlignY, Axis, BorderRadius, ColorRgb, ColorRgba, EdgeInsets,
    render::Renderer,
    widgets::{
        builder::BuildContext, decorated_box::decoration, text::text, virtual_list::virtual_list,
        vstack::vstack, widget::widget, zstack::zstack,
    },
};
use tech_paws_ui_desktop::{
    app::{Application, ApplicationDelegate},
    window::Window,
    window_manager::{WindowDescriptor, WindowManager},
};
use tech_paws_ui_vello::VelloRenderer;

struct DemoApplication;

impl ApplicationDelegate<()> for DemoApplication {
    fn on_start(&mut self, window_manager: &mut WindowManager<Self, ()>)
    where
        Self: std::marker::Sized,
    {
        window_manager.spawn_window(
            MainWindow,
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

pub struct MainWindow;

impl Window<DemoApplication, ()> for MainWindow {
    fn build(&mut self, _: &mut DemoApplication, ctx: &mut BuildContext) {
        vstack()
            .fill_max_size()
            .padding(EdgeInsets::symmetric(0., 8.))
            .build(ctx, |ctx| {
                zstack()
                    .fill_max_size()
                    .margin(EdgeInsets::symmetric(16., 8.))
                    .build(ctx, |ctx| {
                        let response = virtual_list()
                            .fill_max_size()
                            .background(
                                decoration()
                                    .color(ColorRgba::from_hex(0xFFFF0000).with_opacity(0.2))
                                    .border_radius(BorderRadius::all(16.))
                                    .build(ctx),
                            )
                            .items_count(10_000_000_000)
                            .item_size(32.)
                            .build(ctx, |ctx, index| {
                                text(&bumpalo::format!(in &ctx.phase_allocator, "Item {}", index))
                                    .padding(EdgeInsets::symmetric(16., 0.))
                                    .height(32.)
                                    .fill_max_width()
                                    .build(ctx);
                            });

                        if response.overflow_y {
                            ctx.provide(response.clone(), |ctx| {
                                widget::<VerticalScrollBar>().build(ctx);
                            });
                        }
                    });

                zstack()
                    .fill_max_size()
                    .margin(EdgeInsets::symmetric(16., 8.))
                    .build(ctx, |ctx| {
                        let response = virtual_list()
                            .fill_max_size()
                            .scroll_direction(Axis::Horizontal)
                            .background(
                                decoration()
                                    .color(ColorRgba::from_hex(0xFFFF0000).with_opacity(0.2))
                                    .border_radius(BorderRadius::all(16.))
                                    .build(ctx),
                            )
                            .items_count(10_000_000_000)
                            .item_size(150.)
                            .build(ctx, |ctx, index| {
                                text(&bumpalo::format!(in &ctx.phase_allocator, "Item {}", index))
                                    .text_vertical_align(AlignY::Center)
                                    .padding(EdgeInsets::symmetric(16., 0.))
                                    .width(150.)
                                    .fill_max_height()
                                    .build(ctx);
                            });

                        if response.overflow_x {
                            ctx.provide(response.clone(), |ctx| {
                                widget::<HorizontalScrollBar>().build(ctx);
                            });
                        }
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
    Application::run_application(DemoApplication)?;

    Ok(())
}
