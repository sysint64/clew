use std::time::{Duration, Instant};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::keyboard::{KeyCode, KeyModifiers};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct KeyBinding {
    modifiers: KeyModifiers,
    key: KeyCode,
}

#[derive(Debug, Eq, PartialEq)]
struct ShortcutConfig {
    sequence: Vec<KeyBinding>,
    repeat: bool,
}

fn remove_modifiers(sequence: &[KeyBinding], modifiers: KeyModifiers) -> Vec<KeyBinding> {
    if modifiers.is_empty() {
        sequence.to_vec()
    } else {
        sequence
            .iter()
            .map(|binding| KeyBinding {
                modifiers: binding.modifiers & !modifiers,
                key: binding.key,
            })
            .collect()
    }
}

impl KeyBinding {
    pub fn new(key: KeyCode) -> Self {
        Self {
            modifiers: KeyModifiers::empty(),
            key,
        }
    }

    pub fn with_ctrl(mut self) -> Self {
        self.modifiers |= KeyModifiers::CONTROL;

        self
    }

    pub fn with_shift(mut self) -> Self {
        self.modifiers |= KeyModifiers::SHIFT;

        self
    }

    pub fn with_alt(mut self) -> Self {
        self.modifiers |= KeyModifiers::ALT;

        self
    }

