use std::{sync::Arc, time::Duration};

use clew_widgets::{HorizontalScrollBar, VerticalScrollBar, button};
use pollster::FutureExt;
use tech_paws_ui::assets::Assets;
use tech_paws_ui::identifiable::Identifiable;
use tech_paws_ui::state::WidgetState;
// use tech_paws_ui::widgets::decorated_box::decoration;
// use tech_paws_ui::widgets::gesture_detector::{GestureDetectorResponse, gesture_detector};
// use tech_paws_ui::widgets::hstack::hstack;
// use tech_paws_ui::widgets::scroll_area::{ScrollAreaResponse, scroll_area};
// use tech_paws_ui::widgets::svg::svg;
// use tech_paws_ui::widgets::text::text;
// use tech_paws_ui::widgets::widget::widget;
// use tech_paws_ui::widgets::zstack::zstack;
use tech_paws_ui::widgets::*;
use tech_paws_ui::{
    AlignX, AlignY, ColorRgb,
    render::Renderer,
    widgets::{
        builder::BuildContext,
        for_each::for_each,
        component::{Component, component},
        vstack::vstack,
    },
};
use tech_paws_ui::{
    Border, BorderRadius, BorderSide, BoxShape, ColorRgba, CrossAxisAlignment, EdgeInsets,
    LinearGradient, MainAxisAlignment, RadialGradient, ScrollDirection, TextAlign,
};
use tech_paws_ui_derive::{Identifiable, WidgetState};
use tech_paws_ui_desktop::{
    app::{Application, ApplicationDelegate},
    window::Window,
    window_manager::{WindowDescriptor, WindowManager},
};
use tech_paws_ui_vello::VelloRenderer;

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
    fn init_assets(&mut self, assets: &mut Assets) {
        assets.load_font("Inter", include_bytes!("../../assets/fonts/Inter.ttf"));
        assets.load_font(
            "Source Han Serif",
            include_bytes!("../../assets/fonts/SourceHanSerif-Regular.otf"),
        );
        assets.load_font(
            "Noto Emoji",
            include_bytes!("../../assets/fonts/NotoEmoji-VariableFont_wght.ttf"),
        );

        log::info!("Loaded fonts");

        assets.load_svg("e", include_bytes!("../../assets/svg/e.svg"));

        log::info!("Loaded svgs");
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

        window_manager.spawn_window(
            MainWindow::new(),
            WindowDescriptor {
                title: "TODO List".to_string(),
                width: 400,
                height: 300,
                resizable: true,
                fill_color: ColorRgb::from_hex(0x121212),
            },
        );
    }

    fn create_renderer(window: Arc<winit::window::Window>) -> Box<dyn Renderer> {
        Box::new(
            VelloRenderer::new(
                window.clone(),
                window.inner_size().width,
                window.inner_size().height,
            )
            .block_on(),
        )

        // Box::new(TinySkiaRenderer::new(window.clone(), window.clone()))
    }
}

pub struct MainWindow {
    counter: Counter,
}

impl MainWindow {
    fn new() -> Self {
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
        scrollable_demo(self, app, ctx);
    }
}

fn _long_list(ctx: &mut BuildContext) {
    zstack()
        .fill_max_size()
        .margin(EdgeInsets::all(16.))
        .build(ctx, |ctx| {
            let response = scroll_area()
                .fill_max_size()
                .background(
                    decoration()
                        .color(ColorRgba::from_hex(0xFFFF0000).with_opacity(0.2))
                        .border_radius(BorderRadius::all(16.))
                        .build(ctx),
                )
                .scroll_direction(ScrollDirection::Vertical)
                .build(ctx, |ctx| {
                    vstack().build(ctx, |ctx| {
                        for_each(0..10_000).build(ctx, |ctx, index| {
                            text(&bumpalo::format!(in &ctx.phase_allocator, "Item {}", index))
                                .text_vertical_align(AlignY::Center)
                                .padding(EdgeInsets::symmetric(16., 0.))
                                .height(32.)
                                .fill_max_width()
                                .build(ctx);
                        });
                    });
                });

            if response.overflow_y {
                ctx.provide(response.clone(), |ctx| {
                    widget::<VerticalScrollBar>().build(ctx);
                });
            }

            if response.overflow_x {
                ctx.provide(response.clone(), |ctx| {
                    widget::<HorizontalScrollBar>().build(ctx);
                });
            }
        });
}

