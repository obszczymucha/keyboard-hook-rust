use std::collections::{HashMap, HashSet};

use crate::types::ActionType::*;
use crate::types::Key::*;
use crate::types::KeyPresses;
use crate::types::Modifier::ModAlt;
use crate::types::{KeyPress, KeyPressType::Choice, KeyPressType::Single, Mapping};

macro_rules! t {
    ($key:expr) => {
        Mapping::Timeout(Single(KeyPress::nomod($key)))
    };

    ($key:expr, $modifier:expr) => {
        Mapping::Timeout(Single(KeyPress::new($key, $modifier)))
    };
}

macro_rules! tm {
    ([$($keypresses:expr),* $(,)?]) => {
        Mapping::Timeout(KeyPresses(vec![$($keypresses),*]).choice())
    };
}

macro_rules! leader {
    () => {
        t!(KeyA, ModAlt)
    };
}

macro_rules! a {
    ($key:expr, $action:expr) => {
        Mapping::Action(Single(KeyPress::nomod($key)), $action)
    };

    ($key:expr, $modifier:expr, $action:expr) => {
        Mapping::Action(Single(KeyPress::new($key, $modifier)), $action)
    };
}

macro_rules! aat {
    ([$($keypresses:expr),* $(,)?], $action:expr) => {
        Mapping::ActionAfterTimeout(KeyPresses(vec![$($keypresses),*]).choice(), $action)
    }
}

macro_rules! abt {
    ([$($keypresses:expr),* $(,)?], $action:expr) => {
        Mapping::ActionBeforeTimeout(KeyPresses(vec![$($keypresses),*]).choice(), $action)
    };

    ($key:expr, $action:expr) => {
        Mapping::ActionBeforeTimeout(Single(KeyPress::nomod($key)), $action)
    };
}

macro_rules! key {
    ($key:expr) => {
        KeyPress::nomod($key)
    };
}

pub fn define_mappings() -> Vec<Vec<Mapping>> {
    vec![
        vec![leader!(), t!(KeyE), t!(KeyX), t!(KeyI), a!(KeyT, Bye)],
        vec![
            leader!(),
            aat!(
                [key!(Key1), key!(Key2), key!(Key3), key!(Key4), key!(Key5)],
                ToggleChannels
            ),
        ],
        vec![leader!(), abt!([key!(KeyJ), key!(KeyK)], Volume)],
    ]
}

type KeyHashMap = HashMap<KeyPress, MappingTrieNode>;

#[derive(Debug)]
enum MappingTrieNode {
    Root(KeyHashMap),
    OneOff(Mapping, KeyHashMap),
    Repeatable(Mapping, HashSet<KeyPress>, KeyHashMap),
}

use MappingTrieNode::*;

pub struct MappingTrie {
    root: MappingTrieNode,
    buffer: Vec<KeyPress>,
}

impl MappingTrie {
    fn all_keys_available(keys: &KeyHashMap, key_presses: &KeyPresses) -> bool {
        for key_press in &key_presses.0 {
            if keys.contains_key(key_press) {
                return false;
            }
        }

        true
    }

