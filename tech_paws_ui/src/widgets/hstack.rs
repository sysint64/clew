use crate::{
    AlignX, AlignY, Constraints, CrossAxisAlignment, MainAxisAlignment, Size, SizeConstraint,
    impl_position_methods, impl_size_methods,
    layout::{ContainerKind, LayoutCommand},
};

use super::builder::BuildContext;

pub struct HStackBuilder {
    size: Size,
    rtl_aware: bool,
    spacing: f32,
    constraints: Constraints,
    align_x: Option<AlignX>,
    align_y: Option<AlignY>,
    zindex: Option<i32>,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
}

impl HStackBuilder {
    impl_size_methods!();
    impl_position_methods!();

    pub fn rtl_aware(mut self, rtl_aware: bool) -> Self {
        self.rtl_aware = rtl_aware;

        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;

        self
    }

    pub fn main_axis_alignment(mut self, value: MainAxisAlignment) -> Self {
        self.main_axis_alignment = value;

        self
    }

    pub fn cross_axis_alignment(mut self, value: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = value;

        self
    }

    pub fn build<F>(&self, context: &mut BuildContext, callback: F)
    where
        F: FnOnce(&mut BuildContext),
    {
        let last_zindex = context.current_zindex;
        context.current_zindex = self.zindex.unwrap_or(context.current_zindex);

        context.push_layout_command(LayoutCommand::BeginContainer {
            kind: ContainerKind::HStack {
                spacing: self.spacing,
                main_axis_alignment: self.main_axis_alignment,
                cross_axis_alignment: self.cross_axis_alignment,
                rtl_aware: self.rtl_aware,
            },
            size: self.size,
            constraints: self.constraints,
        });

        context.with_align(self.align_x, self.align_y, |context| callback(context));
        context.push_layout_command(LayoutCommand::EndContainer);

        context.current_zindex = last_zindex;
    }
}

pub fn hstack() -> HStackBuilder {
    HStackBuilder {
        size: Size::default(),
        constraints: Constraints::default(),
        rtl_aware: false,
        spacing: 5.,
        zindex: None,
        align_x: None,
        align_y: None,
        main_axis_alignment: MainAxisAlignment::default(),
        cross_axis_alignment: CrossAxisAlignment::default(),
    }
}
