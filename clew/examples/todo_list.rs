use clew::prelude::*;
use clew::{self as ui, SHORTCUTS_ROOT_SCOPE_ID};
use clew_derive::{ShortcutId, ShortcutScopeId};
use clew_desktop::{
    app::{Application, ApplicationDelegate},
    window::Window,
    window_manager::{WindowDescriptor, WindowManager},
};
use clew_vello::VelloRenderer;
use pollster::FutureExt;

struct TodoApplication;

impl ApplicationDelegate<()> for TodoApplication {
    fn on_start(
        &mut self,
        window_manager: &mut WindowManager<Self, ()>,
        shortcuts_registry: &mut ui::ShortcutsRegistry,
    ) where
        Self: std::marker::Sized,
    {
        // Test 1: Child shadows parent
        shortcuts_registry
            .scope(TestScopes::S1)
            .add(
                TestShortcuts::Bind1,
                ui::KeyBinding::new(ui::keyboard::KeyCode::KeyA),
            )
            .add(
                TestShortcuts::Bind2,
                ui::KeyBinding::new(ui::keyboard::KeyCode::KeyG),
            );

        shortcuts_registry.scope(TestScopes::S2).add(
            TestShortcuts::Bind1,
            ui::KeyBinding::new(ui::keyboard::KeyCode::KeyA), // Same key - should shadow S1
        );
        shortcuts_registry.scope(TestScopes::S3).add(
            TestShortcuts::Bind1,
            ui::KeyBinding::new(ui::keyboard::KeyCode::KeyA), // Same key - should shadow S1
        );

        // Test 2: Unique shortcut on non-leaf parent (S4)
        shortcuts_registry.scope(TestScopes::S4).add(
            TestShortcuts::Bind1,
            ui::KeyBinding::new(ui::keyboard::KeyCode::KeyB), // Unique to S4
        );
        // S5 is non-leaf, no shortcut registered
        shortcuts_registry
            .scope(TestScopes::S6)
            .add(
                TestShortcuts::Bind1,
                ui::KeyBinding::new(ui::keyboard::KeyCode::KeyC), // Unique to S6
            )
            .add(
                TestShortcuts::Bind2,
                ui::KeyBinding::new(ui::keyboard::KeyCode::KeyE), // Should shadow S5's KeyE
            );

        shortcuts_registry.scope(TestScopes::S7).add(
            TestShortcuts::Bind1,
            ui::KeyBinding::new(ui::keyboard::KeyCode::KeyD), // Unique to S7
        );

        // Test 3: Multi-level fallthrough
        shortcuts_registry.scope(TestScopes::S5).add(
            TestShortcuts::Bind1,
            ui::KeyBinding::new(ui::keyboard::KeyCode::KeyE), // S5 (non-leaf) gets its own
        );

        // Test 4: Root fallback
        shortcuts_registry.scope(TestScopes::S8).add(
            TestShortcuts::Bind1,
            ui::KeyBinding::new(ui::keyboard::KeyCode::KeyF),
        );

        // Test 5: Global fallback on root
        shortcuts_registry.scope(SHORTCUTS_ROOT_SCOPE_ID).add(
            TestShortcuts::Bind1,
            ui::KeyBinding::new(ui::keyboard::KeyCode::KeyZ), // Should work everywhere
        );

        window_manager.spawn_window(
            MainWindow {
                task_name: ui::TextData::from("Test"),
            },
            WindowDescriptor {
                title: "Todo List".to_string(),
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
    task_name: ui::TextData,
}

#[derive(ShortcutId)]
pub enum TestShortcuts {
    Bind1,
    Bind2,
}

#[derive(ShortcutScopeId)]
pub enum TestScopes {
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
}

impl Window<TodoApplication, ()> for MainWindow {
    fn build(&mut self, _: &mut TodoApplication, ctx: &mut ui::BuildContext) {
        ui::vstack().build(ctx, |ctx| {
            // Test 1: Simple siblings
            ui::shortcut_scope(TestScopes::S1).build(ctx, |ctx| {
                if ctx.is_shortcut(TestShortcuts::Bind1) {
                    println!("S1 / BIND1 (KeyA)");
                }
                if ctx.is_shortcut(TestShortcuts::Bind2) {
                    println!("S1 / BIND2 (KeyG)");
                }

                ui::shortcut_scope(TestScopes::S2).build(ctx, |ctx| {
                    if ctx.is_shortcut(TestShortcuts::Bind1) {
                        println!("S2 / BIND1 (KeyA) - should shadow S1");
                    }
                    if ctx.is_shortcut(TestShortcuts::Bind2) {
                        println!("S2 / BIND2 (KeyG) - should fallthrough to S1");
                    }
                });

                ui::shortcut_scope(TestScopes::S3).build(ctx, |ctx| {
                    if ctx.is_shortcut(TestShortcuts::Bind1) {
                        println!("S3 / BIND1 (KeyA) - should shadow S1");
                    }
                    if ctx.is_shortcut(TestShortcuts::Bind2) {
                        println!("S3 / BIND2 (KeyG) - should fallthrough to S1");
                    }
                });
            });

            // Test 2: Mix of leaf and non-leaf siblings
            ui::shortcut_scope(TestScopes::S4).build(ctx, |ctx| {
                if ctx.is_shortcut(TestShortcuts::Bind1) {
                    println!("S4 / BIND1 (KeyB)");
                }
                if ctx.is_shortcut(TestShortcuts::Bind2) {
                    println!("S4 / BIND2 - should be false (no KeyE here)");
                }

                ui::shortcut_scope(TestScopes::S5).build(ctx, |ctx| {
                    if ctx.is_shortcut(TestShortcuts::Bind1) {
                        println!("S5 / BIND1 (KeyE)");
                    }
                    if ctx.is_shortcut(TestShortcuts::Bind2) {
                        println!("S5 / BIND2 - should be false (KeyE is BIND1, not BIND2)");
                    }

                    ui::shortcut_scope(TestScopes::S6).build(ctx, |ctx| {
                        if ctx.is_shortcut(TestShortcuts::Bind1) {
                            println!("S6 / BIND1 (KeyC) - should shadow S5's KeyE");
                        }
                        if ctx.is_shortcut(TestShortcuts::Bind2) {
                            println!("S6 / BIND2 (KeyE) - shadows S5's BIND1");
                        }
                    });
                });

                ui::shortcut_scope(TestScopes::S7).build(ctx, |ctx| {
                    if ctx.is_shortcut(TestShortcuts::Bind1) {
                        println!("S7 / BIND1 (KeyD)");
                    }
                    if ctx.is_shortcut(TestShortcuts::Bind2) {
                        println!("S7 / BIND2 - should be false (no KeyE here)");
                    }
                });
            });

            // Test 3: Single leaf
            ui::shortcut_scope(TestScopes::S8).build(ctx, |ctx| {
                if ctx.is_shortcut(TestShortcuts::Bind1) {
                    println!("S8 / BIND1 (KeyF)");
                }
                if ctx.is_shortcut(TestShortcuts::Bind2) {
                    println!("S8 / BIND2 - should be false");
                }
            });

            // Test global fallback at root level
            if ctx.is_shortcut(TestShortcuts::Bind1) {
                println!("ROOT / BIND1 (KeyZ) - global fallback");
            }
        });

        // ui::zstack()
        //     .fill_max_size()
        //     .align_x(ui::AlignX::Center)
        //     .align_y(ui::AlignY::Center)
        //     .build(ctx, |ctx| {
        //         ui::vstack().build(ctx, |ctx| {
        //             ui::shortcut_scope(TestScopes::S1).build(ctx, |ctx| {
        //                 ui::shortcut_scope(TestScopes::S2).build(ctx, |ctx| {
        //                     ui::shortcut_scope(TestScopes::S3).build(ctx, |ctx| {
        //                         clew_widgets::button("A").build(ctx);
        //                     });

        //                     ui::shortcut_scope(TestScopes::S4).build(ctx, |ctx| {
        //                         clew_widgets::button("B").build(ctx);
        //                     });
        //                 });

        //                 ui::shortcut_scope(TestScopes::S4).build(ctx, |ctx| {
        //                     clew_widgets::button("C").build(ctx);
        //                 });
        //             });
        //         });
        //     });

        // ui::shortcut_scope(clew_widgets::ShortcutScopeButton).build(ctx, |ctx| {
        //     ui::zstack()
        //         .fill_max_size()
        //         .align_x(ui::AlignX::Center)
        //         .align_y(ui::AlignY::Center)
        //         .build(ctx, |ctx| {
        //             ui::editable_text(&mut self.task_name)
        //                 .text_vertical_align(ui::AlignY::Center)
        //                 .padding(ui::EdgeInsets::symmetric(8., 0.))
        //                 .height(20.)
        //                 .build_with_frame(ctx, |ctx, interaction_state, frame| {
        //                     frame.background(
        //                         ui::decoration()
        //                             .border_radius(ui::BorderRadius::all(3.))
        //                             .color(ui::ColorRgba::from_hex(0xFF000000))
        //                             .border(ui::Border::all(ui::BorderSide::new(
        //                                 1.,
        //                                 if interaction_state.is_focused {
        //                                     ui::ColorRgba::from_hex(0xFF357CCE)
        //                                 } else {
        //                                     ui::ColorRgba::from_hex(0xFF414141)
        //                                 },
        //                             )))
        //                             .build(ctx),
        //                     )
        //                 });
        //         });
        // });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracy_client::Client::start();

    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    log::info!("Starting app");
    Application::run_application(TodoApplication)?;

    Ok(())
}
