use clew::{shortcuts::ShortcutManager, widgets::builder::BuildContext};

pub trait Window<App, Event = ()> {
    fn on_event(&mut self, _app: &mut App, _event: &Event) {}

    fn on_init(&mut self, _shortcut_manager: &mut ShortcutManager) {}

    fn on_shortcut(&mut self, _shortcut_manager: &ShortcutManager) {}

    fn build(&mut self, app: &mut App, ctx: &mut BuildContext);
}
