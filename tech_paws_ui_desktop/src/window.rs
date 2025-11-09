use tech_paws_ui::widgets::builder::BuildContext;

pub trait Window<App> {
    fn build(&mut self, app: &mut App, ctx: &mut BuildContext);
}