    pub fn with_super(mut self) -> Self {
        self.modifiers |= KeyModifiers::SUPER;

        self
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ShortcutScopeId(pub &'static str);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ShortcutModifierId(pub &'static str);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ShortcutId(pub &'static str);

#[derive(Default)]
pub struct ShortcutRegistry {
    scopes: FxHashMap<ShortcutScopeId, ShortcutScope>,
}

#[derive(Default)]
pub struct ShortcutScope {
    shortcuts: FxHashMap<ShortcutId, ShortcutConfig>,
    modifiers: FxHashMap<ShortcutModifierId, KeyModifiers>,
}

impl ShortcutRegistry {
    pub fn scope<T: Into<ShortcutScopeId>>(&mut self, key: T) -> &mut ShortcutScope {
        let scope = ShortcutScope::default();
        let key = key.into();

        if self.scopes.contains_key(&key) {
            panic!("Scope with id = {key:?} already exists");
        }

        self.scopes.insert(key, scope);

        self.scopes.get_mut(&key).unwrap()
    }
}

impl ShortcutScope {
    pub fn add<T: Into<ShortcutId>>(&mut self, id: T, shortcut: KeyBinding) -> &mut ShortcutScope {
        let key = id.into();

        if self.shortcuts.contains_key(&key) {
            panic!("Shortcut with id = {key:?} already exists");
        }

        self.shortcuts.insert(
            key,
            ShortcutConfig {
                sequence: vec![shortcut],
                repeat: false,
            },
        );

        self
    }

    pub fn add_repeat<T: Into<ShortcutId>>(
        &mut self,
        id: T,
        shortcut: KeyBinding,
    ) -> &mut ShortcutScope {
        let key = id.into();

        if self.shortcuts.contains_key(&key) {
            panic!("Shortcut with id = {key:?} already exists");
        }

        self.shortcuts.insert(
            key,
            ShortcutConfig {
                sequence: vec![shortcut],
                repeat: true,
            },
        );

        self
    }

    pub fn add_sequence<T: Into<ShortcutId>>(
        &mut self,
        id: T,
        sequence: &[KeyBinding],
    ) -> &mut ShortcutScope {
        let key = id.into();

        if self.shortcuts.contains_key(&key) {
            panic!("Shortcut with id = {key:?} already exists");
        }

        self.shortcuts.insert(
            key,
            ShortcutConfig {
                sequence: Vec::from(sequence),
                repeat: true,
            },
        );

        self
    }

    pub fn add_modifier<T: Into<ShortcutModifierId>>(
        &mut self,
        id: T,
        modifier: KeyModifiers,
    ) -> &mut ShortcutScope {
        let key = id.into();

        if self.modifiers.contains_key(&key) {
            panic!("Shortcut with id = {key:?} already exists");
        }

        self.modifiers.insert(key, modifier);

        self
    }
}

pub struct ShortcutManager {
    last_sequence: Vec<KeyBinding>,
    last_found_candidate: Option<Instant>,
    chord_timeout: Duration,

    pub(crate) scopes: Vec<ShortcutScopeId>,
    pub(crate) shortcut_id: Option<ShortcutId>,
    pub(crate) modifiers: FxHashSet<ShortcutModifierId>,
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self {
            chord_timeout: Duration::from_secs(2),
            last_found_candidate: Default::default(),
            scopes: Default::default(),
            shortcut_id: Default::default(),
            modifiers: Default::default(),
            last_sequence: Default::default(),
        }
    }
}

impl ShortcutManager {
    pub fn is_shortcut<T: Into<ShortcutId>>(&self, id: T) -> bool {
        self.shortcut_id == Some(id.into())
    }

    pub fn has_modifier<T: Into<ShortcutModifierId>>(&self, id: T) -> bool {
        self.modifiers.contains(&id.into())
    }

    pub fn reset(&mut self) {
        self.scopes.clear();
        self.shortcut_id = None;
        self.modifiers.clear();
    }

    pub(crate) fn push_scope<T: Into<ShortcutScopeId>>(&mut self, scope: T) {
        self.scopes.push(scope.into());
    }

    pub fn on_key_binding_activate(
        &mut self,
        registry: &ShortcutRegistry,
        modifiers: KeyModifiers,
        key: Option<KeyCode>,
        repeat: bool,
    ) {
        if let Some(time) = self.last_found_candidate {
            let duration = time.elapsed();

            if duration > self.chord_timeout {
                self.last_sequence.clear();
            }
        } else {
            self.last_sequence.clear();
        }

        if let Some(key) = key {
            self.last_sequence.push(KeyBinding { modifiers, key });
        }

        let (candidates, shortcut_id) = Self::resolve(
            registry,
            modifiers,
            &self.scopes,
            &mut self.modifiers,
            &self.last_sequence,
            repeat,
        );

        self.shortcut_id = shortcut_id;

        if shortcut_id.is_none() && candidates == 0 {
            self.last_sequence.clear();
            self.last_found_candidate = None;
        } else if shortcut_id.is_none() && candidates > 0 {
            self.last_found_candidate = Some(Instant::now());
        } else {
            self.last_sequence.clear();
        }
    }

    pub(crate) fn resolve(
        registry: &ShortcutRegistry,
        modifiers: KeyModifiers,
        scopes: &[ShortcutScopeId],
        shortucts_modifiers: &mut FxHashSet<ShortcutModifierId>,
        chords: &[KeyBinding],
        repeat: bool,
    ) -> (usize, Option<ShortcutId>) {
        let mut shortcut_id = None;
        let mut candidates = 0;
        let mut resolved_modifiers = KeyModifiers::empty();

        // Resolve modifier
        for scope in scopes.iter().rev() {
            let scope = registry
                .scopes
                .get(scope)
                .unwrap_or_else(|| panic!("Scope with id = {scope:?} doesn't exists"));

            for (id, scope_modifiers) in scope.modifiers.iter() {
                if *scope_modifiers & modifiers == *scope_modifiers {
                    shortucts_modifiers.insert(*id);
                    resolved_modifiers |= *scope_modifiers;
                }
            }
        }

        // Resolve keybinding
        for scope in scopes.iter().rev() {
            let scope = registry
                .scopes
                .get(scope)
                .unwrap_or_else(|| panic!("Scope with id = {scope:?} doesn't exists"));

            for (id, key_bindings) in scope.shortcuts.iter() {
                if repeat && repeat != key_bindings.repeat {
                    continue;
                }

                if key_bindings.sequence == chords {
                    shortcut_id = Some(*id);
                    break;
                }

                if key_bindings.sequence.starts_with(chords) {
                    candidates += 1;
                    break;
                }

                let chords = remove_modifiers(chords, resolved_modifiers);

                if key_bindings.sequence == chords {
                    shortcut_id = Some(*id);
                    break;
                }

                if key_bindings.sequence.starts_with(&chords) {
                    candidates += 1;
                }
            }

            if shortcut_id.is_some() {
                break;
            }
        }

        (candidates, shortcut_id)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     impl Into<ShortcutId> for &'static str {
//         fn into(self) -> ShortcutId {
//             ShortcutId(self)
//         }
//     }

//     impl Into<ShortcutModifierId> for &'static str {
//         fn into(self) -> ShortcutModifierId {
//             ShortcutModifierId(self)
//         }
//     }

//     impl Into<ShortcutScopeId> for &'static str {
//         fn into(self) -> ShortcutScopeId {
//             ShortcutScopeId(self)
//         }
//     }

//     fn mock_registry() -> ShortcutRegistry {
//         let mut registry = ShortcutRegistry::default();
//         registry
//             .scope("scope1")
//             .add("shorcut1", KeyBinding::new(KeyCode::KeyA).with_ctrl())
//             .add(
//                 "shorcut2",
//                 KeyBinding::new(KeyCode::KeyB).with_ctrl().with_shift(),
//             )
//             .add("shorcut3", KeyBinding::new(KeyCode::KeyC))
//             .add("shorcut4", KeyBinding::new(KeyCode::KeyB))
//             .add_repeat("shortcut_repeat", KeyBinding::new(KeyCode::ArrowLeft))
//             .add_modifier("modifier1", KeyModifiers::alt())
//             .add_modifier("modifier2", KeyModifiers::ctrl());

//         registry
//             .scope("scope2")
//             .add(
//                 "shorcut5",
//                 KeyBinding::new(KeyCode::KeyB).with_ctrl().with_shift(),
//             )
//             .add("shorcut6", KeyBinding::new(KeyCode::KeyD))
//             .add("shorcut7", KeyBinding::new(KeyCode::KeyE))
//             .add_sequence(
//                 "sequence1",
//                 &[
//                     KeyBinding::new(KeyCode::KeyF).with_ctrl(),
//                     KeyBinding::new(KeyCode::KeyG).with_ctrl(),
//                 ],
//             )
//             .add_sequence(
//                 "sequence2",
//                 &[
//                     KeyBinding::new(KeyCode::KeyF).with_ctrl(),
//                     KeyBinding::new(KeyCode::KeyI).with_ctrl(),
//                 ],
//             )
//             .add_sequence(
//                 "sequence3",
//                 &[
//                     KeyBinding::new(KeyCode::KeyF).with_ctrl(),
//                     KeyBinding::new(KeyCode::KeyK).with_ctrl(),
//                 ],
//             )
//             .add_modifier("modifier3", KeyModifiers::alt())
//             .add_modifier("modifier4", KeyModifiers::shift());

//         registry
//     }

//     #[test]
//     fn test_simple_resolve() {
//         let registry = mock_registry();
//         let mut modifiers = FxHashSet::default();

//         let (_, shortcut_id) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::empty(),
//             &["scope1".into()],
//             &mut modifiers,
//             &[KeyBinding::new(KeyCode::KeyB)],
//             false,
//         );

//         assert_eq!(shortcut_id, Some("shorcut4".into()));
//         assert_eq!(modifiers.is_empty(), true);
//     }

//     #[test]
//     fn test_repeat_resolve_fail_when_non_repeatable() {
//         let registry = mock_registry();
//         let mut modifiers = FxHashSet::default();

//         let (_, shortcut_id) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::empty(),
//             &["scope1".into()],
//             &mut modifiers,
//             &[KeyBinding::new(KeyCode::KeyB)],
//             false,
//         );

//         assert_eq!(shortcut_id, Some("shorcut4".into()));
//         assert_eq!(modifiers.is_empty(), true);

//         let (_, shortcut_id) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::empty(),
//             &["scope1".into()],
//             &mut modifiers,
//             &[KeyBinding::new(KeyCode::KeyB)],
//             true,
//         );

//         assert_eq!(shortcut_id, None);
//         assert_eq!(modifiers.is_empty(), true);
//     }

//     #[test]
//     fn test_repeat_resolve_success_when_repeatable() {
//         let registry = mock_registry();
//         let mut modifiers = FxHashSet::default();

//         let (_, shortcut_id) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::empty(),
//             &["scope1".into()],
//             &mut modifiers,
//             &[KeyBinding::new(KeyCode::ArrowLeft)],
//             false,
//         );

//         assert_eq!(shortcut_id, Some("shortcut_repeat".into()));
//         assert_eq!(modifiers.is_empty(), true);

//         let (_, shortcut_id) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::empty(),
//             &["scope1".into()],
//             &mut modifiers,
//             &[KeyBinding::new(KeyCode::ArrowLeft)],
//             true,
//         );

//         assert_eq!(shortcut_id, Some("shortcut_repeat".into()));
//         assert_eq!(modifiers.is_empty(), true);
//     }

//     #[test]
//     fn test_modifier_resolve_mix() {
//         let registry = mock_registry();
//         let mut modifiers = FxHashSet::default();

//         let (_, _) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::alt().with_ctrl(),
//             &["scope1".into()],
//             &mut modifiers,
//             &[],
//             false,
//         );
//         assert_eq!(modifiers.contains(&"modifier1".into()), true);
//         assert_eq!(modifiers.contains(&"modifier2".into()), true);
//     }

//     #[test]
//     fn test_modifier_resolve_mix_with_shortcut() {
//         let registry = mock_registry();
//         let mut modifiers = FxHashSet::default();

//         let (_, shortcut_id) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::alt().with_ctrl(),
//             &["scope1".into()],
//             &mut modifiers,
//             &[KeyBinding::new(KeyCode::KeyC).with_ctrl().with_alt()],
//             false,
//         );

//         assert_eq!(modifiers.contains(&"modifier1".into()), true);
//         assert_eq!(modifiers.contains(&"modifier2".into()), true);
//         assert_eq!(shortcut_id, Some("shorcut3".into()));
//     }

//     #[test]
//     fn test_modifier_resolve() {
//         let registry = mock_registry();
//         let mut modifiers = FxHashSet::default();

//         let (_, _) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::alt(),
//             &["scope1".into()],
//             &mut modifiers,
//             &[],
//             false,
//         );
//         assert_eq!(modifiers.contains(&"modifier1".into()), true);
//         assert_eq!(modifiers.contains(&"modifier3".into()), false);

//         modifiers.clear();

//         let (_, _) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::alt(),
//             &["scope2".into()],
//             &mut modifiers,
//             &[],
//             false,
//         );
//         assert_eq!(modifiers.contains(&"modifier1".into()), false);
//         assert_eq!(modifiers.contains(&"modifier3".into()), true);

//         modifiers.clear();

//         let (_, _) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::alt(),
//             &["scope1".into(), "scope2".into()],
//             &mut modifiers,
//             &[],
//             false,
//         );
//         assert_eq!(modifiers.contains(&"modifier1".into()), true);
//         assert_eq!(modifiers.contains(&"modifier3".into()), true);
//     }

//     #[test]
//     fn test_modifiers_intersection_resolve() {
//         let registry = mock_registry();
//         let mut modifiers = FxHashSet::default();

//         let (_, _) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::ctrl().with_alt(),
//             &["scope1".into()],
//             &mut modifiers,
//             &[],
//             false,
//         );
//         assert_eq!(modifiers.contains(&"modifier1".into()), true);
//         assert_eq!(modifiers.contains(&"modifier2".into()), true);

//         modifiers.clear();
//     }

//     #[test]
//     #[ignore]
//     fn test_sequence_candidates_resolve() {
//         let registry = mock_registry();
//         let mut modifiers = FxHashSet::default();

//         let (candidates, shortcut_id) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::empty(),
//             &["scope2".into()],
//             &mut modifiers,
//             &[KeyBinding::new(KeyCode::KeyF).with_ctrl()],
//             false,
//         );

//         assert_eq!(candidates, 3);
//         assert_eq!(shortcut_id, None);
//         assert_eq!(modifiers.is_empty(), true);
//     }

//     #[test]
//     fn test_sequence_resolve() {
//         let registry = mock_registry();
//         let mut modifiers = FxHashSet::default();

//         let (_, shortcut_id) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::empty(),
//             &["scope2".into()],
//             &mut modifiers,
//             &[
//                 KeyBinding::new(KeyCode::KeyF).with_ctrl(),
//                 KeyBinding::new(KeyCode::KeyG).with_ctrl(),
//             ],
//             false,
//         );

//         assert_eq!(shortcut_id, Some("sequence1".into()));
//         assert_eq!(modifiers.is_empty(), true);

//         let (_, shortcut_id) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::empty(),
//             &["scope2".into()],
//             &mut modifiers,
//             &[
//                 KeyBinding::new(KeyCode::KeyF).with_ctrl(),
//                 KeyBinding::new(KeyCode::KeyI).with_ctrl(),
//             ],
//             false,
//         );

//         assert_eq!(shortcut_id, Some("sequence2".into()));
//         assert_eq!(modifiers.is_empty(), true);

//         let (_, shortcut_id) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::empty(),
//             &["scope2".into()],
//             &mut modifiers,
//             &[
//                 KeyBinding::new(KeyCode::KeyF).with_ctrl(),
//                 KeyBinding::new(KeyCode::KeyK).with_ctrl(),
//             ],
//             false,
//         );

//         assert_eq!(shortcut_id, Some("sequence3".into()));
//         assert_eq!(modifiers.is_empty(), true);
//     }

//     #[test]
//     fn test_scopes_resolve() {
//         let registry = mock_registry();
//         let mut modifiers = FxHashSet::default();

//         let (_, shortcut_id) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::alt(),
//             &["scope1".into(), "scope2".into()],
//             &mut modifiers,
//             &[KeyBinding::new(KeyCode::KeyB).with_ctrl().with_shift()],
//             false,
//         );

//         assert_eq!(shortcut_id, Some("shorcut5".into()));
//         assert_eq!(modifiers.contains(&"modifier1".into()), true);
//         assert_eq!(modifiers.contains(&"modifier3".into()), true);

//         modifiers.clear();

//         let (_, shortcut_id) = ShortcutManager::resolve(
//             &registry,
//             KeyModifiers::alt(),
//             &["scope2".into(), "scope1".into()],
//             &mut modifiers,
//             &[KeyBinding::new(KeyCode::KeyB).with_ctrl().with_shift()],
//             false,
//         );

//         assert_eq!(shortcut_id, Some("shorcut2".into()));
//         assert_eq!(modifiers.contains(&"modifier1".into()), true);
//         assert_eq!(modifiers.contains(&"modifier3".into()), true);
//     }

//     #[test]
//     fn test_on_key_binding_activate_mix_with_shortcut() {
//         let registry = mock_registry();
//         let mut manager = ShortcutManager::default();

//         manager.push_scope("scope1");
//         manager.on_key_binding_activate(
//             &registry,
//             KeyModifiers::alt().with_ctrl(),
//             Some(KeyCode::KeyC),
//             false,
//         );

//         assert_eq!(manager.last_sequence, &[]);
//         assert_eq!(manager.modifiers.contains(&"modifier1".into()), true);
//         assert_eq!(manager.modifiers.contains(&"modifier2".into()), true);
//         assert_eq!(manager.shortcut_id, Some("shorcut3".into()));
//     }

//     #[test]
//     fn test_on_key_binding_activate_simple() {
//         let registry = mock_registry();
//         let mut manager = ShortcutManager::default();

//         manager.push_scope("scope1");
//         manager.on_key_binding_activate(
//             &registry,
//             KeyModifiers::empty(),
//             Some(KeyCode::KeyB),
//             false,
//         );

//         assert_eq!(manager.last_sequence, &[]);
//         assert_eq!(manager.shortcut_id, Some("shorcut4".into()));
//         assert_eq!(manager.modifiers.is_empty(), true);
//     }

//     #[test]
//     fn test_on_key_binding_activate_empty_scopes() {
//         let registry = mock_registry();
//         let mut manager = ShortcutManager::default();

//         manager.on_key_binding_activate(
//             &registry,
//             KeyModifiers::empty(),
//             Some(KeyCode::KeyB),
//             false,
//         );

//         assert_eq!(manager.last_sequence, &[]);
//         assert_eq!(manager.shortcut_id, None);
//         assert_eq!(manager.modifiers.is_empty(), true);
//     }

//     #[test]
//     fn test_on_key_binding_activate_repeat() {
//         let registry = mock_registry();
//         let mut manager = ShortcutManager::default();

//         manager.push_scope("scope1");
//         manager.on_key_binding_activate(
//             &registry,
//             KeyModifiers::empty(),
//             Some(KeyCode::KeyB),
//             false,
//         );

//         assert_eq!(manager.last_sequence, &[]);
//         assert_eq!(manager.shortcut_id, Some("shorcut4".into()));
//         assert_eq!(manager.modifiers.is_empty(), true);

//         manager.reset();
//         manager.push_scope("scope1");
//         manager.on_key_binding_activate(
//             &registry,
//             KeyModifiers::empty(),
//             Some(KeyCode::KeyB),
//             false,
//         );

//         assert_eq!(manager.last_sequence, &[]);
//         assert_eq!(manager.shortcut_id, Some("shorcut4".into()));
//         assert_eq!(manager.modifiers.is_empty(), true);
//     }

//     #[test]
//     fn test_on_key_binding_activate_sequence() {
//         let registry = mock_registry();
//         let mut manager = ShortcutManager::default();

//         manager.push_scope("scope2");
//         manager.on_key_binding_activate(
//             &registry,
//             KeyModifiers::ctrl(),
//             Some(KeyCode::KeyF),
//             false,
//         );

//         assert_eq!(
//             manager.last_sequence,
//             &[KeyBinding::new(KeyCode::KeyF).with_ctrl()]
//         );
//         assert_eq!(manager.shortcut_id, None);
//         assert_eq!(manager.modifiers.is_empty(), true);

//         manager.reset();
//         manager.push_scope("scope2");
//         manager.on_key_binding_activate(
//             &registry,
//             KeyModifiers::ctrl(),
//             Some(KeyCode::KeyG),
//             false,
//         );

//         assert_eq!(manager.last_sequence, &[]);
//         assert_eq!(manager.shortcut_id, Some("sequence1".into()));
//         assert_eq!(manager.modifiers.is_empty(), true);
//     }

//     #[test]
//     fn test_on_key_binding_activate_sequence_timeout() {
//         let registry = mock_registry();
//         let mut manager = ShortcutManager::default();

//         manager.push_scope("scope2");
//         manager.on_key_binding_activate(
//             &registry,
//             KeyModifiers::ctrl(),
//             Some(KeyCode::KeyF),
//             false,
//         );

//         assert_eq!(
//             manager.last_sequence,
//             &[KeyBinding::new(KeyCode::KeyF).with_ctrl()]
//         );
//         assert_eq!(manager.shortcut_id, None);
//         assert_eq!(manager.modifiers.is_empty(), true);

//         manager.reset();
//         manager.last_found_candidate = None;
//         manager.push_scope("scope2");
//         manager.on_key_binding_activate(
//             &registry,
//             KeyModifiers::ctrl(),
//             Some(KeyCode::KeyG),
//             false,
//         );

//         assert_eq!(manager.last_sequence, &[]);
//         assert_eq!(manager.shortcut_id, None);
//         assert_eq!(manager.modifiers.is_empty(), true);
//     }

//     #[test]
//     fn test_on_key_binding_activate_sequence_repeat_after_timeout() {
//         let registry = mock_registry();
//         let mut manager = ShortcutManager::default();

//         manager.push_scope("scope2");
//         manager.on_key_binding_activate(
//             &registry,
//             KeyModifiers::ctrl(),
//             Some(KeyCode::KeyF),
//             false,
//         );

//         assert_eq!(
//             manager.last_sequence,
//             &[KeyBinding::new(KeyCode::KeyF).with_ctrl()]
//         );
//         assert_eq!(manager.shortcut_id, None);
//         assert_eq!(manager.modifiers.is_empty(), true);

//         manager.reset();
//         manager.last_found_candidate = None;

//         manager.push_scope("scope2");
//         manager.on_key_binding_activate(
//             &registry,
//             KeyModifiers::ctrl(),
//             Some(KeyCode::KeyF),
//             false,
//         );

//         assert_eq!(
//             manager.last_sequence,
//             &[KeyBinding::new(KeyCode::KeyF).with_ctrl()]
//         );
//         assert_eq!(manager.shortcut_id, None);
//         assert_eq!(manager.modifiers.is_empty(), true);

//         manager.reset();
//         manager.push_scope("scope2");
//         manager.on_key_binding_activate(
//             &registry,
//             KeyModifiers::ctrl(),
//             Some(KeyCode::KeyG),
//             false,
//         );

//         assert_eq!(manager.last_sequence, &[]);
//         assert_eq!(manager.shortcut_id, Some("sequence1".into()));
//         assert_eq!(manager.modifiers.is_empty(), true);
//     }
// }
