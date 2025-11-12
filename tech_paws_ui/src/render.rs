use std::collections::HashMap;

use glam::Vec2;

use crate::{
    Border, BorderRadius, BorderSide, ColorRgba, ColorStop, Gradient, LayoutDirection,
    LinearGradient, RadialGradient, Rect, View, WidgetType,
    interaction::{InteractionState, handle_interaction},
    io::UserInput,
    layout::{LayoutState, layout},
    state::UiState,
    text::{FontId, FontResources, StringId, StringInterner, TextId, TextsResources},
    widgets,
};

#[derive(Debug, Default)]
pub struct RenderState {
    pub(crate) commands: Vec<RenderCommand>,
}

impl RenderState {
    pub fn commands(&self) -> &[RenderCommand] {
        &self.commands
    }
}

pub trait Renderer {
    fn process_commands(
        &mut self,
        view: &View,
        state: &RenderState,
        fonts: &mut FontResources,
        text: &mut TextsResources,
    );
}

pub struct RenderContext<'a, 'b> {
    pub interaction: &'a InteractionState,
    pub input: &'a UserInput,
    pub view: &'a View,
    pub text: &'a mut TextsResources<'b>,
    pub fonts: &'a mut FontResources,
    pub string_interner: &'a mut StringInterner,
    pub strings: &'a mut HashMap<StringId, TextId>,
    pub layout_direction: LayoutDirection,
    commands: &'a mut Vec<RenderCommand>,
}

impl RenderContext<'_, '_> {
    pub fn push_command(&mut self, command: RenderCommand) {
        self.commands.push(command);
    }
}

#[derive(Debug)]
pub enum RenderCommand {
    Rect {
        boundary: Rect,
        fill: Fill,
        border_radius: BorderRadius,
        border: Border,
    },
    Oval {
        boundary: Rect,
        fill: Fill,
        border: Option<BorderSide>,
    },
    Text {
        x: f32,
        y: f32,
        text_id: TextId,
        tint_color: Option<ColorRgba>,
    },
    PushClipRect(Rect),
    PopClip,
}

#[derive(Debug)]
pub enum Fill {
    None,
    Color(ColorRgba),
    Gradient(Gradient),
}

pub trait PixelExtension<T> {
    fn px(self, ctx: &RenderContext) -> T;
}

impl PixelExtension<f32> for f32 {
    fn px(self, ctx: &RenderContext) -> f32 {
        self * ctx.view.scale_factor.ceil()
    }
}

impl PixelExtension<Vec2> for Vec2 {
    fn px(self, ctx: &RenderContext) -> Vec2 {
        Vec2::new(self.x.px(ctx), self.y.px(ctx))
    }
}

impl PixelExtension<Rect> for Rect {
    fn px(self, ctx: &RenderContext) -> Rect {
        self * ctx.view.scale_factor.ceil()
    }
}

pub fn cache_string<F>(ctx: &mut RenderContext, symbol: StringId, create_text_id: F) -> TextId
where
    F: FnOnce(&mut RenderContext) -> TextId,
{
    match ctx.strings.get(&symbol) {
        Some(text_id) => *text_id,
        None => {
            let text_id = create_text_id(ctx);
            ctx.strings.insert(symbol, text_id);

            text_id
        }
    }
}

