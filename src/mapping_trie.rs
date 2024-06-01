use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::fmt::Display;

use crate::types::KeyPresses;
use crate::types::{KeyPress, KeyPressType::Choice, KeyPressType::Single, Mapping};

#[macro_export]
macro_rules! t {
    ($key:expr) => {
        Mapping::Timeout($crate::types::KeyPressType::Single(
            $crate::types::KeyPress::nomod($key),
        ))
    };

    ($key:expr, $modifier:expr) => {
        Mapping::Timeout($crate::types::KeyPressType::Single(
            $crate::types::KeyPress::new($key, $modifier),
        ))
    };
}

#[allow(dead_code)]
#[macro_export]
macro_rules! tm {
    ([$($keypresses:expr),* $(,)?]) => {
        Mapping::Timeout(KeyPresses(vec![$($keypresses),*]).choice())
    };
}

#[macro_export]
macro_rules! a {
    ($key:expr, $action:expr) => {
        Mapping::Action(
            $crate::types::KeyPressType::Single($crate::types::KeyPress::nomod($key)),
            $action,
        )
    };

    ($key:expr, $modifier:expr, $action:expr) => {
        Mapping::Action(
            $crate::types::KeyPressType::Single($crate::types::KeyPress::new($key, $modifier)),
            $action,
        )
    };
}

#[macro_export]
macro_rules! aat {
    ([$($keypresses:expr),* $(,)?], $action:expr) => {
        Mapping::ActionAfterTimeout($crate::types::KeyPresses(vec![$($keypresses),*]).choice(), $action)
    }
}

#[macro_export]
macro_rules! abt {
    ([$($keypresses:expr),* $(,)?], $action:expr) => {
        Mapping::ActionBeforeTimeout($crate::types::KeyPresses(vec![$($keypresses),*]).choice(), $action)
    };

    ($key:expr, $action:expr) => {
        Mapping::ActionBeforeTimeout(Single(KeyPress::nomod($key)), $action)
    };
}

#[macro_export]
macro_rules! key {
    ($key:expr) => {
        $crate::types::KeyPress::nomod($key)
    };
}

#[macro_export]
macro_rules! alt {
    ($key:expr) => {
        $crate::types::KeyPress::alt($key)
    };
}

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

pub struct MappingTrie<T>
where
    T: PartialEq + Eq + Clone + Debug + Display + Send + Sync,
{
    root: MappingTrieNode<T>,
    buffer: Vec<KeyPress>,
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

    pub fn find_mapping(&mut self, key: &KeyPress) -> Option<Mapping<T>> {
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
                        Some(mapping.clone())
                    }
                    Repeatable(mapping, _, _) => {
                        self.buffer.push(key.clone());
                        Some(mapping.clone())
                    }
                },
            },
            Repeatable(mapping, repeatable_set, next) => {
                if repeatable_set.contains(key) {
                    self.buffer.push(key.clone());
                    Some(mapping.clone())
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
                                Some(mapping.clone())
                            }
                            Repeatable(mapping, _, _) => {
                                self.buffer.push(key.clone());
                                Some(mapping.clone())
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

    #[rstest]
    #[case(m!([[t!(KeyA)]]), &[key!(KeyX)], None)]
    #[case(m!([[t!(KeyA, ModAlt)]]), &[alt!(KeyA)], Some(t!(KeyA, ModAlt)))]
    #[case(m!([[a!(KeyA, ModAlt, u!(Princess))]]), &[alt!(KeyA)], Some(a!(KeyA, ModAlt, u!(Princess))))]
    #[case(m!([[t!(KeyA), t!(Key1)]]), &[key!(KeyA), key!(Key1)], Some(t!(Key1)))]
    #[case(m!([[t!(KeyA), t!(Key1)]]), &[key!(Key2)], None)]
    #[case(m!([[t!(KeyA), t!(Key1), a!(Key2, u!(Princess))]]), &[key!(KeyA), key!(Key2), key!(Key2)], None)]
    #[case(m!([[t!(KeyA), t!(Key1), a!(Key2, u!(Princess))]]), &[key!(KeyA), key!(Key1), key!(Key2)], Some(a!(Key2, u!(Princess))))]
    #[case(m!([[aat!([key!(Key1)], u!(Princess))]]), &[key!(Key1)], Some(aat!([key!(Key1)], u!(Princess))))]
    #[case(m!([[aat!([key!(Key1)], u!(Princess))]]), &[key!(Key1)], Some(aat!([key!(Key1)], u!(Princess))))]
    #[case(m!([[aat!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(Key1)], Some(aat!([key!(Key1), key!(Key2)], u!(Princess))))]
    #[case(m!([[aat!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(Key2)], Some(aat!([key!(Key1), key!(Key2)], u!(Princess))))]
    #[case(m!([[aat!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(Key2)], Some(aat!([key!(Key1), key!(Key2)], u!(Princess))))]
    #[case(m!([[abt!([key!(Key1)], u!(Princess))]]), &[key!(Key1)], Some(abt!([key!(Key1)], u!(Princess))))]
    #[case(m!([[abt!([key!(Key1)], u!(Princess))]]), &[key!(Key1)], Some(abt!([key!(Key1)], u!(Princess))))]
    #[case(m!([[abt!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(Key1)], Some(abt!([key!(Key1), key!(Key2)], u!(Princess))))]
    #[case(m!([[abt!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(Key2)], Some(abt!([key!(Key1), key!(Key2)], u!(Princess))))]
    #[case(m!([[abt!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(Key2)], Some(abt!([key!(Key1), key!(Key2)], u!(Princess))))]
    #[case(m!([[t!(KeyA), a!(KeyX, u!(Princess))], [t!(KeyA), aat!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(KeyA), key!(Key1)], Some(aat!([key!(Key1), key!(Key2)], u!(Princess))))]
    #[case(m!([[t!(KeyA), a!(KeyX, u!(Princess))], [t!(KeyA), aat!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(KeyA), key!(Key1), key!(Key3)], None)]
    #[case(m!([[t!(KeyA), a!(KeyX, u!(Princess))], [t!(KeyA), aat!([key!(Key1), key!(Key2)], u!(Princess))]]), &[key!(KeyA), key!(Key3)], None)]
    #[case(m!([[t!(KeyA), tm!([key!(KeyB)]), a!(KeyX, u!(Kenny))]]), &[key!(KeyA), key!(KeyB), key!(KeyB), key!(KeyX)], Some(a!(KeyX, u!(Kenny))))]
    #[case(m!([[t!(KeyA), tm!([key!(KeyB)]), t!(KeyX), tm!([key!(KeyC)]), a!(KeyX, u!(Kenny))]]), &[key!(KeyA), key!(KeyB), key!(KeyB), key!(KeyX), key!(KeyC), key!(KeyC), key!(KeyX)], Some(a!(KeyX, u!(Kenny))))]
    fn should_match_keys_to_mappings(
        #[case] mappings: Vec<Vec<Mapping<TestAction>>>,
        #[case] keypresses: &[KeyPress],
        #[case] expected: Option<Mapping<TestAction>>,
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
