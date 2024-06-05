use core::hash::Hash;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::fmt::Display;

use crate::key_handler::KeyHandlerAction::*;
use crate::key_handler::{ActionsOnTimeout, KeyHandlerAction};
use crate::types::Behaviours;
use crate::types::{Behaviour, Mapping, Mapping::Choice, Mapping::Single};
use crate::KeyPress;

type KeyHashMap<A, T> = HashMap<KeyPress, MappingTrieNode<A, T>>;

#[derive(Debug)]
enum MappingTrieNode<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Sync + Send + Hash,
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send + Hash,
{
    Root(KeyHashMap<A, T>),
    OneOff(Mapping<A, T>, KeyHashMap<A, T>),
    Repeatable(Mapping<A, T>, HashSet<KeyPress>, KeyHashMap<A, T>),
}

impl<A, T> Display for MappingTrieNode<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
    T: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Root(_) => write!(f, "Root"),
            OneOff(mapping, _) => write!(f, "OneOff({})", mapping),
            Repeatable(mapping, _, _) => write!(f, "Repeatable({})", mapping),
        }
    }
}

use MappingTrieNode::*;

pub(crate) struct MappingTrie<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
    T: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
{
    root: MappingTrieNode<A, T>,
    actions_on_timeout: Option<ActionsOnTimeout<A, T>>,
    buffer: Vec<KeyPress>,
}

