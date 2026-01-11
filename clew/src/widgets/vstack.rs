use smallvec::SmallVec;

use crate::{
    Clip, Constraints, CrossAxisAlignment, EdgeInsets, MainAxisAlignment, Size, SizeConstraint,
    WidgetRef, impl_position_methods, impl_size_methods,
    layout::{ContainerKind, LayoutCommand},
};

use super::builder::BuildContext;

pub struct VStackBuilder {
    size: Size,
    constraints: Constraints,
    zindex: i32,
    padding: EdgeInsets,
    margin: EdgeInsets,
    backgrounds: SmallVec<[WidgetRef; 8]>,
    foregrounds: SmallVec<[WidgetRef; 8]>,

    rtl_aware: bool,
    spacing: f32,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    clip: Clip,
}

impl VStackBuilder {
    impl_size_methods!();
    impl_position_methods!();

    pub fn clip(mut self, clip: Clip) -> Self {
        self.clip = clip;

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

    pub fn foreground(mut self, decorator: WidgetRef) -> Self {
        self.foregrounds.push(decorator);

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
        let mut backgrounds = std::mem::take(context.backgrounds);
        backgrounds.append(&mut self.backgrounds);

        let mut foregrounds = std::mem::take(context.foregrounds);
        foregrounds.append(&mut self.foregrounds);

        context.push_layout_command(LayoutCommand::BeginContainer {
            backgrounds,
            foregrounds,
            zindex: self.zindex,
            padding: self.padding,
            margin: self.margin,
            kind: ContainerKind::VStack {
                spacing: self.spacing,
                rtl_aware: self.rtl_aware,
                main_axis_alignment: self.main_axis_alignment,
                cross_axis_alignment: self.cross_axis_alignment,
            },
            size: self.size,
            constraints: self.constraints,
            clip: self.clip,
        });
        callback(context);
        context.push_layout_command(LayoutCommand::EndContainer);
    }
}

pub fn vstack() -> VStackBuilder {
    VStackBuilder {
        size: Size::default(),
        constraints: Constraints::default(),
        spacing: 5.,
        rtl_aware: false,
        zindex: 0,
        main_axis_alignment: MainAxisAlignment::default(),
        cross_axis_alignment: CrossAxisAlignment::default(),
        padding: EdgeInsets::ZERO,
        margin: EdgeInsets::ZERO,
        backgrounds: SmallVec::new(),
        foregrounds: SmallVec::new(),
        clip: Clip::None,
    }
}
