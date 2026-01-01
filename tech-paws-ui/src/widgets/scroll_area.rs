use std::any::Any;

use smallvec::SmallVec;

use crate::{
    Clip, Constraints, EdgeInsets, Size, SizeConstraint, WidgetId, WidgetRef, WidgetType, impl_id,
    impl_size_methods,
    interaction::InteractionState,
    io::UserInput,
    layout::{ContainerKind, LayoutCommand, LayoutMeasure},
    state::WidgetState,
};
use std::hash::Hash;

use super::builder::BuildContext;

pub struct ScrollAreaWidget;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollDirection {
    Horizontal,
    Vertical,
    Both,
}

pub struct ScrollAreaBuilder {
    id: WidgetId,
    size: Size,
    constraints: Constraints,
    zindex: Option<i32>,
    padding: EdgeInsets,
    margin: EdgeInsets,
    scroll_direction: ScrollDirection,
    clip: Clip,
    backgrounds: SmallVec<[WidgetRef; 8]>,
}

#[derive(Clone, PartialEq)]
pub struct State {
    last_offset_x: f32,
    last_offset_y: f32,
    offset_x: f32,
    offset_y: f32,
    fraction_x: f32,
    fraction_y: f32,
    progress_x: f32,
    progress_y: f32,
    width: f32,
    height: f32,
    content_width: f32,
    content_height: f32,
    overflow_x: bool,
    overflow_y: bool,
    scroll_direction: ScrollDirection,
}

#[derive(Clone, PartialEq)]
pub struct ScrollAreaResponse {
    pub id: WidgetId,
    pub offset_x: f32,
    pub offset_y: f32,
    pub fraction_x: f32,
    pub fraction_y: f32,
    pub progress_x: f32,
    pub progress_y: f32,
    pub width: f32,
    pub height: f32,
    pub content_width: f32,
    pub content_height: f32,
    pub overflow_x: bool,
    pub overflow_y: bool,
}

impl WidgetState for State {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl ScrollAreaBuilder {
    impl_id!();
    impl_size_methods!();

    pub fn clip(mut self, clip: Clip) -> Self {
        self.clip = clip;

        self
    }

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;

        self
    }

    pub fn margin(mut self, margin: EdgeInsets) -> Self {
        self.margin = margin;

        self
    }

    pub fn background(mut self, decorator: WidgetRef) -> Self {
        self.backgrounds.push(decorator);

        self
    }

    pub fn scroll_direction(mut self, scroll_direction: ScrollDirection) -> Self {
        self.scroll_direction = scroll_direction;

        self
    }

    #[profiling::function]
    pub fn build<F>(mut self, context: &mut BuildContext, callback: F) -> ScrollAreaResponse
    where
        F: FnOnce(&mut BuildContext),
    {
        let id = self.id.with_seed(context.id_seed);
        let widget_ref = WidgetRef::new(WidgetType::of::<ScrollAreaWidget>(), id);

        let last_zindex = context.current_zindex;
        context.current_zindex = self.zindex.unwrap_or(context.current_zindex);
        let mut widget_refs = std::mem::take(context.decorators);
        widget_refs.append(&mut self.backgrounds);
        widget_refs.push(widget_ref);

        let (offset_x, offset_y, response) = {
            let state = context
                .widgets_states
                .scroll_area
                .get_or_insert(id, || State {
                    last_offset_x: 0.,
                    last_offset_y: 0.,
                    offset_x: 0.,
                    offset_y: 0.,
                    overflow_x: false,
                    overflow_y: false,
                    scroll_direction: self.scroll_direction,
                    fraction_x: 0.,
                    fraction_y: 0.,
                    progress_x: 0.,
                    progress_y: 0.,
                    width: 0.,
                    height: 0.,
                    content_width: 0.,
                    content_height: 0.,
                });

            state.scroll_direction = self.scroll_direction;

            (
                state.offset_x,
                state.offset_y,
                ScrollAreaResponse {
                    id,
                    offset_x: state.offset_x,
                    offset_y: state.offset_y,
                    overflow_x: state.overflow_x,
                    overflow_y: state.overflow_y,
                    fraction_x: state.fraction_x,
                    fraction_y: state.fraction_y,
                    progress_x: state.progress_x,
                    progress_y: state.progress_y,
                    width: state.width,
                    height: state.height,
                    content_width: state.content_width,
                    content_height: state.content_height,
                },
            )
        };

        context.push_layout_command(LayoutCommand::BeginContainer {
            backgrounds: widget_refs,
            zindex: 0,
            padding: self.padding,
            margin: self.margin,
            kind: ContainerKind::Measure { id },
            size: self.size,
            constraints: self.constraints,
            clip: self.clip,
        });

        context.push_layout_command(LayoutCommand::BeginOffset { offset_x, offset_y });
        context.with_user_data(response.clone(), callback);
        context.push_layout_command(LayoutCommand::EndOffset);

        context.push_layout_command(LayoutCommand::EndContainer);

        context.current_zindex = last_zindex;

        context
            .widgets_states
            .scroll_area
            .accessed_this_frame
            .insert(id);

        response
    }
}

