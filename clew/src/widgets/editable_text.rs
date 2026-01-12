use std::time::Instant;

use clew_derive::WidgetBuilder;

use crate::{
    Vec2,
    text::TextId,
    text_history::{TextEditDelta, TextEditHistoryManager},
};

use super::FrameBuilder;

pub struct EditableTextWidget;

#[derive(WidgetBuilder)]
pub struct EditableTextBuilder {
    frame: FrameBuilder,
}

#[derive(Clone, PartialEq)]
pub(crate) enum EditableTextDelta {
    Undo(TextEditDelta),
    Apply(TextEditDelta),
}

#[derive(Clone, PartialEq)]
pub struct State {
    pub(crate) text_id: Option<TextId>,
    pub(crate) deltas: Vec<EditableTextDelta>,
    pub(crate) save_history: bool,
    pub(crate) scroll_x: f32,
    pub(crate) auto_scroll_to_cursor: bool,
    pub(crate) reached_end: bool,
    pub(crate) was_relayout: bool,
    pub(crate) recompose_text_content: bool,
    pub(crate) last_boundary_size: Vec2,
    pub(crate) ime_cursor_end: cosmic_text::Cursor,
    pub(crate) direction_decided: bool,
    pub(crate) text_offset: Vec2,
    pub(crate) history_manager: TextEditHistoryManager,
    pub(crate) multi_line: bool,
    pub(crate) auto_rtl: bool,
    pub(crate) visible_view_updated: bool,
    pub(crate) last_mouse_x: f64,
    pub(crate) last_mouse_y: f64,
    pub(crate) mouse_path_x: f32,
    pub(crate) mouse_path_y: f32,
    pub(crate) last_drag: Option<Instant>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            text_id: None,
            save_history: true,
            auto_scroll_to_cursor: false,
            reached_end: false,
            was_relayout: false,
            visible_view_updated: false,
            scroll_x: 0.0,
            recompose_text_content: true,
            ime_cursor_end: cosmic_text::Cursor::default(),
            direction_decided: false,
            text_offset: Vec2::ZERO,
            history_manager: TextEditHistoryManager::new(20, true),
            multi_line: true,
            auto_rtl: false,
            last_boundary_size: Vec2::ZERO,
            last_mouse_x: 0.,
            last_mouse_y: 0.,
            mouse_path_x: 0.,
            mouse_path_y: 0.,
            last_drag: None,
            deltas: vec![],
        }
    }
}
