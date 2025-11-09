use crate::{layout::LayoutCommand, state::WidgetsStates, task_spawner::TaskSpawner};

pub struct BuildContext<'a> {
    pub current_zindex: i32,
    pub layout_commands: &'a mut Vec<LayoutCommand>,
    pub widgets_states: &'a mut WidgetsStates,
    pub task_spawner: &'a mut TaskSpawner,
}

impl BuildContext<'_> {
    pub fn push_layout_command(&mut self, command: LayoutCommand) {
        self.layout_commands.push(command);
    }

    pub fn emit<Event>(&mut self, event: Event) {}

    pub fn spawn<F, Event>(&self, future: F)
    where
        F: std::future::Future<Output = Event> + Send + 'static,
    {
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
