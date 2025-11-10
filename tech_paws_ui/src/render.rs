use crate::{
    text::{FontId, FontResources, TextId, TextsResources}, Border, BorderRadius, BorderSide, ColorRgba, Gradient, View
};

#[derive(Default)]
pub struct RenderState<'a> {
    pub fonts: FontResources,
    pub texts: TextsResources<'a>,
    render_commands: Vec<RenderCommand>,
}

pub trait Renderer {
    fn process_commands(&mut self, view: &View, state: &RenderState, commands: &[RenderCommand]);
}

pub enum RenderCommand {
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        fill: Fill,
        border_radius: BorderRadius,
        border: Border,
    },
    Oval {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        fill: Fill,
        border: Option<BorderSide>,
    },
    Text {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        text_id: TextId,
        tint_color: Option<ColorRgba>,
    },
    PushClipRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    PopClip,
}

pub enum Fill {
    None,
    Color(ColorRgba),
    Gradient(Gradient),
}
