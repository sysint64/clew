use std::hash::{DefaultHasher, Hash, Hasher};

use super::builder::BuildContext;

pub struct ScopeBuilder {
    key: u64,
}

impl ScopeBuilder {
    pub fn build<F, T>(&self, context: &mut BuildContext, callback: F) -> T
    where
        F: FnOnce(&mut BuildContext) -> T,
    {
        context.with_id_seed(self.key, callback)
    }
}

pub fn scope(key: impl Hash) -> ScopeBuilder {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);

    ScopeBuilder {
        key: hasher.finish(),
    }
}
