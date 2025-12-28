use std::any::Any;
use std::hash::Hash;

use glam::Vec2;

use crate::{
    AlignX, AlignY, Border, BorderRadius, BorderSide, ColorRgba, Constraints, EdgeInsets, Size,
    SizeConstraint, WidgetId, WidgetRef, WidgetType, impl_id, impl_position_methods,
    impl_size_methods, impl_width_methods,
    layout::{ContainerKind, LayoutCommand, WidgetPlacement},
    render::{Fill, PixelExtension, RenderCommand, RenderContext, cache_string},
    state::WidgetState,
    text::StringId,
};

use super::builder::BuildContext;

pub struct ZStack;

pub struct ZStackBuilder {
    padding: EdgeInsets,
    align_x: AlignX,
    align_y: AlignY,
    zindex: Option<i32>,
    constraints: Constraints,
    size: Size,
}

impl ZStackBuilder {
    impl_size_methods!();

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

    pub fn build<F>(&self, context: &mut BuildContext, callback: F)
    where
        F: FnOnce(&mut BuildContext),
    {
        let size = Size::new(SizeConstraint::Wrap, SizeConstraint::Wrap);
        let mut constraints = Constraints::default();

        let widgets = std::mem::take(context.decorators);
        let last_zindex = context.current_zindex;
        context.current_zindex += 1;

        context.push_layout_command(LayoutCommand::BeginContainer {
            decorators: widgets,
            zindex: last_zindex,
            padding: self.padding,
            kind: ContainerKind::ZStack,
            size,
            constraints,
        });
        callback(context);
        context.push_layout_command(LayoutCommand::EndContainer);

        context.current_zindex = last_zindex;
    }
}

#[track_caller]
pub fn zstack() -> ZStackBuilder {
    ZStackBuilder {
        padding: EdgeInsets::ZERO,
        align_x: AlignX::Left,
        align_y: AlignY::Top,
        zindex: None,
        constraints: Constraints::default(),
        size: Size::default(),
    }
}
