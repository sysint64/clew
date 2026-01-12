use crate::{
    Vec2,
    layout::WidgetPlacement,
    render::{PixelExtension, RenderCommand, RenderContext},
};

use super::State;

pub fn render(ctx: &mut RenderContext, placement: &WidgetPlacement, state: &State) {
    let size = placement.rect.size().px(ctx);
    let position = placement.rect.position().px(ctx);

    let text_id = state
        .text_id
        .expect("Should be initialized during build phase");

    let text = ctx.text.get_mut(text_id);
    let text_size = text.layout();
    let text_position =
        position + Vec2::new(0., state.vertical_align.position(size.y, text_size.y));

    ctx.push_command(
        placement.zindex,
        RenderCommand::Text {
            x: text_position.x,
            y: text_position.y,
            text_id,
            tint_color: Some(state.color),
        },
    );
}
