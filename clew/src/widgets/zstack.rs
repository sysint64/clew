use clew_derive::WidgetBuilder;

use crate::{
    AlignX, AlignY,
    layout::{ContainerKind, LayoutCommand},
};

use super::{FrameBuilder, builder::BuildContext};

pub struct ZStack;

#[derive(WidgetBuilder)]
pub struct ZStackBuilder {
    frame: FrameBuilder,
    align_x: AlignX,
    align_y: AlignY,
}

impl ZStackBuilder {
    pub fn align_x(mut self, align: AlignX) -> Self {
        self.align_x = align;
        self
    }

    pub fn align_y(mut self, align: AlignY) -> Self {
        self.align_y = align;
        self
    }

    pub fn build<F>(mut self, context: &mut BuildContext, callback: F)
    where
        F: FnOnce(&mut BuildContext),
    {
        let mut backgrounds = std::mem::take(context.backgrounds);
        backgrounds.append(&mut self.frame.backgrounds);

        let mut foregrounds = std::mem::take(context.foregrounds);
        foregrounds.append(&mut self.frame.foregrounds);

        if self.frame.offset_x != 0. || self.frame.offset_y != 0. {
            context.push_layout_command(LayoutCommand::BeginOffset {
                offset_x: self.frame.offset_x,
                offset_y: self.frame.offset_y,
            });
        }

        context.push_layout_command(LayoutCommand::BeginContainer {
            backgrounds,
            foregrounds,
            zindex: self.frame.zindex,
            padding: self.frame.padding,
            margin: self.frame.margin,
            kind: ContainerKind::ZStack {
                align_x: self.align_x,
                align_y: self.align_y,
            },
            size: self.frame.size,
            constraints: self.frame.constraints,
            clip: self.frame.clip,
        });
        callback(context);
        context.push_layout_command(LayoutCommand::EndContainer);

        if self.frame.offset_x != 0. || self.frame.offset_y != 0. {
            context.push_layout_command(LayoutCommand::EndOffset);
        }
    }
}

pub fn zstack() -> ZStackBuilder {
    ZStackBuilder {
        frame: FrameBuilder::new(),
        align_x: AlignX::Left,
        align_y: AlignY::Top,
    }
}