#[track_caller]
pub fn scroll_area() -> ScrollAreaBuilder {
    ScrollAreaBuilder {
        id: WidgetId::auto(),
        size: Size::default(),
        constraints: Constraints::default(),
        zindex: None,
        padding: EdgeInsets::ZERO,
        margin: EdgeInsets::ZERO,
        scroll_direction: ScrollDirection::Vertical,
        clip: Clip::Rect,
        backgrounds: SmallVec::new(),
    }
}

pub fn set_scroll_offset_x(context: &mut BuildContext, id: WidgetId, value: f32) {
    let state = context.widgets_states.scroll_area.get_mut(id);

    if let Some(state) = state {
        state.offset_x = -value;
    }
}

pub fn set_scroll_offset_y(context: &mut BuildContext, id: WidgetId, value: f32) {
    let state = context.widgets_states.scroll_area.get_mut(id);

    if let Some(state) = state {
        state.offset_y = -value;
    }
}

pub fn set_scroll_progress_x(context: &mut BuildContext, id: WidgetId, value: f32) {
    let state = context.widgets_states.scroll_area.get_mut(id);

    if let Some(state) = state {
        state.offset_x = -(state.content_width - state.width) * value;
    }
}

pub fn set_scroll_progress_y(context: &mut BuildContext, id: WidgetId, value: f32) {
    let state = context.widgets_states.scroll_area.get_mut(id);

    if let Some(state) = state {
        state.offset_y = -(state.content_height - state.height) * value;
    }
}

pub fn handle_interaction(
    id: WidgetId,
    input: &UserInput,
    interaction: &mut InteractionState,
    widget_state: &mut State,
    layout_measure: &LayoutMeasure,
) {
    if widget_state.scroll_direction == ScrollDirection::Vertical
        || widget_state.scroll_direction == ScrollDirection::Both
    {
        if input.mouse_wheel_delta_y != 0. {
            widget_state.offset_y += input.mouse_wheel_delta_y as f32;
        }

        widget_state.offset_y = widget_state.offset_y.clamp(
            f32::min(0., -(layout_measure.wrap_height - layout_measure.height)),
            0.,
        );

        widget_state.overflow_y = layout_measure.height - layout_measure.wrap_height <= 0.;
        widget_state.fraction_y = layout_measure.height / layout_measure.wrap_height;
        widget_state.height = layout_measure.height;
        widget_state.content_height = layout_measure.wrap_height;
        widget_state.progress_y =
            -widget_state.offset_y / (layout_measure.wrap_height - layout_measure.height);
        widget_state.progress_y = widget_state.progress_y.clamp(0., 1.);
    }

    if widget_state.scroll_direction == ScrollDirection::Horizontal
        || widget_state.scroll_direction == ScrollDirection::Both
    {
        if input.mouse_wheel_delta_x != 0. {
            widget_state.offset_x += input.mouse_wheel_delta_x as f32;
        }

        widget_state.offset_x = widget_state.offset_x.clamp(
            f32::min(0., -(layout_measure.wrap_width - layout_measure.width)),
            0.,
        );

        widget_state.overflow_x = layout_measure.width - layout_measure.wrap_width <= 0.;
        widget_state.fraction_x = layout_measure.width / layout_measure.wrap_width;
        widget_state.width = layout_measure.width;
        widget_state.content_width = layout_measure.wrap_width;
        widget_state.progress_x =
            -widget_state.offset_x / (layout_measure.wrap_width - layout_measure.width);
        widget_state.progress_x = widget_state.progress_x.clamp(0., 1.);
    }
}
