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
            )
            .add_sequence(
                TestShortcuts::Chord1,
                &[
                    ui::KeyBinding::new(ui::keyboard::KeyCode::KeyK),
                    ui::KeyBinding::new(ui::keyboard::KeyCode::KeyC),
                ],
            );

        shortcuts_registry.scope(TestScopes::S2).add(
            TestShortcuts::Bind1,
            ui::KeyBinding::new(ui::keyboard::KeyCode::KeyA),
        );
        shortcuts_registry.scope(TestScopes::S3).add(
            TestShortcuts::Bind1,
            ui::KeyBinding::new(ui::keyboard::KeyCode::KeyA),
        );

        // Test 2: Unique shortcut on non-leaf parent (S4)
        shortcuts_registry.scope(TestScopes::S4).add(
            TestShortcuts::Bind1,
            ui::KeyBinding::new(ui::keyboard::KeyCode::KeyB),
        );
        // S5 is non-leaf, no shortcut registered
        shortcuts_registry
            .scope(TestScopes::S6)
            .add(
                TestShortcuts::Bind1,
                ui::KeyBinding::new(ui::keyboard::KeyCode::KeyC),
            )
            .add(
                TestShortcuts::Bind2,
                ui::KeyBinding::new(ui::keyboard::KeyCode::KeyE),
            );

        shortcuts_registry.scope(TestScopes::S7).add(
            TestShortcuts::Bind1,
            ui::KeyBinding::new(ui::keyboard::KeyCode::KeyD),
        );

        // Test 3: Multi-level fallthrough
        shortcuts_registry.scope(TestScopes::S5).add(
            TestShortcuts::Bind1,
            ui::KeyBinding::new(ui::keyboard::KeyCode::KeyE),
        );

        // Test 4: Root fallback
        shortcuts_registry.scope(TestScopes::S8).add(
            TestShortcuts::Bind1,
            ui::KeyBinding::new(ui::keyboard::KeyCode::KeyF),
        );

        // Test 5: Global fallback on root
        shortcuts_registry.scope(SHORTCUTS_ROOT_SCOPE_ID).add(
            TestShortcuts::Bind1,
            ui::KeyBinding::new(ui::keyboard::KeyCode::KeyZ),
        );

        //

        shortcuts_registry
            .scope(ui::ShortcutScopes::TextEditing)
            .add_repeat(
                ui::TextEditingShortcut::Delete,
                ui::KeyBinding::new(ui::keyboard::KeyCode::Delete),
            )
            .add_repeat(
                ui::TextEditingShortcut::Backspace,
                ui::KeyBinding::new(ui::keyboard::KeyCode::Backspace),
            )
            .add_repeat(
                ui::TextEditingShortcut::MoveNext,
                ui::KeyBinding::new(ui::keyboard::KeyCode::ArrowRight),
            )
            .add_repeat(
                ui::TextEditingShortcut::MovePrev,
                ui::KeyBinding::new(ui::keyboard::KeyCode::ArrowLeft),
            )
            .add_repeat(
                ui::TextEditingShortcut::MoveUp,
                ui::KeyBinding::new(ui::keyboard::KeyCode::ArrowUp),
            )
            .add_repeat(
                ui::TextEditingShortcut::MoveDown,
                ui::KeyBinding::new(ui::keyboard::KeyCode::ArrowDown),
            )
            .add_repeat(
                ui::TextEditingShortcut::NextLine,
                ui::KeyBinding::new(ui::keyboard::KeyCode::Enter),
            )
            .add_repeat(
                ui::TextEditingShortcut::MoveStart,
                ui::KeyBinding::new(ui::keyboard::KeyCode::Home),
            )
            .add(
                ui::TextEditingShortcut::MoveEnd,
                ui::KeyBinding::new(ui::keyboard::KeyCode::End),
            )
            .add_repeat(
                ui::TextEditingShortcut::BufferStart,
                ui::KeyBinding::new(ui::keyboard::KeyCode::Home).with_super(),
            )
            .add(
                ui::TextEditingShortcut::BufferEnd,
                ui::KeyBinding::new(ui::keyboard::KeyCode::End).with_super(),
            )
            .add_repeat(
                ui::TextEditingShortcut::PageUp,
                ui::KeyBinding::new(ui::keyboard::KeyCode::PageUp),
            )
            .add_repeat(
                ui::TextEditingShortcut::PageDown,
                ui::KeyBinding::new(ui::keyboard::KeyCode::PageDown),
            )
            .add(
                ui::TextEditingShortcut::SelectAll,
                ui::KeyBinding::new(ui::keyboard::KeyCode::KeyA).with_super(),
            )
            .add_modifier(
                ui::TextInputModifier::Select,
                ui::keyboard::KeyModifiers::shift(),
            )
            .add_modifier(
                ui::TextInputModifier::Word,
                ui::keyboard::KeyModifiers::super_key(),
            )
            .add_modifier(
                ui::TextInputModifier::Paragraph,
                ui::keyboard::KeyModifiers::super_key(),
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
    Chord1,
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
                if ctx.is_shortcut(TestShortcuts::Chord1) {
                    println!("Chord K+C triggered");
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

        ui::zstack()
            .align_x(ui::AlignX::Center)
            .align_y(ui::AlignY::Center)
            .fill_max_size()
            .build(ctx, |ctx| {
                ui::vstack().spacing(10.).build(ctx, |ctx| {
                    ui::text("Text Editing Shortcuts Test").build(ctx);

                    ui::shortcut_scope(ui::ShortcutScopes::TextEditing).build(ctx, |ctx| {
                        // Navigation shortcuts
                        if ctx.is_shortcut(ui::TextEditingShortcut::MoveNext) {
                            println!("MoveNext (ArrowRight)");
                        }
                        if ctx.is_shortcut(ui::TextEditingShortcut::MovePrev) {
                            println!("MovePrev (ArrowLeft)");
                        }
                        if ctx.is_shortcut(ui::TextEditingShortcut::MoveUp) {
                            println!("MoveUp (ArrowUp)");
                        }
                        if ctx.is_shortcut(ui::TextEditingShortcut::MoveDown) {
                            println!("MoveDown (ArrowDown)");
                        }

                        // Line navigation
                        if ctx.is_shortcut(ui::TextEditingShortcut::MoveStart) {
                            println!("MoveStart (Home)");
                        }
                        if ctx.is_shortcut(ui::TextEditingShortcut::MoveEnd) {
                            println!("MoveEnd (End)");
                        }

                        // Buffer navigation
                        if ctx.is_shortcut(ui::TextEditingShortcut::BufferStart) {
                            println!("BufferStart (Super+Home)");
                        }
                        if ctx.is_shortcut(ui::TextEditingShortcut::BufferEnd) {
                            println!("BufferEnd (Super+End)");
                        }

                        // Page navigation
                        if ctx.is_shortcut(ui::TextEditingShortcut::PageUp) {
                            println!("PageUp");
                        }
                        if ctx.is_shortcut(ui::TextEditingShortcut::PageDown) {
                            println!("PageDown");
                        }

                        // Editing shortcuts
                        if ctx.is_shortcut(ui::TextEditingShortcut::Delete) {
                            println!("Delete");
                        }
                        if ctx.is_shortcut(ui::TextEditingShortcut::Backspace) {
                            println!("Backspace");
                        }
                        if ctx.is_shortcut(ui::TextEditingShortcut::NextLine) {
                            println!("NextLine (Enter)");
                        }

                        // Selection shortcut
                        if ctx.is_shortcut(ui::TextEditingShortcut::SelectAll) {
                            println!("SelectAll (Super+A)");
                        }

                        // Modifiers
                        if ctx.has_modifier(ui::TextInputModifier::Select) {
                            println!("Modifier: Select (Shift held)");
                        }
                        if ctx.has_modifier(ui::TextInputModifier::Word) {
                            println!("Modifier: Word (Super held)");
                        }
                        if ctx.has_modifier(ui::TextInputModifier::Paragraph) {
                            println!("Modifier: Paragraph (Super held)");
                        }

                        // Visual feedback for testing
                        ui::vstack().spacing(5.).build(ctx, |ctx| {
                            ui::text("Try these shortcuts:").build(ctx);
                            ui::text("- Arrow keys (repeatable)").build(ctx);
                            ui::text("- Home/End").build(ctx);
                            ui::text("- Super+Home/End").build(ctx);
                            ui::text("- PageUp/PageDown (repeatable)").build(ctx);
                            ui::text("- Delete/Backspace (repeatable)").build(ctx);
                            ui::text("- Enter (repeatable)").build(ctx);
                            ui::text("- Super+A (Select All)").build(ctx);
                            ui::text("- Hold Shift (Select modifier)").build(ctx);
                            ui::text("- Hold Super (Word/Paragraph modifier)").build(ctx);
                        });
                    });
                });
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

        // ui::gesture_detector()
        //     .clickable(true)
        //     .focusable(true)
        //     .build(ctx, |ctx| {
        //         let response = ctx.of::<ui::GestureDetectorResponse>().unwrap();

        //         ui::text();
        //         ui::editable_text(&mut self.task_name)
        //             .gesture_response(response.clone())
        //             .text_vertical_align(ui::AlignY::Center)
        //             .padding(ui::EdgeInsets::symmetric(8., 0.))
        //             .background(
        //                 ui::decoration()
        //                     .border_radius(ui::BorderRadius::all(3.))
        //                     .color(ui::ColorRgba::from_hex(0xFF000000))
        //                     .border(ui::Border::all(ui::BorderSide::new(
        //                         1.,
        //                         if response.is_focused() {
        //                             ui::ColorRgba::from_hex(0xFF357CCE)
        //                         } else {
        //                             ui::ColorRgba::from_hex(0xFF414141)
        //                         },
        //                     )))
        //                     .build(ctx),
        //             )
        //             .height(20.)
            });

        // ui::shortcut_scope(clew_widgets::ShortcutScopeButton).build(ctx, |ctx| {
        //     ui::zstack()
        //         .fill_max_size()
        //         .align_x(ui::AlignX::Center)
        //         .align_y(ui::AlignY::Center)
        //         .build(ctx, |ctx| {
        // ui::editable_text(&mut self.task_name)
        //     .text_vertical_align(ui::AlignY::Center)
        //     .padding(ui::EdgeInsets::symmetric(8., 0.))
        //     .height(20.)
        //     .build_with_frame(ctx, |ctx, interaction_state, frame| {
        //         frame.background(
        //             ui::decoration()
        //                 .border_radius(ui::BorderRadius::all(3.))
        //                 .color(ui::ColorRgba::from_hex(0xFF000000))
        //                 .border(ui::Border::all(ui::BorderSide::new(
        //                     1.,
        //                     if interaction_state.is_focused {
        //                         ui::ColorRgba::from_hex(0xFF357CCE)
        //                     } else {
        //                         ui::ColorRgba::from_hex(0xFF414141)
        //                     },
        //                 )))
        //                 .build(ctx),
        //         )
        //     });
        //         });
        // });
    }
}

fn test(ctx: &mut BuildContext) {
    // ui::editable_text(&mut self.task_name)
    //     .text_vertical_align(ui::AlignY::Center)
    //     .padding(ui::EdgeInsets::symmetric(8., 0.))
    //     .height(20.)
    // .build(ctx, |ctx, frame| {
    //     ui::hstack().build(ctx, |ctx| {
    //         let response = ctx.of::<GestureDetectorResponse>().unwrap();

    //         ui::text(self.prefix).build(ctx);

    //         ui::editable_text()
    //             .background(
    //                 decoration()
    //                     .border(if response.is_focused {
    //                         BLUE_BORDER
    //                     } else {
    //                         GRAY_BORDER
    //                     })
    //                     .build(ctx),
    //             )
    //             .build(ctx);
    //     });
    // });
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