fn scrollable_demo(window: &mut MainWindow, app: &mut DemoApplication, ctx: &mut BuildContext) {
    zstack()
        .fill_max_size()
        // .clip_shape(Some(ClipShape::Oval))
        // .clip(Clip::RoundedRect {
        //     border_radius: BorderRadius::all(16.),
        // })
        .margin(EdgeInsets::all(16.))
        .build(ctx, |ctx| {
            // decorated_box()
            //     .color(ColorRgba::from_hex(0xFFFF0000).with_opacity(0.2))
            //     .fill_max_size()
            //     .clip(Clip::RoundedRect {
            //         border_radius: BorderRadius::all(16.),
            //     })
            //     .build(ctx);

            let response = scroll_area()
                .scroll_direction(ScrollDirection::Both)
                .fill_max_size()
                .background(
                    decoration()
                        .color(ColorRgba::from_hex(0xFFFF0000).with_opacity(0.2))
                        .border_radius(BorderRadius::all(16.))
                        .build(ctx),
                )
                .build(ctx, |ctx| {
                    let _response = ctx.of::<ScrollAreaResponse>().unwrap();

                    hstack()
                        // .padding(if response.overflow_y {
                        //     EdgeInsets::new().right(16.)
                        // } else {
                        //     EdgeInsets::ZERO
                        // })
                        // .fill_max_width()
                        .build(ctx, |ctx| {
                            component::<Counter>(app).build(ctx);
                            component(app).state(&mut window.counter).build(ctx);
                        });
                });

            if response.overflow_y {
                ctx.provide(response.clone(), |ctx| {
                    widget::<VerticalScrollBar>().build(ctx);
                });
            }

            if response.overflow_x {
                ctx.provide(response.clone(), |ctx| {
                    widget::<HorizontalScrollBar>().build(ctx);
                });
            }
        });
}

#[derive(WidgetState)]
struct Counter {
    books: Vec<Book>,
}

impl Default for Counter {
    fn default() -> Self {
        Self {
            books: vec![
                Book {
                    id: 12,
                    key: 21,
                    title: "Auto Book 1".to_string(),
                },
                Book {
                    id: 113,
                    key: 311,
                    title: "Auto Book 2".to_string(),
                },
                Book {
                    id: 114,
                    key: 411,
                    title: "Auto Book 3".to_string(),
                },
            ],
        }
    }
}

#[derive(Identifiable)]
#[allow(dead_code)]
struct Book {
    id: u64,
    #[id]
    key: u64,
    title: String,
}

enum CounterComponentEvent {
    HelloWorld,
}

impl Component for Counter {
    type App = DemoApplication;
    type Event = CounterComponentEvent;

    fn on_event(&mut self, _: &mut DemoApplication, _: &CounterComponentEvent) -> bool {
        println!("Hello World!");

        true
    }