pub fn render(
    state: &mut UiState,
    text: &mut TextsResources,
    fonts: &mut FontResources,
    string_interner: &mut StringInterner,
    strings: &mut HashMap<StringId, TextId>,
) {
    let start = std::time::Instant::now();

    layout(
        &mut state.layout_state,
        &state.view,
        &state.layout_commands,
        &mut state.widget_placements,
    );

    for placement in &state.widget_placements {
        let mut render_context = RenderContext {
            interaction: &state.interaction_state,
            input: &state.user_input,
            view: &state.view,
            text,
            fonts,
            string_interner,
            strings,
            layout_direction: state.layout_direction,
            commands: &mut state.render_state.commands,
        };

        if placement.widget_ref.widget_type == WidgetType::of::<widgets::button::ButtonWidget>() {
            widgets::button::render(
                &mut render_context,
                placement,
                state
                    .widgets_states
                    .get_mut::<widgets::button::State>(placement.widget_ref.id)
                    .unwrap(),
            );
        }
    }

    handle_interaction(
        &mut state.user_input,
        &mut state.interaction_state,
        &mut state.widgets_states,
        &state.view,
        text,
        fonts,
        &state.widget_placements,
    );

    state.widgets_states.sweep(&mut state.interaction_state);
    state.user_input.clear_frame_events();
}