    fn map(root: &mut MappingTrieNode, mapping: &Vec<Mapping>, starting_pos: usize) {
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
    pub fn from_mappings(mappings: &Vec<Vec<Mapping>>) -> Self {
        let mut root: MappingTrieNode = Root(HashMap::new());

        for mapping in mappings {
            Self::map(&mut root, mapping, 0);
        }

        Self {
            root,
            buffer: vec![],
        }
    }

    pub fn find_mapping(&mut self, key: &KeyPress) -> Option<Mapping> {
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
    use rstest::rstest;

    macro_rules! alt {
        ($key:expr) => {
            KeyPress::alt($key)
        };
    }

    macro_rules! m {
        ( [ $( [ $( $x:expr ),* $(,)? ] ),* $(,)? ] ) => {
            vec![
                $(
                    vec![$($x),*]
                ),*
            ]
        }
    }

    #[rstest]
    #[case(m!([[t!(KeyA)]]), &[key!(KeyX)], None)]
    #[case(m!([[t!(KeyA, ModAlt)]]), &[alt!(KeyA)], Some(t!(KeyA, ModAlt)))]
    #[case(m!([[a!(KeyA, ModAlt, Bye)]]), &[alt!(KeyA)], Some(a!(KeyA, ModAlt, Bye)))]
    #[case(m!([[t!(KeyA), t!(Key1)]]), &[key!(KeyA), key!(Key1)], Some(t!(Key1)))]
    #[case(m!([[t!(KeyA), t!(Key1)]]), &[key!(Key2)], None)]
    #[case(m!([[t!(KeyA), t!(Key1), a!(Key2, Bye)]]), &[key!(KeyA), key!(Key2), key!(Key2)], None)]
    #[case(m!([[t!(KeyA), t!(Key1), a!(Key2, Bye)]]), &[key!(KeyA), key!(Key1), key!(Key2)], Some(a!(Key2, Bye)))]
    #[case(m!([[aat!([key!(Key1)], Bye)]]), &[key!(Key1)], Some(aat!([key!(Key1)], Bye)))]
    #[case(m!([[aat!([key!(Key1)], Bye)]]), &[key!(Key1)], Some(aat!([key!(Key1)], Bye)))]
    #[case(m!([[aat!([key!(Key1), key!(Key2)], Bye)]]), &[key!(Key1)], Some(aat!([key!(Key1), key!(Key2)], Bye)))]
    #[case(m!([[aat!([key!(Key1), key!(Key2)], Bye)]]), &[key!(Key2)], Some(aat!([key!(Key1), key!(Key2)], Bye)))]
    #[case(m!([[aat!([key!(Key1), key!(Key2)], Bye)]]), &[key!(Key2)], Some(aat!([key!(Key1), key!(Key2)], Bye)))]
    #[case(m!([[abt!([key!(Key1)], Bye)]]), &[key!(Key1)], Some(abt!([key!(Key1)], Bye)))]
    #[case(m!([[abt!([key!(Key1)], Bye)]]), &[key!(Key1)], Some(abt!([key!(Key1)], Bye)))]
    #[case(m!([[abt!([key!(Key1), key!(Key2)], Bye)]]), &[key!(Key1)], Some(abt!([key!(Key1), key!(Key2)], Bye)))]
    #[case(m!([[abt!([key!(Key1), key!(Key2)], Bye)]]), &[key!(Key2)], Some(abt!([key!(Key1), key!(Key2)], Bye)))]
    #[case(m!([[abt!([key!(Key1), key!(Key2)], Bye)]]), &[key!(Key2)], Some(abt!([key!(Key1), key!(Key2)], Bye)))]
    #[case(m!([[t!(KeyA), a!(KeyX, Bye)], [t!(KeyA), aat!([key!(Key1), key!(Key2)], Bye)]]), &[key!(KeyA), key!(Key1)], Some(aat!([key!(Key1), key!(Key2)], Bye)))]
    #[case(m!([[t!(KeyA), a!(KeyX, Bye)], [t!(KeyA), aat!([key!(Key1), key!(Key2)], Bye)]]), &[key!(KeyA), key!(Key1), key!(Key3)], None)]
    #[case(m!([[t!(KeyA), a!(KeyX, Bye)], [t!(KeyA), aat!([key!(Key1), key!(Key2)], Bye)]]), &[key!(KeyA), key!(Key3)], None)]
    #[case(m!([[t!(KeyA), tm!([key!(KeyB)]), a!(KeyX, ToggleChannels)]]), &[key!(KeyA), key!(KeyB), key!(KeyB), key!(KeyX)], Some(a!(KeyX, ToggleChannels)))]
    #[case(m!([[t!(KeyA), tm!([key!(KeyB)]), t!(KeyX), tm!([key!(KeyC)]), a!(KeyX, ToggleChannels)]]), &[key!(KeyA), key!(KeyB), key!(KeyB), key!(KeyX)], Some(a!(KeyX, ToggleChannels)))]
    fn should_match_keys_to_mappings(
        #[case] mappings: Vec<Vec<Mapping>>,
        #[case] keypresses: &[KeyPress],
        #[case] expected: Option<Mapping>,
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
