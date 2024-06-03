use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::fmt::Display;

use crate::key_handler::KeyHandlerAction;
use crate::key_handler::KeyHandlerAction::*;
use crate::types::ActionMapping::*;
use crate::types::ActionType::*;
use crate::types::{Action, KeyPresses};
use crate::types::{KeyPress, KeyPressType::Choice, KeyPressType::Single, Mapping};

type KeyHashMap<T> = HashMap<KeyPress, MappingTrieNode<T>>;

#[derive(Debug)]
enum MappingTrieNode<T>
where
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    Root(KeyHashMap<T>),
    OneOff(Mapping<T>, KeyHashMap<T>),
    Repeatable(Mapping<T>, HashSet<KeyPress>, KeyHashMap<T>),
}

impl<T> Display for MappingTrieNode<T>
where
    T: PartialEq + Eq + Clone + Debug + Display + Send + Sync,
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

pub(crate) struct MappingTrie<T>
where
    T: PartialEq + Eq + Clone + Debug + Display + Send + Sync,
{
    root: MappingTrieNode<T>,
    buffer: Vec<KeyPress>,
}

fn to_handler_action<T>(mapping: &Mapping<T>, keys: &[KeyPress]) -> KeyHandlerAction<T>
where
    T: PartialEq + Eq + Clone + Debug + Display + Send + Sync,
{
    match mapping {
        Mapping::Timeout(_) => Timeout,
        Mapping::Action(_, action_mapping) => match action_mapping {
            NoTimeout(action_type) => match action_type {
                System(system_action_type) => {
                    SendAction(Action::System(system_action_type.clone()))
                }
                User(user_action_type) => {
                    SendAction(Action::User(user_action_type.clone(), keys.to_vec()))
                }
            },
            TimeoutAfterAction(action_type) => match action_type {
                System(system_action_type) => {
                    SendActionBeforeTimeout(Action::System(system_action_type.clone()))
                }
                User(user_action_type) => {
                    SendActionBeforeTimeout(Action::User(user_action_type.clone(), keys.to_vec()))
                }
            },
            TimeoutBeforeAction(action_type) => match action_type {
                System(system_action_type) => {
                    SendActionOnTimeout(Action::System(system_action_type.clone()))
                }
                User(user_action_type) => {
                    SendActionOnTimeout(Action::User(user_action_type.clone(), keys.to_vec()))
                }
            },
        },
    }
}