pub fn create_test_commands() -> Vec<RenderCommand> {
    vec![
        // Test 1: Simple solid color rectangle
        RenderCommand::Rect {
            boundary: Rect::from_pos_size(Vec2::new(10.0, 10.0), Vec2::new(100., 100.)),
            fill: Fill::Color(ColorRgba::from_hex(0xFFFF0000)), // Red
            border_radius: BorderRadius::default(),
            border: Border::default(),
        },
        // // Test 2: Rectangle with rounded corners
        // RenderCommand::Rect {
        //     x: 120.0,
        //     y: 10.0,
        //     width: 100.0,
        //     height: 100.0,
        //     fill: Fill::Color(ColorRgba::from_hex(0xFF00FF00)), // Green
        //     border_radius: BorderRadius::all(20.0),
        //     border: Border::default(),
        // },
        // // Test 3: Rectangle with border
        // RenderCommand::Rect {
        //     x: 230.0,
        //     y: 10.0,
        //     width: 100.0,
        //     height: 100.0,
        //     fill: Fill::Color(ColorRgba::from_hex(0xFF0000FF)), // Blue
        //     border_radius: BorderRadius::default(),
        //     border: Border::all(BorderSide::new(5.0, ColorRgba::from_hex(0xFFFFFF00))), // Yellow border
        // },
        // // Test 4: Rounded rectangle with border
        // RenderCommand::Rect {
        //     x: 340.0,
        //     y: 10.0,
        //     width: 100.0,
        //     height: 100.0,
        //     fill: Fill::Color(ColorRgba::from_hex(0xFFFF00FF)), // Magenta
        //     border_radius: BorderRadius::all(15.0),
        //     border: Border::all(BorderSide::new(3.0, ColorRgba::from_hex(0xFF000000))), // Black border
        // },
        // // Test 5: Rectangle with different corner radii
        // RenderCommand::Rect {
        //     x: 450.0,
        //     y: 10.0,
        //     width: 100.0,
        //     height: 100.0,
        //     fill: Fill::Color(ColorRgba::from_hex(0xFF00FFFF)), // Cyan
        //     border_radius: BorderRadius::new(5.0, 15.0, 25.0, 35.0),
        //     border: Border::default(),
        // },
        // // Test 6: Simple circle (oval with equal dimensions)
        // RenderCommand::Oval {
        //     x: 10.0,
        //     y: 130.0,
        //     width: 100.0,
        //     height: 100.0,
        //     fill: Fill::Color(ColorRgba::from_hex(0xFFFF8800)), // Orange
        //     border: None,
        // },
        // // Test 7: Ellipse (oval with different dimensions)
        // RenderCommand::Oval {
        //     x: 120.0,
        //     y: 130.0,
        //     width: 150.0,
        //     height: 100.0,
        //     fill: Fill::Color(ColorRgba::from_hex(0xFF8800FF)), // Purple
        //     border: None,
        // },
        // // Test 8: Circle with border
        // RenderCommand::Oval {
        //     x: 280.0,
        //     y: 130.0,
        //     width: 100.0,
        //     height: 100.0,
        //     fill: Fill::Color(ColorRgba::from_hex(0xFFFFFFFF)), // White
        //     border: Some(BorderSide::new(4.0, ColorRgba::from_hex(0xFF000000))), // Black border
        // },
        // // Test 9: Linear gradient - horizontal
        // RenderCommand::Rect {
        //     x: 10.0,
        //     y: 250.0,
        //     width: 200.0,
        //     height: 100.0,
        //     fill: Fill::Gradient(Gradient::Linear(LinearGradient::horizontal(vec![
        //         ColorRgba::from_hex(0xFFFF0000), // Red
        //         ColorRgba::from_hex(0xFF0000FF), // Blue
        //     ]))),
        //     border_radius: BorderRadius::default(),
        //     border: Border::default(),
        // },
        // // Test 10: Linear gradient - vertical
        // RenderCommand::Rect {
        //     x: 220.0,
        //     y: 250.0,
        //     width: 100.0,
        //     height: 100.0,
        //     fill: Fill::Gradient(Gradient::Linear(LinearGradient::vertical(vec![
        //         ColorRgba::from_hex(0xFFFFFF00), // Yellow
        //         ColorRgba::from_hex(0xFFFF00FF), // Magenta
        //     ]))),
        //     border_radius: BorderRadius::default(),
        //     border: Border::default(),
        // },
        // // Test 11: Linear gradient - diagonal (45 degrees)
        // RenderCommand::Rect {
        //     x: 330.0,
        //     y: 250.0,
        //     width: 100.0,
        //     height: 100.0,
        //     fill: Fill::Gradient(Gradient::Linear(LinearGradient::angled(
        //         std::f32::consts::PI / 4.0,
        //         vec![
        //             ColorRgba::from_hex(0xFF00FF00), // Green
        //             ColorRgba::from_hex(0xFF00FFFF), // Cyan
        //         ],
        //     ))),
        //     border_radius: BorderRadius::all(10.0),
        //     border: Border::default(),
        // },
        // // Test 12: Multi-stop linear gradient
        // RenderCommand::Rect {
        //     x: 440.0,
        //     y: 250.0,
        //     width: 100.0,
        //     height: 100.0,
        //     fill: Fill::Gradient(Gradient::Linear(LinearGradient::new(
        //         (0.0, 0.5),
        //         (1.0, 0.5),
        //         vec![
        //             ColorStop::new(0.0, ColorRgba::from_hex(0xFFFF0000)), // Red
        //             ColorStop::new(0.5, ColorRgba::from_hex(0xFF00FF00)), // Green
        //             ColorStop::new(1.0, ColorRgba::from_hex(0xFF0000FF)), // Blue
        //         ],
        //     ))),
        //     border_radius: BorderRadius::default(),
        //     border: Border::default(),
        // },
        // // Test 13: Radial gradient
        // RenderCommand::Oval {
        //     x: 10.0,
        //     y: 370.0,
        //     width: 100.0,
        //     height: 100.0,
        //     fill: Fill::Gradient(Gradient::Radial(RadialGradient::circle(vec![
        //         ColorRgba::from_hex(0xFFFFFFFF), // White center
        //         ColorRgba::from_hex(0xFFFF0000), // Red edge
        //     ]))),
        //     border: None,
        // },
        // // Test 14: Radial gradient in rectangle
        // RenderCommand::Rect {
        //     x: 120.0,
        //     y: 370.0,
        //     width: 150.0,
        //     height: 100.0,
        //     fill: Fill::Gradient(Gradient::Radial(RadialGradient::circle(vec![
        //         ColorRgba::from_hex(0xFFFFFF00), // Yellow center
        //         ColorRgba::from_hex(0xFFFF00FF), // Magenta edge
        //         ColorRgba::from_hex(0xFF000000), // Black outer
        //     ]))),
        //     border_radius: BorderRadius::all(20.0),
        //     border: Border::default(),
        // },
        // // Test 15: Transparency test - semi-transparent rectangles
        // RenderCommand::Rect {
        //     x: 280.0,
        //     y: 370.0,
        //     width: 80.0,
        //     height: 80.0,
        //     fill: Fill::Color(ColorRgba::new(1.0, 0.0, 0.0, 1.0)), // Opaque red
        //     border_radius: BorderRadius::default(),
        //     border: Border::default(),
        // },
        // RenderCommand::Rect {
        //     x: 320.0,
        //     y: 410.0,
        //     width: 80.0,
        //     height: 80.0,
        //     fill: Fill::Color(ColorRgba::new(0.0, 0.0, 1.0, 0.5)), // Semi-transparent blue
        //     border_radius: BorderRadius::default(),
        //     border: Border::default(),
        // },
        // // Test 16: Different border configurations
        // RenderCommand::Rect {
        //     x: 410.0,
        //     y: 370.0,
        //     width: 100.0,
        //     height: 100.0,
        //     fill: Fill::Color(ColorRgba::from_hex(0xFFCCCCCC)), // Light gray
        //     border_radius: BorderRadius::default(),
        //     border: Border::new(
        //         Some(BorderSide::new(5.0, ColorRgba::from_hex(0xFFFF0000))), // Red top
        //         Some(BorderSide::new(5.0, ColorRgba::from_hex(0xFF00FF00))), // Green right
        //         Some(BorderSide::new(5.0, ColorRgba::from_hex(0xFF0000FF))), // Blue bottom
        //         Some(BorderSide::new(5.0, ColorRgba::from_hex(0xFFFFFF00))), // Yellow left
        //     ),
        // },
        // // Test 17: Clipping test - shapes inside clip rect
        // RenderCommand::PushClipRect {
        //     x: 10.0,
        //     y: 490.0,
        //     width: 150.0,
        //     height: 150.0,
        // },
        // RenderCommand::Rect {
        //     x: 0.0,
        //     y: 480.0,
        //     width: 200.0,
        //     height: 50.0,
        //     fill: Fill::Color(ColorRgba::from_hex(0xFFFF0000)), // This should be clipped
        //     border_radius: BorderRadius::default(),
        //     border: Border::default(),
        // },
        // RenderCommand::Oval {
        //     x: 50.0,
        //     y: 520.0,
        //     width: 100.0,
        //     height: 100.0,
        //     fill: Fill::Color(ColorRgba::from_hex(0xFF00FF00)), // Partially clipped
        //     border: Some(BorderSide::new(3.0, ColorRgba::from_hex(0xFF000000))),
        // },
        // RenderCommand::PopClip,
        // // Test 18: Very small shapes (edge case testing)
        // RenderCommand::Rect {
        //     x: 170.0,
        //     y: 490.0,
        //     width: 5.0,
        //     height: 5.0,
        //     fill: Fill::Color(ColorRgba::from_hex(0xFFFF0000)),
        //     border_radius: BorderRadius::default(),
        //     border: Border::default(),
        // },
        // RenderCommand::Oval {
        //     x: 180.0,
        //     y: 490.0,
        //     width: 5.0,
        //     height: 5.0,
        //     fill: Fill::Color(ColorRgba::from_hex(0xFF00FF00)),
        //     border: None,
        // },
        // // Test 19: Very large border radius (should be clamped)
        // RenderCommand::Rect {
        //     x: 190.0,
        //     y: 490.0,
        //     width: 80.0,
        //     height: 80.0,
        //     fill: Fill::Color(ColorRgba::from_hex(0xFF00FFFF)),
        //     border_radius: BorderRadius::all(100.0), // Larger than half the size
        //     border: Border::default(),
        // },
        // // Test 20: Empty fill (Fill::None)
        // RenderCommand::Rect {
        //     x: 280.0,
        //     y: 490.0,
        //     width: 100.0,
        //     height: 100.0,
        //     fill: Fill::None,
        //     border_radius: BorderRadius::default(),
        //     border: Border::all(BorderSide::new(3.0, ColorRgba::from_hex(0xFF000000))),
        // },
    ]
}
