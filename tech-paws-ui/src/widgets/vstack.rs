use crate::{
    AlignX, AlignY, Constraints, CrossAxisAlignment, MainAxisAlignment, Size, SizeConstraint,
    impl_position_methods, impl_size_methods,
    layout::{ContainerKind, LayoutCommand},
};

use super::builder::BuildContext;

pub struct VStackBuilder {
    size: Size,
    spacing: f32,
    constraints: Constraints,
    align_x: Option<AlignX>,
    align_y: Option<AlignY>,
    zindex: Option<i32>,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
}

impl VStackBuilder {
    impl_size_methods!();
    impl_position_methods!();

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

        context.with_align(self.align_x, self.align_y, |context| {
            context.push_layout_command(LayoutCommand::BeginContainer {
                widget_ref: None,
                zindex: 0,
                kind: ContainerKind::VStack {
                    spacing: self.spacing,
                    main_axis_alignment: self.main_axis_alignment,
                    cross_axis_alignment: self.cross_axis_alignment,
                },
                size: self.size,
                constraints: self.constraints,
            });
            callback(context);
            context.push_layout_command(LayoutCommand::EndContainer);
        });

        context.current_zindex = last_zindex;
    }
}

pub fn vstack() -> VStackBuilder {
    VStackBuilder {
        size: Size::default(),
        constraints: Constraints::default(),
        spacing: 5.,
        zindex: None,
        align_x: None,
        align_y: None,
        main_axis_alignment: MainAxisAlignment::default(),
        cross_axis_alignment: CrossAxisAlignment::default(),
    }
}