impl<T> MappingTrie<T>
where
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    fn all_keys_available(keys: &KeyHashMap<T>, key_presses: &KeyPresses) -> bool {
        for key_press in &key_presses.0 {
            if keys.contains_key(key_press) {
                return false;
            }
        }

        true
    }

    fn map(root: &mut MappingTrieNode<T>, mapping: &Vec<Mapping<T>>, starting_pos: usize) {
        let mut node = root;

        for i in starting_pos..mapping.len() {
            let m = &mapping[i];
            let key = m.get_key();

            match key {
                Single(key_press) => match node {
                    Root(next) | OneOff(_, next) => {
                        node = next
                            .entry(key_press.clone())
                            .or_insert(OneOff(m.clone(), HashMap::new()));
                    }
                    Repeatable(_, _, next) => {
                        node = next
                            .entry(key_press.clone())
                            .or_insert(OneOff(m.clone(), HashMap::new()));
                    }
                },
                Choice(key_presses) => match node {
                    Root(next) | OneOff(_, next) => {
                        if Self::all_keys_available(next, key_presses) {
                            for key_press in key_presses.clone().0 {
                                let set = HashSet::from_iter(key_presses.clone().0.into_iter());
                                let next_node = next
                                    .entry(key_press.clone())
                                    .or_insert(Repeatable(m.clone(), set, HashMap::new()));
                                Self::map(next_node, mapping, i + 1);
                            }

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

    pub fn from_mappings(mappings: &Vec<Vec<Mapping<T>>>) -> Self {
        let mut root: MappingTrieNode<T> = Root(HashMap::new());

        for mapping in mappings {
            Self::map(&mut root, mapping, 0);
        }

        Self {
            root,
            buffer: vec![],
        }
    }

    pub fn find_mapping(&mut self, key: &KeyPress) -> Option<KeyHandlerAction<T>> {
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
                        self.buffer.push(key.clone());
                        let action = to_handler_action(mapping, &self.buffer);
                        Some(action)
                    }
                    Repeatable(mapping, _, _) => {
                        self.buffer.push(key.clone());
                        let action = to_handler_action(mapping, &self.buffer);
                        Some(action)
                    }
                },
            },
            Repeatable(mapping, repeatable_set, next) => {
                if repeatable_set.contains(key) {
                    self.buffer.push(key.clone());
                    let action = to_handler_action(mapping, &self.buffer);
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
                                self.buffer.push(key.clone());
                                let action = to_handler_action(mapping, &self.buffer);
                                Some(action)
                            }
                            Repeatable(mapping, _, _) => {
                                self.buffer.push(key.clone());
                                let action = to_handler_action(mapping, &self.buffer);
                                Some(action)
                            }
                        },
                    }
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
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

    #[derive(Eq, Debug, Clone, PartialEq)]
    enum TestAction {
        Princess,
        Kenny,
    }

    impl Display for TestAction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestAction::Princess => write!(f, "Princess"),
                TestAction::Kenny => write!(f, "Kenny"),
            }
        }
    }

    macro_rules! u {
        ($user_action:expr) => {
            $crate::types::ActionType::User($user_action)
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
    #[case(m!([[t!(KeyA, ModAlt)]]), &[alt!(KeyA)], Some(timeout!()))]
    #[case(m!([[a!(KeyA, ModAlt, u!(Princess))]]), &[alt!(KeyA)], Some(action!(Princess, [alt!(KeyA)])))]
    #[case(m!([[t!(KeyA), t!(Key1)]]), &[key!(KeyA), key!(Key1)], Some(timeout!()))]
    #[case(m!([[t!(KeyA), t!(Key1)]]), &[key!(Key2)], None)]
    #[case(m!([[t!(KeyA), t!(Key1), a!(Key2, u!(Princess))]]), &[key!(KeyA), key!(Key2), key!(Key2)], None)]
    #[case(m!([[t!(KeyA), t!(Key1), a!(Key2, u!(Princess))]]), &[key!(KeyA), key!(Key1), key!(Key2)], Some(action!(Princess, [key!(KeyA), key!(Key1), key!(Key2)])))]
    #[case(m!([[aat!([key!(Key1)], u!(Princess))]]), &[key!(Key1)], Some(action_t!(Princess, [key!(Key1)])))]
    #[case(m!([[aat!([key!(Key1)], u!(Princess))]]), &[key!(Key1)], Some(action_t!(Princess, [key!(Key1)])))]
    #[case(m!([[aat!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(Key1)], Some(action_t!(Princess, [key!(Key1)])))]
    #[case(m!([[aat!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(Key2)], Some(action_t!(Princess, [key!(Key2)])))]
    #[case(m!([[aat!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(Key2)], Some(action_t!(Princess, [key!(Key2)])))]
    #[case(m!([[abt!([key!(Key1)], u!(Princess))]]), &[key!(Key1)], Some(t_action!(Princess, [key!(Key1)])))]
    #[case(m!([[abt!([key!(Key1)], u!(Princess))]]), &[key!(Key1)], Some(t_action!(Princess, [key!(Key1)])))]
    #[case(m!([[abt!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(Key1)], Some(t_action!(Princess, [key!(Key1)])))]
    #[case(m!([[abt!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(Key2)], Some(t_action!(Princess, [key!(Key2)])))]
    #[case(m!([[abt!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(Key2)], Some(t_action!(Princess, [key!(Key2)])))]
    #[case(m!([[t!(KeyA), a!(KeyX, u!(Princess))], [t!(KeyA), aat!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(KeyA), key!(Key1)], Some(action_t!(Princess, [key!(KeyA), key!(Key1)])))]
    #[case(m!([[t!(KeyA), a!(KeyX, u!(Princess))], [t!(KeyA), aat!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(KeyA), key!(Key1), key!(Key3)], None)]
    #[case(m!([[t!(KeyA), a!(KeyX, u!(Princess))], [t!(KeyA), aat!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(KeyA), key!(Key3)], None)]
    #[case(m!([[t!(KeyA), tm!([key!(KeyB)]), a!(KeyX, u!(Kenny))]]), &[key!(KeyA), key!(KeyB), key!(KeyB), key!(KeyX)], Some(action!(Kenny, [key!(KeyA), key!(KeyB), key!(KeyB), key!(KeyX)])))]
    #[case(m!([[t!(KeyA), tm!([key!(KeyB)]), t!(KeyX), tm!([key!(KeyC)]), a!(KeyX, u!(Kenny))]]), &[key!(KeyA), key!(KeyB), key!(KeyB), key!(KeyX), key!(KeyC), key!(KeyC), key!(KeyX)], Some(action!(Kenny, [key!(KeyA), key!(KeyB), key!(KeyB), key!(KeyX), key!(KeyC), key!(KeyC), key!(KeyX)])))]
    fn should_match_keys_to_mappings(
        #[case] mappings: Vec<Vec<Mapping<TestAction>>>,
        #[case] keypresses: &[KeyPress],
        #[case] expected: Option<KeyHandlerAction<TestAction>>,
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