    fn build(&mut self, app: &mut DemoApplication, ctx: &mut BuildContext) {
        vstack()
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .fill_max_size()
            .padding(EdgeInsets::all(12.))
            .build(ctx, |ctx| {
                hstack().cross_axis_alignment(CrossAxisAlignment::Center).build(ctx, |ctx| {
                    svg("e").build(ctx);
                    svg("e").padding(EdgeInsets::all(8.)).build(ctx);
                    svg("e").padding(EdgeInsets::all(8.)).size(128.).build(ctx);
                    svg("e").size(128.).build(ctx);
                    svg("e").color(ColorRgba::from_hex(0xFFFFFFFF)).build(ctx);
                });

                for_each(0..2).build(ctx, |ctx, i| {
                    for_each(&mut self.books).build(ctx, |ctx, book| {
                        if button(&format!("{i}: {}", book.title)).build(ctx).clicked() {
                            book.title = "Changed!".to_string();
                            println!("Clicked to book with id id: {}", book.id());
                        }
                    });
                });

                button("Button").build(ctx);
                // gap().fill_max_height().build(ctx);
                button("Button").build(ctx);

                // colored_box(ColorRgba::from_hex(0xFFCC0000)).build(ctx, |ctx| {
                hstack()
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .width(400.)
                    .padding(EdgeInsets::all(8.))
                    .build(ctx, |ctx| {
                        if gesture_detector()
                            .clickable(true)
                            .build(ctx, |ctx| {
                                let response = ctx.of::<GestureDetectorResponse>().unwrap();

                                vstack()
                                    .fill_max_height()
                                    .fill_max_width()
                                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                                    .background(
                                        decoration()
                                            .color(ColorRgba::from_hex(0xFFCC0000))
                                            .shape(if response.is_hot() {
                                                BoxShape::Oval
                                            } else {
                                                BoxShape::Rect
                                            })
                                            .border_radius(BorderRadius::all(8.))
                                            .border(Border::all(BorderSide::new(
                                                1.,
                                                if response.is_focused() {
                                                    ColorRgba::from_hex(0xFF00DD00)
                                                } else {
                                                    ColorRgba::from_hex(0xFF007700)
                                                },
                                            )))
                                            .add_linear_gradient(LinearGradient::vertical(vec![
                                                ColorRgba::from_hex(0xFF2F2F2F),
                                                ColorRgba::from_hex(0xFF272727),
                                            ]))
                                            .add_radial_gradient(RadialGradient::circle(vec![
                                                if response.is_active() {
                                                    ColorRgba::from_hex(0xFFFF0000)
                                                } else {
                                                    ColorRgba::from_hex(0x00000000)
                                                },
                                                if response.is_active() {
                                                    ColorRgba::from_hex(0x00000000)
                                                } else {
                                                    ColorRgba::from_hex(0xFFFF0000)
                                                },
                                            ])).build(ctx)
                                    )
                                    .build(ctx, |ctx| {
                                        vstack()
                                            .fill_max_height()
                                            .cross_axis_alignment(
                                                CrossAxisAlignment::Stretch,
                                            )
                                            .build(ctx, |ctx| {
                                                text("Counter:")
                                                    .text_align_x(AlignX::Center)
                                                    .text_vertical_align(AlignY::Center)
                                                    .build(ctx);

                                                text(&format!("{}", app.counter))
                                                    .text_align_x(AlignX::Center)
                                                    .text_vertical_align(AlignY::Center)
                                                    .build(ctx);
                                            });
                                        text("TEST BUTTON")
                                            .text_align_x(AlignX::Center)
                                            .text_vertical_align(AlignY::Center)
                                            .build(ctx);
                                    });
                            })
                            .clicked()
                        {
                            println!("Clicked!");
                        }

                        button("Hello World!").build(ctx);
                    });

                text("Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.")
                    .color(ColorRgba::from_hex(0xFFFFFFFF))
                    .text_align(TextAlign::Justified)
                    .fill_max_width()
                    .build(ctx);

                text("شكمنتشعيتد منتشسيب كمنتشسيب منت")
                    .color(ColorRgba::from_hex(0xFFFFFFFF))
                    .text_align(TextAlign::Right)
                    .fill_max_width()
                    .build(ctx);

                text("日本語はすごいです！")
                    .color(ColorRgba::from_hex(0xFFFFFFFF))
                    .text_align(TextAlign::Center)
                    .fill_max_width()
                    .build(ctx);

                text("شكمنتشعيتد منتشسيب كمنتشسيب منت")
                    .color(ColorRgba::from_hex(0xFFFFFFFF))
                    .text_align(TextAlign::End)
                    .fill_max_width()
                    .build(ctx);

                if button(&format!("Counter:\nValue: {}", app.counter))
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

#[allow(dead_code)]
#[profiling::function]
fn ui_benchmark(ctx: &mut BuildContext) {
    vstack().fill_max_width().build(ctx, |ctx| {
        // gap().height(128.).show(ctx);

        for i in 0..500 {
            hstack().fill_max_width().build(ctx, |ctx| {
                for j in 0..30 {
                    // hstack().show(ctx, |ctx| {});
                    if button(&bumpalo::format!(in &ctx.phase_allocator, "Button {}_{}", i, j))
                        .id((i, j))
                        .build(ctx)
                        .clicked()
                    {
                        // if button("Button")
                        // if button_id("Button", (i, j)).show(ctx) {
                        println!("Button {i}_{j} Clicked");
                        // 1000 total buttons
                    }
                }
            });
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let server_addr = format!("0.0.0.0:{}", puffin_http::DEFAULT_PORT);
    // let _puffin_server = puffin_http::Server::new(&server_addr).unwrap();
    // eprintln!("Serving demo profile data on {server_addr}. Run `puffin_viewer` to view it.");
    // puffin::set_scopes_on(true);
    tracy_client::Client::start();

    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    log::info!("Starting app");
    Application::run_application(DemoApplication::new())?;

    Ok(())
}
