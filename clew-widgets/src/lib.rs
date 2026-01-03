use std::hash::Hash;
use tech_paws_ui::{
    AlignX, AlignY, Border, BorderRadius, BorderSide, ColorRgba, Constraints, EdgeInsets,
    LinearGradient, SizeConstraint, WidgetId, impl_id, impl_position_methods, impl_width_methods,
    state::WidgetState,
    widgets::{
        builder::BuildContext,
        decorated_box::{decorated_box, decoration},
        gesture_detector::{DragState, GestureDetectorResponse, gesture_detector},
        scope::scope,
        scroll_area::{ScrollAreaResponse, set_scroll_progress_x, set_scroll_progress_y},
        text::text,
        widget::Widget,
        zstack::zstack,
    },
};
use tech_paws_ui_derive::WidgetState;

pub struct ButtonBuilder<'a> {
    id: WidgetId,
    text: &'a str,
    width: SizeConstraint,
    constraints: Constraints,
    zindex: Option<i32>,
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

    #[profiling::function]
    pub fn build(self, ctx: &mut BuildContext) -> ButtonResponse {
        let response = scope(self.id).build(ctx, |ctx| {
            gesture_detector()
                .clickable(true)
                .focusable(true)
                .build(ctx, |ctx| {
                    let response = ctx.of::<GestureDetectorResponse>().unwrap();

                    let gradient = {
                        if response.is_active() && response.is_hot() {
                            LinearGradient::vertical((
                                ColorRgba::from_hex(0xFF1C1C1C),
                                ColorRgba::from_hex(0xFF212121),
                            ))
                        } else if response.is_hot() {
                            LinearGradient::vertical((
                                ColorRgba::from_hex(0xFF383838),
                                ColorRgba::from_hex(0xFF2E2E2E),
                            ))
                        } else {
                            LinearGradient::vertical((
                                ColorRgba::from_hex(0xFF2F2F2F),
                                ColorRgba::from_hex(0xFF272727),
                            ))
                        }
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

                    text(self.text)
                        .background(
                            decoration()
                                .border_radius(BorderRadius::all(3.))
                                .add_linear_gradient(gradient)
                                .border(Border::all(BorderSide::new(1., border_color)))
                                .build(ctx),
                        )
                        .text_align_x(AlignX::Center)
                        .text_vertical_align(AlignY::Center)
                        .width(self.width)
                        .constraints(self.constraints)
                        .padding(EdgeInsets::symmetric(12., 8.))
                        .build(ctx);
                })
        });

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
        width: SizeConstraint::Wrap,
        zindex: None,
        constraints: Constraints {
            min_width: 20.,
            min_height: 0.,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
        },
    }
}

#[derive(WidgetState, Default)]
pub struct HorizontalScrollBar {
    offset: f64,
    last_offset: f64,
}

impl Widget for HorizontalScrollBar {
    type Event = ();

    fn build(&mut self, ctx: &mut BuildContext) {
        zstack()
            .fill_max_size()
            .align_y(AlignY::Bottom)
            .build(ctx, |ctx| {
                gesture_detector().dragable(true).build(ctx, |ctx| {
                    let gesture = ctx.of::<GestureDetectorResponse>().unwrap().clone();

                    let color = ColorRgba::from_hex(0xFFFFFFFF).with_opacity(
                        if gesture.is_hot() || gesture.is_active() {
                            0.5
                        } else {
                            0.4
                        },
                    );

                    let response = ctx.of::<ScrollAreaResponse>().unwrap().clone();
                    let horizontal_padding = 16.;
                    let mut scroll_area_width = response.width - horizontal_padding;

                    if response.overflow_y {
                        scroll_area_width -= 8.;
                    }

                    let bar_width = f64::max(16., scroll_area_width * response.fraction_x);

                    if gesture.drag_state == DragState::None || gesture.drag_state == DragState::End
                    {
                        self.offset = (scroll_area_width - bar_width) * response.progress_x;
                    } else if gesture.drag_state == DragState::Start {
                        self.last_offset = self.offset;
                    } else {
                        self.offset =
                            self.last_offset + gesture.drag_x as f64 - gesture.drag_start_x as f64;
                        self.offset = self.offset.clamp(0., scroll_area_width - bar_width);

                        let progress_x = self.offset / (scroll_area_width - bar_width);

                        set_scroll_progress_x(ctx, response.id, progress_x);
                    }

                    decorated_box()
                        .color(color)
                        .border_radius(BorderRadius::all(if gesture.is_active() { 0. } else { 2. }))
                        .width(bar_width)
                        .height(if gesture.is_active() { 8. } else { 4. })
                        .offset_x(self.offset as f32)
                        .padding(if gesture.is_active() {
                            EdgeInsets::symmetric(8., 6.)
                        } else {
                            EdgeInsets::all(8.)
                        })
                        .build(ctx);
                });
            });
    }
}

#[derive(WidgetState, Default)]
pub struct VerticalScrollBar {
    offset: f64,
    last_offset: f64,
}

impl Widget for VerticalScrollBar {
    type Event = ();

    fn build(&mut self, ctx: &mut BuildContext) {
        zstack()
            .fill_max_size()
            .align_x(AlignX::Right)
            .build(ctx, |ctx| {
                gesture_detector().dragable(true).build(ctx, |ctx| {
                    let gesture = ctx.of::<GestureDetectorResponse>().unwrap().clone();

                    let color = ColorRgba::from_hex(0xFFFFFFFF).with_opacity(
                        if gesture.is_hot() || gesture.is_active() {
                            0.5
                        } else {
                            0.4
                        },
                    );

                    let response = ctx.of::<ScrollAreaResponse>().unwrap().clone();
                    let vertical_padding = 16.;
                    let mut scroll_area_height = response.height - vertical_padding;

                    if response.overflow_x {
                        scroll_area_height -= 8.;
                    }

                    let bar_height = f64::max(16., scroll_area_height * response.fraction_y);

                    if gesture.drag_state == DragState::None || gesture.drag_state == DragState::End
                    {
                        self.offset = (scroll_area_height - bar_height) * response.progress_y;
                    } else if gesture.drag_state == DragState::Start {
                        self.last_offset = self.offset;
                    } else {
                        self.offset =
                            self.last_offset + gesture.drag_y as f64 - gesture.drag_start_y as f64;
                        self.offset = self.offset.clamp(0., scroll_area_height - bar_height);

                        let progress_y = self.offset / (scroll_area_height - bar_height);

                        set_scroll_progress_y(ctx, response.id, progress_y);
                    }

                    decorated_box()
                        .color(color)
                        .border_radius(BorderRadius::all(if gesture.is_active() { 0. } else { 2. }))
                        .width(if gesture.is_active() { 8. } else { 4. })
                        .height(bar_height)
                        .offset_y(self.offset as f32)
                        .padding(if gesture.is_active() {
                            EdgeInsets::symmetric(6., 8.)
                        } else {
                            EdgeInsets::all(8.)
                        })
                        .build(ctx);
                });
            });
    }
}
