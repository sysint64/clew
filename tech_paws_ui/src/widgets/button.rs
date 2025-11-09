use super::builder::BuildContext;
use crate::{
    AlignX, AlignY, Constraints, EdgeInsets, Size, SizeConstraint, WidgetId, impl_position_methods,
    impl_width_methods,
    layout::{ContainerKind, LayoutCommand},
};
use std::hash::Hash;

pub struct ButtonBuilder<'a> {
    id: WidgetId,
    text: &'a str,
    width: SizeConstraint,
    constraints: Constraints,
    align_x: Option<AlignX>,
    align_y: Option<AlignY>,
    zindex: Option<i32>,
    padding: Option<EdgeInsets>,
}

pub struct ButtonResponse {
    clicked: bool,
}

impl ButtonResponse {
    pub fn clicked(&self) -> bool {
        self.clicked
    }
}

pub(crate) struct ButtonState {
    // pub(crate) text: StringId,
    pub(crate) clicked: bool,
}

impl<'a> ButtonBuilder<'a> {
    impl_width_methods!();
    impl_position_methods!();

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = Some(padding);

        self
    }

    pub fn build(&self, context: &mut BuildContext) -> ButtonResponse {
        // let text = context.state.string_interner.get_or_intern(self.text);
        let size = Size::new(self.width, SizeConstraint::Fixed(20.0));

        if let Some(padding) = self.padding {
            let mut padding_containts = self.constraints;
            padding_containts.expand(padding);

            context.push_layout_command(LayoutCommand::BeginContainer {
                kind: ContainerKind::Padding { padding },
                size,
                constraints: self.constraints,
            });

            context.push_layout_command(LayoutCommand::Fixed {
                id: self.id,
                constraints: self.constraints,
                size,
                zindex: self.zindex.unwrap_or(context.current_zindex),
            });

            context.push_layout_command(LayoutCommand::EndContainer);
        } else {
            context.push_layout_command(LayoutCommand::Fixed {
                id: self.id,
                constraints: self.constraints,
                size,
                zindex: self.zindex.unwrap_or(context.current_zindex),
            });
        }

        context.widgets_states.accessed_this_frame.insert(self.id);

        let state = context
            .widgets_states
            .get_or_insert::<ButtonState, _>(self.id, || ButtonState { clicked: false });

        // state.text = text;

        ButtonResponse {
            clicked: state.clicked,
        }
    }
}

#[track_caller]
pub fn button(text: &str) -> ButtonBuilder<'_> {
    ButtonBuilder {
        id: WidgetId::auto_with_seed(text),
        text,
        width: SizeConstraint::Wrap,
        align_x: None,
        align_y: None,
        padding: None,
        zindex: None,
        constraints: Constraints {
            min_width: Some(100.),
            min_height: Some(20.),
            max_width: None,
            max_height: Some(20.),
        },
    }
}

#[track_caller]
pub fn button_id(id: impl Hash, text: &str) -> ButtonBuilder {
    ButtonBuilder {
        id: WidgetId::auto_with_seed(id),
        text,
        width: SizeConstraint::Wrap,
        align_x: None,
        align_y: None,
        padding: None,
        zindex: None,
        constraints: Constraints {
            min_width: Some(100.),
            min_height: Some(20.),
            max_width: None,
            max_height: Some(20.),
        },
    }
}
