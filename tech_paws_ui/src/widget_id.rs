use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
    pub seed: Option<u64>,
}

impl WidgetId {
    #[track_caller]
    pub(crate) fn auto_with_seed(seed: impl Hash) -> Self {
        let location = std::panic::Location::caller();

        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);

        Self {
            file: location.file(),
            line: location.line(),
            column: location.column(),
            seed: Some(hasher.finish()),
        }
    }

    #[track_caller]
    pub(crate) fn _auto() -> Self {
        let location = std::panic::Location::caller();

        Self {
            file: location.file(),
            line: location.line(),
            column: location.column(),
            seed: None,
        }
    }
}

pub struct LayoutWidget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WidgetType {
    type_id: std::any::TypeId,
    name: &'static str,
}

impl WidgetType {
    pub fn of<T: 'static>() -> Self {
        Self {
            type_id: std::any::TypeId::of::<T>(),
            name: std::any::type_name::<T>(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WidgetRef {
    pub widget_type: WidgetType,
    pub id: WidgetId,
}

impl WidgetRef {
    pub(crate) fn new(widget_type: WidgetType, id: WidgetId) -> Self {
        Self { widget_type, id }
    }
}
