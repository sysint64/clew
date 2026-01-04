use smallvec::SmallVec;

use crate::{
    AlignX, AlignY, Clip, Constraints, EdgeInsets, Size, SizeConstraint, WidgetRef,
    impl_size_methods,
    layout::{ContainerKind, LayoutCommand},
};

use super::builder::BuildContext;

pub struct ZStack;

pub struct ZStackBuilder {
    size: Size,
    constraints: Constraints,
    zindex: Option<i32>,
    padding: EdgeInsets,
    margin: EdgeInsets,
    backgrounds: SmallVec<[WidgetRef; 8]>,

    align_x: AlignX,
    align_y: AlignY,
    offset_x: f32,
    offset_y: f32,
    clip: Clip,
}

impl ZStackBuilder {
    impl_size_methods!();

    pub fn clip(mut self, clip: Clip) -> Self {
        self.clip = clip;

        self
    }

    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset_x = x;
        self.offset_y = y;

        self
    }

    pub fn offset_x(mut self, offset: f32) -> Self {
        self.offset_x = offset;

        self
    }

    pub fn offset_y(mut self, offset: f32) -> Self {
        self.offset_y = offset;

        self
    }

    pub fn align_x(mut self, align: AlignX) -> Self {
        self.align_x = align;
        self
    }

    pub fn align_y(mut self, align: AlignY) -> Self {
        self.align_y = align;
        self
    }

    pub fn zindex(mut self, zindex: i32) -> Self {
        self.zindex = Some(zindex);
        self
    }

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;

        self
    }

    pub fn margin(mut self, margin: EdgeInsets) -> Self {
        self.margin = margin;

        self
    }

    pub fn background(mut self, decorator: WidgetRef) -> Self {
        self.backgrounds.push(decorator);

        self
    }

    pub fn build<F>(mut self, context: &mut BuildContext, callback: F)
    where
        F: FnOnce(&mut BuildContext),
    {
        let mut widgets = std::mem::take(context.decorators);
        widgets.append(&mut self.backgrounds);
        let last_zindex = context.current_zindex;
        context.current_zindex += 1;

        if self.offset_x != 0. || self.offset_y != 0. {
            context.push_layout_command(LayoutCommand::BeginOffset {
                offset_x: self.offset_x,
                offset_y: self.offset_y,
            });
        }

        context.push_layout_command(LayoutCommand::BeginContainer {
            backgrounds: widgets,
            zindex: last_zindex,
            padding: self.padding,
            margin: self.margin,
            kind: ContainerKind::ZStack {
                align_x: self.align_x,
                align_y: self.align_y,
            },
            size: self.size,
            constraints: self.constraints,
            clip: self.clip,
        });
        callback(context);
        context.push_layout_command(LayoutCommand::EndContainer);

        if self.offset_x != 0. || self.offset_y != 0. {
            context.push_layout_command(LayoutCommand::EndOffset);
        }

        context.current_zindex = last_zindex;
    }
}

#[track_caller]
pub fn zstack() -> ZStackBuilder {
    ZStackBuilder {
        padding: EdgeInsets::ZERO,
        margin: EdgeInsets::ZERO,
        align_x: AlignX::Left,
        align_y: AlignY::Top,
        zindex: None,
        constraints: Constraints::default(),
        size: Size::default(),
        backgrounds: SmallVec::new(),
        clip: Clip::Rect,
        offset_x: 0.,
        offset_y: 0.,
    }
}
