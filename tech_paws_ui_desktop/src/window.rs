use tech_paws_ui::widgets::builder::BuildContext;

use crate::window_manager::WindowManager;

pub trait Window<App, Event = ()> {
    fn on_event(&mut self, app: &mut App, event: &Event) {}

    fn build(&mut self, app: &mut App, ctx: &mut BuildContext);
}