impl<A, T> MappingTrie<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Sync + Send + Hash,
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send + Hash,
{
    fn all_keys_available(keys: &KeyHashMap<A, T>, key_presses: &Behaviours<A>) -> bool {
        for key_mapping in &key_presses.0 {
            if keys.contains_key(&key_mapping.get_key()) {
                return false;
            }
        }

        true
    }

    fn to_handler_action(
        &mut self,
        mapping: &Mapping<A, T>,
        key: &KeyPress,
    ) -> KeyHandlerAction<A, T>
    where
        T: PartialEq + Eq + Clone + Debug + Display + Send + Sync,
    {
        match mapping {
            Single(behaviour) => match behaviour {
                Behaviour::Timeout(_) => Timeout,
                Behaviour::Action(_, action_type) => SendActionBeforeTimeout(action_type.clone()),
                Behaviour::ActionOnTimeout(_, action_type) => {
                    SendActionOnTimeout(action_type.clone())
                }
            },
            Choice(behaviours, tag) => {
                // TODO: fix this
                let actions = self.actions_on_timeout.clone();

                match behaviours.get_mapping(key) {
                    Some(mapping) => match (mapping, actions) {
                        (Behaviour::Timeout(_), None) => Timeout,
                        (Behaviour::Timeout(_), Some(aot)) => SendActionsOnTimeout(aot.clone()),
                        (Behaviour::Action(_, action), None) => {
                            SendActionBeforeTimeout(action.clone())
                        }
                        (Behaviour::Action(_, action), Some(aot)) => {
                            SendActionBeforeTimeoutAndOnTimeout {
                                before: action.clone(),
                                on: aot.clone(),
                            }
                        }
                        (Behaviour::ActionOnTimeout(_, action), None) => {
                            self.actions_on_timeout =
                                Some(ActionsOnTimeout::new(action.clone(), tag.clone()));
                            SendActionOnTimeout(action.clone())
                        }
                        (Behaviour::ActionOnTimeout(_, action), Some(mut aot)) => {
                            aot.actions.push(action.clone());
                            SendActionsOnTimeout(aot.clone())
                        }
                    },
                    None => Nothing, // Should never happen.
                }
            }
        }
    }

    fn map(root: &mut MappingTrieNode<A, T>, mapping: &Vec<Mapping<A, T>>, starting_pos: usize) {
        let mut node = root;

        for i in starting_pos..mapping.len() {
            let m = &mapping[i];

            match m {
                Single(behaviour) => match node {
                    Root(next) | OneOff(_, next) => {
                        node = next
                            .entry(behaviour.get_key().clone())
                            .or_insert(OneOff(m.clone(), HashMap::new()));
                    }
                    Repeatable(_, _, next) => {
                        node = next
                            .entry(behaviour.get_key().clone())
                            .or_insert(OneOff(m.clone(), HashMap::new()));
                    }
                },
                Choice(behaviours, _) => match node {
                    Root(next) | OneOff(_, next) => {
                        if Self::all_keys_available(next, behaviours) {
                            behaviours.0.iter().for_each(|b| {
                                let next_node = next.entry(b.get_key().clone()).or_insert(
                                    Repeatable(m.clone(), HashSet::new(), HashMap::new()),
                                );
                                Self::map(next_node, mapping, i + 1);
                            });

                            break;
                        } else {
                            println!("Not all keys are available for mapping: {:?}", mapping);
                            break;
                        }
                    }
                    Repeatable(conflicting_mapping, _, _) => {
                        println!(
                                "Repeatable mapping conflicts with another repeatable mapping. Mapping: {}, conflicting mapping: {}",
                                m, conflicting_mapping);
                        break;
                    }
                },
            }
        }
    }

    pub fn from_mappings(mappings: &Vec<Vec<Mapping<A, T>>>) -> Self {
        let mut root: MappingTrieNode<A, T> = Root(HashMap::new());

        for mapping in mappings {
            Self::map(&mut root, mapping, 0);
        }

        Self {
            root,
            actions_on_timeout: None,
            buffer: vec![],
        }
    }

    pub fn find_mapping(&self, key: &KeyPress) -> Option<&Mapping<A, T>> {
        let mut node = &self.root;

        for key_press in &self.buffer {
            let key = key_press;

            match node {
                Root(next) | OneOff(_, next) => {
                    if !next.contains_key(key) {
                        return None;
                    } else {
                        node = &next.get(key).unwrap();
                    }
                }
                Repeatable(_, repeatable_set, next) => {
                    if !repeatable_set.contains(key) {
                        match next.get(key) {
                            None => {
                                return None;
                            }
                            Some(next_node) => {
                                node = next_node;
                            }
                        }
                    }
                }
            }
        }

        match node {
            Root(next) | OneOff(_, next) => match next.get(key) {
                None => None,
                Some(keys) => match keys {
                    Root(_) => None,
                    OneOff(mapping, _) | Repeatable(mapping, _, _) => Some(mapping),
                },
            },
            Repeatable(mapping, repeatable_set, next) => {
                if repeatable_set.contains(key) {
                    return Some(mapping);
                }

                match next.get(key) {
                    None => {
                        self.reset();
                        None
                    }
                    Some(next_node) => match next_node {
                        Root(_) => {
                            self.reset();
                            None
                        }
                        OneOff(mapping, _) | Repeatable(mapping, _, _) => Some(mapping),
                    },
                }
            }
        }
    }

    pub fn chuj(&self, key: &KeyPress) -> Option<KeyHandlerAction<A, T>> {
        let mut node = &self.root;

        for key_press in &self.buffer {
            let key = key_press;

            match node {
                Root(next) | OneOff(_, next) => {
                    if !next.contains_key(key) {
                        self.reset();
                        return None;
                    } else {
                        node = &next.get(key).unwrap();
                    }
                }
                Repeatable(_, repeatable_set, next) => {
                    if !repeatable_set.contains(key) {
                        match next.get(key) {
                            None => {
                                self.reset();
                                return None;
                            }
                            Some(next_node) => {
                                node = next_node;
                            }
                        }
                    }
                }
            }
        }

        match node {
            Root(next) | OneOff(_, next) => match next.get(key) {
                None => {
                    self.reset();
                    None
                }
                Some(keys) => match keys {
                    Root(_) => {
                        self.reset();
                        None
                    }
                    OneOff(mapping, _) => {
                        let action = self.to_handler_action(mapping, &key);
                        Some(action)
                    }
                    Repeatable(mapping, _, _) => {
                        let action = self.to_handler_action(mapping, &key);
                        Some(action)
                    }
                },
            },
            Repeatable(mapping, repeatable_set, next) => {
                if repeatable_set.contains(key) {
                    let action = self.to_handler_action(mapping, &key);
                    Some(action)
                } else {
                    match next.get(key) {
                        None => {
                            self.reset();
                            None
                        }
                        Some(next_node) => match next_node {
                            Root(_) => {
                                self.reset();
                                None
                            }
                            OneOff(mapping, _) => {
                                let action = self.to_handler_action(mapping, &key);
                                Some(action)
                            }
                            Repeatable(mapping, _, _) => {
                                let action = self.to_handler_action(mapping, &key);
                                Some(action)
                            }
                        },
                    }
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.actions_on_timeout = None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Key::*;
    use crate::types::Modifier::ModAlt;
    use crate::*;
    use rstest::rstest;

    macro_rules! m {
        ( [ $( [ $( $x:expr ),* $(,)? ] ),* $(,)? ] ) => {
            vec![
                $(
                    vec![$($x),*]
                ),*
            ]
        }
    }

    #[derive(Eq, Debug, Clone, PartialEq, Hash)]
    enum TestAction {
        VolumeUp,
        VolumeDown,
    }

    impl Display for TestAction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestAction::VolumeUp => write!(f, "VolumeUp"),
                TestAction::VolumeDown => write!(f, "VolumeDown"),
            }
        }
    }

    #[derive(Eq, Debug, Clone, PartialEq, Hash)]
    enum TestTag {
        Volume,
    }

    impl Display for TestTag {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestTag::Volume => write!(f, "Volume"),
            }
        }
    }

    macro_rules! u {
        ($user_action:expr) => {
            $crate::types::Action::User($user_action)
        };
    }
    use TestAction::*;

    macro_rules! timeout {
        () => {
            $crate::key_handler::KeyHandlerAction::Timeout
        };
    }

    macro_rules! action {
        ($action:expr, [$($keypresses:expr),* $(,)?]) => {
            $crate::key_handler::KeyHandlerAction::SendAction($crate::types::Action::User($action, vec![$($keypresses),*]))
        };
    }

    macro_rules! action_t {
        ($action:expr, [$($keypresses:expr),* $(,)?]) => {
            $crate::key_handler::KeyHandlerAction::SendActionOnTimeout($crate::types::Action::User($action, vec![$($keypresses),*]))
        };
    }

    macro_rules! t_action {
        ($action:expr, [$($keypresses:expr),* $(,)?]) => {
            $crate::key_handler::KeyHandlerAction::SendActionBeforeTimeout($crate::types::Action::User($action, vec![$($keypresses),*]))
        };
    }

    #[rstest]
    #[case(m!([[t!(KeyA)]]), &[key!(KeyX)], None)]
    // #[case(m!([[t!(KeyA, ModAlt)]]), &[alt!(KeyA)], Some(timeout!()))]
    // #[case(m!([[a!(KeyA, ModAlt, u!(VolumeUp))]]), &[alt!(KeyA)], Some(action!(VolumeUp, [alt!(KeyA)])))]
    // #[case(m!([[t!(KeyA), t!(Key1)]]), &[key!(KeyA), key!(Key1)], Some(timeout!()))]
    // #[case(m!([[t!(KeyA), t!(Key1)]]), &[key!(Key2)], None)]
    // #[case(m!([[t!(KeyA), t!(Key1), a!(Key2, u!(VolumeUp))]]), &[key!(KeyA), key!(Key2), key!(Key2)], None)]
    // #[case(m!([[t!(KeyA), t!(Key1), a!(Key2, u!(VolumeUp))]]), &[key!(KeyA), key!(Key1), key!(Key2)], Some(action!(VolumeUp, [key!(KeyA), key!(Key1), key!(Key2)])))]
    // #[case(m!([[aot!([key!(Key1)], u!(VolumeUp))]]), &[key!(Key1)], Some(action_t!(VolumeUp, [key!(Key1)])))]
    // #[case(m!([[aot!([key!(Key1)], u!(VolumeUp))]]), &[key!(Key1)], Some(action_t!(VolumeUp, [key!(Key1)])))]
    // #[case(m!([[aot!([key!(Key1), key!(Key2)], u!(VolumeUp))]]), &[key!(Key1)], Some(action_t!(VolumeUp, [key!(Key1)])))]
    // #[case(m!([[aot!([key!(Key1), key!(Key2)], u!(VolumeUp))]]), &[key!(Key2)], Some(action_t!(VolumeUp, [key!(Key2)])))]
    // #[case(m!([[aot!([key!(Key1), key!(Key2)], u!(VolumeUp))]]), &[key!(Key2)], Some(action_t!(VolumeUp, [key!(Key2)])))]

    // #[case(m!([[abt!([key!(Key1)], u!(VolumeUp))]]), &[key!(Key1)], Some(t_action!(VolumeUp, [key!(Key1)])))]
    // #[case(m!([[abt!([key!(Key1)], u!(VolumeUp))]]), &[key!(Key1)], Some(t_action!(VolumeUp, [key!(Key1)])))]
    // #[case(m!([[abt!([key!(Key1), key!(Key2)], u!(VolumeUp))]]), &[key!(Key1)], Some(t_action!(VolumeUp, [key!(Key1)])))]
    // #[case(m!([[abt!([key!(Key1), key!(Key2)], u!(VolumeUp))]]), &[key!(Key2)], Some(t_action!(VolumeUp, [key!(Key2)])))]
    // #[case(m!([[abt!([key!(Key1), key!(Key2)], u!(VolumeUp))]]), &[key!(Key2)], Some(t_action!(VolumeUp, [key!(Key2)])))]

    // #[case(m!([[t!(KeyA), a!(KeyX, u!(VolumeUp))], [t!(KeyA), aot!([key!(Key1), key!(Key2)], u!(VolumeUp))]]), &[key!(KeyA), key!(Key1)], Some(action_t!(VolumeUp, [key!(KeyA), key!(Key1)])))]
    // #[case(m!([[t!(KeyA), a!(KeyX, u!(VolumeUp))], [t!(KeyA), aot!([key!(Key1), key!(Key2)], u!(VolumeUp))]]), &[key!(KeyA), key!(Key1), key!(Key3)], None)]
    // #[case(m!([[t!(KeyA), a!(KeyX, u!(VolumeUp))], [t!(KeyA), aot!([key!(Key1), key!(Key2)], u!(VolumeUp))]]), &[key!(KeyA), key!(Key3)], None)]
    // #[case(m!([[t!(KeyA), t!([key!(KeyB)]), a!(KeyX, u!(VolumeDown))]]), &[key!(KeyA), key!(KeyB), key!(KeyB), key!(KeyX)], Some(action!(VolumeDown, [key!(KeyA), key!(KeyB), key!(KeyB), key!(KeyX)])))]
    // #[case(m!([[t!(KeyA), t!([key!(KeyB)]), t!(KeyX), t!([key!(KeyC)]), a!(KeyX, u!(VolumeDown))]]), &[key!(KeyA), key!(KeyB), key!(KeyB), key!(KeyX), key!(KeyC), key!(KeyC), key!(KeyX)], Some(action!(VolumeDown, [key!(KeyA), key!(KeyB), key!(KeyB), key!(KeyX), key!(KeyC), key!(KeyC), key!(KeyX)])))]
    // a choice of only timeouts
    // a choice of timeouts and actions
    // a choice of actions and actions after timeouts
    // a choice of timeouts, actions and actions after timeouts
    fn should_match_keys_to_mappings(
        #[case] mappings: Vec<Vec<Mapping<TestAction, TestTag>>>,
        #[case] keypresses: &[KeyPress],
        #[case] expected: Option<KeyHandlerAction<TestAction, TestTag>>,
    ) {
        // Given
        let mut trie = MappingTrie::from_mappings(&mappings);
        let mut result = None;

        // When
        for key in keypresses {
            result = trie.find_mapping(key);
        }

        // Then
        assert_eq!(result, expected)
    }
}
