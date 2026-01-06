extern crate self as clew;

pub mod animation;
pub mod assets;
mod foundation;
pub mod identifiable;
mod interaction;
pub mod io;
pub mod keyboard;
mod layout;
pub mod render;
pub mod state;
pub mod text;
mod widget_id;
pub mod widgets;

pub use animation::*;
pub use foundation::*;
pub use render::{Renderer, render};
pub use widget_id::*;
pub use widgets::*;

pub mod prelude {
    pub use crate::animation::Animation;
    pub use crate::foundation::Value;
    pub use crate::identifiable::Identifiable;
    pub use crate::state::WidgetState;
    pub use crate::widgets::builder::{Resolve, WidgetBuilder};
    pub use crate::widgets::stateful::StatefulWidgetBuilder;
}
