use smallvec::{SmallVec, smallvec};

use crate::{
    Clip, Constraints, EdgeInsets, ScrollDirection, Size, SizeConstraint, WidgetId, WidgetRef,
    impl_id, impl_size_methods,
    interaction::InteractionState,
    io::UserInput,
    layout::{ContainerKind, DeriveWrapSize, LayoutCommand, LayoutMeasure},
    widgets::{scope::scope, scroll_area},
};
use std::hash::Hash;

use super::{builder::BuildContext, scroll_area::ScrollAreaResponse};

pub struct VirtualListBuilder {
    id: WidgetId,
    item_size: f32,
    items_count: u64,
    size: Size,
    constraints: Constraints,
    zindex: Option<i32>,
    padding: EdgeInsets,
    margin: EdgeInsets,
    clip: Clip,
    backgrounds: SmallVec<[WidgetRef; 8]>,
    scroll_direction: ScrollDirection,
}

impl VirtualListBuilder {
    impl_id!();
    impl_size_methods!();

    pub fn item_size(mut self, size: f32) -> Self {
        self.item_size = size;

        self
    }

    pub fn items_count(mut self, count: u64) -> Self {
        self.items_count = count;

        self
    }

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
    pub fn build<F>(mut self, context: &mut BuildContext, item_build: F) -> ScrollAreaResponse
    where
        F: Fn(&mut BuildContext, u64),
    {
        let id = self.id.with_seed(context.id_seed);

        let last_zindex = context.current_zindex;
        context.current_zindex = self.zindex.unwrap_or(context.current_zindex);
        let mut widget_refs = std::mem::take(context.decorators);
        widget_refs.append(&mut self.backgrounds);

        let (offset_x, offset_y, response) = {
            let state =
                context
                    .widgets_states
                    .scroll_area
                    .get_or_insert(id, || scroll_area::State {
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

            let layout_measures = context.widgets_states.layout_measures.get_mut(id);
            let wrap_height = self.item_size as f64 * (self.items_count as f64);

            if let Some(layout_measures) = layout_measures {
                scroll_area::handle_interaction(
                    context.input,
                    state,
                    layout_measures,
                    0.,
                    wrap_height,
                );
            }

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

        let viewport_height = if response.height == 0. {
            context.view.size.height as f32
        } else {
            response.height as f32
        };

        let scroll_offset = -offset_y;

        let first_visible = (scroll_offset / self.item_size as f64).floor() as u64;
        let visible_count = (viewport_height / self.item_size).ceil() as u64 + 1;
        let last_visible = (first_visible + visible_count).min(self.items_count);
        let item_size = self.item_size as f64;

        // context.push_layout_command(LayoutCommand::Spacer {
        //     constraints: Constraints::default(),
        //     size: Size {
        //         width: SizeConstraint::Fixed(1.),
        //         height: SizeConstraint::Fixed(self.item_size * (self.items_count as f32)),
        //     },
        // });

        // for i in 0..self.items_count {
        //     context.push_layout_command(LayoutCommand::BeginOffset {
        //         offset_x: offset_x as f32,
        //         offset_y: i as f32 * self.item_size + offset_y as f32,
        //     });
        //     scope(i).build(context, |ctx| item_build(ctx, i));
        //     context.push_layout_command(LayoutCommand::EndOffset);
        // }

        for i in first_visible..last_visible {
            // Position relative to viewport top
            let relative_y = ((i - first_visible) as f64) * item_size;

            // Adjust for partial scroll (how much of first item is scrolled off)
            let first_item_offset = scroll_offset % item_size; // small number, 0 to item_size
            let final_y = relative_y - first_item_offset;

            context.push_layout_command(LayoutCommand::BeginOffset {
                offset_x: offset_x as f32,
                offset_y: final_y as f32,
            });
            scope(i).build(context, |ctx| item_build(ctx, i));
            context.push_layout_command(LayoutCommand::EndOffset);
        }

        context.push_layout_command(LayoutCommand::EndContainer);

        context.current_zindex = last_zindex;

        context
            .widgets_states
            .scroll_area
            .accessed_this_frame
            .insert(id);
        context
            .widgets_states
            .layout_measures
            .accessed_this_frame
            .insert(id);

        response
    }
}

#[track_caller]
pub fn virtual_list() -> VirtualListBuilder {
    VirtualListBuilder {
        id: WidgetId::auto(),
        size: Size::default(),
        constraints: Constraints::default(),
        zindex: None,
        padding: EdgeInsets::ZERO,
        margin: EdgeInsets::ZERO,
        scroll_direction: ScrollDirection::Vertical,
        clip: Clip::Rect,
        backgrounds: SmallVec::new(),
        item_size: 32.,
        items_count: 0,
    }
}
