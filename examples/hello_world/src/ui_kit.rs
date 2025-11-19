use std::{any::Any, hash::Hash, rc::Rc};
use tech_paws_ui::{
    AlignX, AlignY, Border, BorderRadius, BorderSide, ColorRgba, Constraints, EdgeInsets, Gradient,
    LinearGradient, Size, SizeConstraint, WidgetId, WidgetRef, WidgetType, impl_id,
    impl_position_methods, impl_width_methods,
    io::UserInput,
    render::{Fill, PixelExtension, RenderCommand, RenderContext, cache_string},
    state::WidgetState,
    text::{StringId, TextId},
    widgets::{
        builder::BuildContext,
        decorated_box::decorated_box,
        gesture_detector::{GestureDetectorResponse, gesture_detector},
        scope::scope,
        text::text,
    },
};

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

impl<'a> ButtonBuilder<'a> {
    impl_id!();
    impl_width_methods!();
    impl_position_methods!();

    pub fn build(self, ctx: &mut BuildContext) -> ButtonResponse {
        let response = scope(self.id).build(ctx, |ctx| {
            gesture_detector().build(ctx, |ctx| {
                let response = ctx.of::<GestureDetectorResponse>().unwrap();

                let gradient = if response.is_active() && response.is_hot() {
                    LinearGradient::vertical(vec![
                        ColorRgba::from_hex(0xFF1C1C1C),
                        ColorRgba::from_hex(0xFF212121),
                    ])
                } else if response.is_hot() {
                    LinearGradient::vertical(vec![
                        ColorRgba::from_hex(0xFF383838),
                        ColorRgba::from_hex(0xFF2E2E2E),
                    ])
                } else {
                    LinearGradient::vertical(vec![
                        ColorRgba::from_hex(0xFF2F2F2F),
                        ColorRgba::from_hex(0xFF272727),
                    ])
                };

                let border_color = if response.is_focused() {
                    ColorRgba::from_hex(0xFF357CCE)
                } else if response.is_active() && response.is_hot() {
                    ColorRgba::from_hex(0xFF414141)
                } else if response.is_hot() {
                    ColorRgba::from_hex(0xFF616161)
                } else {
                    ColorRgba::from_hex(0xFF414141)
                };

                decorated_box()
                    .border_radius(BorderRadius::all(3.))
                    .add_linear_gradient(gradient)
                    .border(Border::all(BorderSide::new(1., border_color)))
                    .build(ctx, |ctx| {
                        text(self.text)
                            .text_align_x(AlignX::Center)
                            .text_align_y(AlignY::Center)
                            .width(self.width)
                            .constraints(self.constraints)
                            .build(ctx);
                    });
            })
        });

        // let response = scope(self.id).build(ctx, |ctx| {
        //     gesture_detector().build(ctx, |ctx| {
        //         let response = ctx.of::<GestureDetectorResponse>().unwrap();

        //         let gradient = if response.is_active() && response.is_hot() {
        //             LinearGradient::vertical(vec![
        //                 ColorRgba::from_hex(0xFF1C1C1C),
        //                 ColorRgba::from_hex(0xFF212121),
        //             ])
        //         } else if response.is_hot() {
        //             LinearGradient::vertical(vec![
        //                 ColorRgba::from_hex(0xFF383838),
        //                 ColorRgba::from_hex(0xFF2E2E2E),
        //             ])
        //         } else {
        //             LinearGradient::vertical(vec![
        //                 ColorRgba::from_hex(0xFF2F2F2F),
        //                 ColorRgba::from_hex(0xFF272727),
        //             ])
        //         };

        //         let border_color = if response.is_focused() {
        //             ColorRgba::from_hex(0xFF357CCE)
        //         } else if response.is_active() && response.is_hot() {
        //             ColorRgba::from_hex(0xFF414141)
        //         } else if response.is_hot() {
        //             ColorRgba::from_hex(0xFF616161)
        //         } else {
        //             ColorRgba::from_hex(0xFF414141)
        //         };

        //         decorated_box()
        //             .border_radius(BorderRadius::all(3.))
        //             .add_linear_gradient(gradient)
        //             .border(Border::all(BorderSide::new(1., border_color)))
        //             .build(ctx, |ctx| {
        //                 text(self.text)
        //                     .text_align_x(AlignX::Center)
        //                     .text_align_y(AlignY::Center)
        //                     .width(self.width)
        //                     .constraints(self.constraints)
        //                     .build(ctx);
        //             });
        //     })
        // });

        ButtonResponse {
            clicked: response.clicked(),
        }
    }
}

#[track_caller]
pub fn button(text: &str) -> ButtonBuilder<'_> {
    ButtonBuilder {
        id: WidgetId::auto(),
        text,
        width: SizeConstraint::Fixed(100.),
        // width: SizeConstraint::Wrap,
        align_x: None,
        align_y: None,
        padding: None,
        zindex: None,
        constraints: Constraints {
            min_width: 20.,
            min_height: 20.,
            max_width: f32::INFINITY,
            max_height: 20.,
        },
    }
}
