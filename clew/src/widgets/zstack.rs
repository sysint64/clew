use clew_derive::WidgetBuilder;

use crate::{
    AlignX, AlignY,
    layout::{ContainerKind, LayoutCommand},
};

use super::builder::{BuildContext, WidgetCommon};

pub struct ZStack;

#[derive(WidgetBuilder)]
pub struct ZStackBuilder {
    common: WidgetCommon,
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
        backgrounds.append(&mut self.common.backgrounds);

        let mut foregrounds = std::mem::take(context.foregrounds);
        foregrounds.append(&mut self.common.foregrounds);

        let last_zindex = context.current_zindex;
        context.current_zindex += 1;

        if self.common.offset_x != 0. || self.common.offset_y != 0. {
            context.push_layout_command(LayoutCommand::BeginOffset {
                offset_x: self.common.offset_x,
                offset_y: self.common.offset_y,
            });
        }

        context.push_layout_command(LayoutCommand::BeginContainer {
            backgrounds,
            foregrounds,
            zindex: last_zindex,
            padding: self.common.padding,
            margin: self.common.margin,
            kind: ContainerKind::ZStack {
                align_x: self.align_x,
                align_y: self.align_y,
            },
            size: self.common.size,
            constraints: self.common.constraints,
            clip: self.common.clip,
        });
        callback(context);
        context.push_layout_command(LayoutCommand::EndContainer);

        if self.common.offset_x != 0. || self.common.offset_y != 0. {
            context.push_layout_command(LayoutCommand::EndOffset);
        }

        context.current_zindex = last_zindex;
    }
}

#[track_caller]
pub fn zstack() -> ZStackBuilder {
    ZStackBuilder {
        common: WidgetCommon::default(),
        align_x: AlignX::Left,
        align_y: AlignY::Top,
    }
}
