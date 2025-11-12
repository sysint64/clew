use crate::{
    AlignX, AlignY, View,
    event_queue::EventQueue,
    layout::{ContainerKind, LayoutCommand},
    state::WidgetsStates,
    task_spawner::TaskSpawner,
    text::{FontResources, StringInterner, TextsResources},
};

pub struct BuildContext<'a, 'b> {
    pub current_zindex: i32,
    pub layout_commands: &'a mut Vec<LayoutCommand>,
    pub widgets_states: &'a mut WidgetsStates,
    pub task_spawner: &'a mut TaskSpawner,
    pub event_queue: &'a mut EventQueue,
    pub text: &'a mut TextsResources<'b>,
    pub fonts: &'a mut FontResources,
    pub view: &'a View,
    pub string_interner: &'a mut StringInterner,
}

impl BuildContext<'_, '_> {
    pub fn push_layout_command(&mut self, command: LayoutCommand) {
        self.layout_commands.push(command);
    }

    pub fn with_align<F>(&mut self, align_x: Option<AlignX>, align_y: Option<AlignY>, callback: F)
    where
        F: FnOnce(&mut BuildContext),
    {
        if align_x.is_some() || align_y.is_some() {
            self.push_layout_command(LayoutCommand::BeginAlign {
                align_x: align_x.unwrap_or(AlignX::Left),
                align_y: align_y.unwrap_or(AlignY::Top),
            });
            callback(self);
            self.push_layout_command(LayoutCommand::EndAlign);
        } else {
            callback(self);
        }
    }
}

#[macro_export]
macro_rules! impl_size_methods {
    () => {
        pub fn size(mut self, size: Size) -> Self {
            self.size = size;
            self
        }

        pub fn width<T: Into<SizeConstraint>>(mut self, size: T) -> Self {
            self.size.width = size.into();
            self
        }

        pub fn height<T: Into<SizeConstraint>>(mut self, size: T) -> Self {
            self.size.height = size.into();
            self
        }

        pub fn fill_max_width(mut self) -> Self {
            self.size.width = SizeConstraint::Fill(1.);
            self
        }

        pub fn fill_max_height(mut self) -> Self {
            self.size.height = SizeConstraint::Fill(1.);
            self
        }

        pub fn fill_max_size(mut self) -> Self {
            self.size.width = SizeConstraint::Fill(1.);
            self.size.height = SizeConstraint::Fill(1.);
            self
        }

        pub fn constraints(mut self, constraints: Constraints) -> Self {
            self.constraints = constraints;
            self
        }

        pub fn max_width(mut self, value: f32) -> Self {
            self.constraints.max_width = Some(value);
            self
        }

        pub fn max_height(mut self, value: f32) -> Self {
            self.constraints.max_height = Some(value);
            self
        }

        pub fn min_width(mut self, value: f32) -> Self {
            self.constraints.min_width = Some(value);
            self
        }

        pub fn min_height(mut self, value: f32) -> Self {
            self.constraints.min_height = Some(value);
            self
        }
    };
}

#[macro_export]
macro_rules! impl_width_methods {
    () => {
        pub fn width<T: Into<SizeConstraint>>(mut self, size: T) -> Self {
            self.width = size.into();
            self
        }

        pub fn fill_max_width(mut self) -> Self {
            self.width = SizeConstraint::Fill(1.);
            self
        }

        pub fn max_width(mut self, value: f32) -> Self {
            self.constraints.max_width = Some(value);
            self
        }

        pub fn min_width(mut self, value: f32) -> Self {
            self.constraints.min_width = Some(value);
            self
        }
    };
}

#[macro_export]
macro_rules! impl_position_methods {
    () => {
        pub fn align_x(mut self, align: AlignX) -> Self {
            self.align_x = Some(align);
            self
        }

        pub fn align_y(mut self, align: AlignY) -> Self {
            self.align_y = Some(align);
            self
        }

        pub fn zindex(mut self, zindex: i32) -> Self {
            self.zindex = Some(zindex);
            self
        }
    };
}
