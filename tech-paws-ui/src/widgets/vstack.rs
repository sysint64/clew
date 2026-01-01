use smallvec::SmallVec;

use crate::{
    AlignX, AlignY, Constraints, CrossAxisAlignment, EdgeInsets, MainAxisAlignment, Size,
    SizeConstraint, WidgetRef, impl_position_methods, impl_size_methods,
    layout::{ContainerKind, LayoutCommand},
};

use super::builder::BuildContext;

pub struct VStackBuilder {
    size: Size,
    rtl_aware: bool,
    spacing: f32,
    constraints: Constraints,
    zindex: Option<i32>,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    padding: EdgeInsets,
    backgrounds: SmallVec<[WidgetRef; 8]>,
}

impl VStackBuilder {
    impl_size_methods!();
    impl_position_methods!();

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;

        self
    }

    pub fn rtl_aware(mut self, rtl_aware: bool) -> Self {
        self.rtl_aware = rtl_aware;

        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;

        self
    }

    pub fn background(mut self, decorator: WidgetRef) -> Self {
        self.backgrounds.push(decorator);

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

    #[profiling::function]
    pub fn build<F>(mut self, context: &mut BuildContext, callback: F)
    where
        F: FnOnce(&mut BuildContext),
    {
        let last_zindex = context.current_zindex;
        context.current_zindex = self.zindex.unwrap_or(context.current_zindex);
        let mut backgrounds = std::mem::take(context.decorators);
        backgrounds.append(&mut self.backgrounds);

        context.push_layout_command(LayoutCommand::BeginContainer {
            decorators: backgrounds,
            zindex: 0,
            padding: self.padding,
            kind: ContainerKind::VStack {
                spacing: self.spacing,
                rtl_aware: self.rtl_aware,
                main_axis_alignment: self.main_axis_alignment,
                cross_axis_alignment: self.cross_axis_alignment,
            },
            size: self.size,
            constraints: self.constraints,
        });
        callback(context);
        context.push_layout_command(LayoutCommand::EndContainer);

        context.current_zindex = last_zindex;
    }
}

pub fn vstack() -> VStackBuilder {
    VStackBuilder {
        size: Size::default(),
        constraints: Constraints::default(),
        spacing: 5.,
        rtl_aware: false,
        zindex: None,
        main_axis_alignment: MainAxisAlignment::default(),
        cross_axis_alignment: CrossAxisAlignment::default(),
        padding: EdgeInsets::ZERO,
        backgrounds: SmallVec::new(),
    }
}
