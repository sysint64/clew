use clew::widgets::builder::BuildContext;

pub trait Window<App, Event = ()> {
    fn on_event(&mut self, _app: &mut App, _event: &Event) {}

    fn build(&mut self, app: &mut App, ctx: &mut BuildContext);
}
