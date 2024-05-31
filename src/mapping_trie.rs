use std::collections::{HashMap, HashSet};

use crate::types::Action::*;
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
    }
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

trait KeyMapGetter {
    fn get_next_mapping(&self, key: &KeyPress) -> &MappingTrieNode;
}

#[derive(Debug)]
enum MappingTrieNode {
    Root(KeyHashMap),
    OneOff(Mapping, KeyHashMap),
    Repeatable(KeyPress, Mapping, HashSet<KeyPress>),
}

use MappingTrieNode::*;

pub struct MappingTrie {
    keys: MappingTrieNode,
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

    pub fn from_mappings(mappings: Vec<Vec<Mapping>>) -> Self {
        let mut keys: MappingTrieNode = Root(HashMap::new());

        for mapping in mappings {
            let mut node = &mut keys;

            for m in mapping {
                let key = m.get_key();

                match key {
                    Single(key_press) => match node {
                        Root(next) => {
                            node = next
                                .entry(key_press.clone())
                                .or_insert(OneOff(m, HashMap::new()));
                        }
                        OneOff(_, next) => {
                            node = next
                                .entry(key_press.clone())
                                .or_insert(OneOff(m, HashMap::new()));
                        }
                        Repeatable(_, mapping, _) => {
                            println!(
                                "Can't add mapping {} because it conflicts with {}.",
                                m, mapping
                            );
                            break;
                        }
                    },
                    Choice(key_presses) => match node {
                        Root(next) | OneOff(_, next) => {
                            if Self::all_keys_available(next, key_presses) {
                                for key_press in key_presses.clone().0 {
                                    let set = HashSet::from_iter(key_presses.clone().0.into_iter());

                                    next.insert(
                                        key_press.clone(),
                                        Repeatable(key_press, m.clone(), set),
                                    );
                                }
                            } else {
                                println!("Not all keys are available!");
                                break;
                            }
                        }
                        Repeatable(..) => {
                            println!("Dupa!");
                            break;
                        }
                    },
                }
            }
        }

        Self {
            keys,
            buffer: vec![],
        }
    }

    pub fn find_mapping(&mut self, key: &KeyPress) -> Option<Mapping> {
        let mut keys = &self.keys;

        for key_press in &self.buffer {
            let key = key_press;

            match keys {
                Root(next) | OneOff(_, next) => {
                    if next.contains_key(key) {
                        keys = &next.get(key).unwrap();
                    } else {
                        self.reset();
                        return None;
                    }
                }
                Repeatable(key, _, next) => {
                    if !next.contains(key) {
                        self.reset();
                        return None;
                    }
                }
            }
        }

        match keys {
            Root(next) | OneOff(_, next) => {
                if next.contains_key(key) {
                    let keys = next.get(key).unwrap();

                    match keys {
                        Root(_) => {
                            self.reset();
                            None
                        }
                        OneOff(mapping, _) => {
                            self.buffer.push(key.clone());
                            Some(mapping.clone())
                        }
                        Repeatable(_, mapping, _) => {
                            self.buffer.push(key.clone());
                            Some(mapping.clone())
                        }
                    }
                } else {
                    self.reset();
                    None
                }
            }
            Repeatable(_, mapping, next) => {
                if !next.contains(key) {
                    self.reset();
                    None
                } else {
                    self.buffer.push(key.clone());
                    Some(mapping.clone())
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
    fn should_match_keys_to_mappings(
        #[case] mappings: Vec<Vec<Mapping>>,
        #[case] keypresses: &[KeyPress],
        #[case] expected: Option<Mapping>,
    ) {
        // Given
        let mut trie = MappingTrie::from_mappings(mappings);
        let mut result = None;

        // When
        for key in keypresses {
            result = trie.find_mapping(key);
        }

        // Then
        assert_eq!(result, expected)
    }
}
