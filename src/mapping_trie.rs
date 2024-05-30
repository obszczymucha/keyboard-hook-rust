use std::collections::{HashMap, HashSet};

use crate::types::Key::*;
use crate::types::{Action::*, KEY_A};
use crate::types::{KeyPress, KeyPressType::Choice, KeyPressType::Single, Mapping};
use crate::types::{KeyPresses, ALT_A, KEY_X};

macro_rules! t {
    ($key:expr) => {
        Mapping::Timeout($key)
    };
}

macro_rules! a {
    ($key:expr, $action:expr) => {
        Mapping::Action($key, $action)
    };
}

macro_rules! aat {
    ([$($keypresses:expr),* $(,)?], $action:expr) => {
        Mapping::ActionAfterTimeout(KeyPresses(vec![$($keypresses),*]).choice(), $action)
    }
}

macro_rules! key {
    ($key:expr) => {
        KeyPress::nomod($key)
    };
}

pub fn define_mappings() -> Vec<Vec<Mapping>> {
    vec![
        vec![t!(ALT_A), a!(KEY_X, Bye)],
        vec![t!(ALT_A), t!(KEY_A), a!(KEY_X, Bye)],
        vec![t!(ALT_A), t!(KEY_A), a!(KEY_A, PrincessKenny)],
        vec![
            t!(ALT_A),
            aat!(
                [key!(KeyA), key!(Key2), key!(Key3), key!(Key4), key!(Key5)],
                PrincessKenny
            ),
        ],
    ]
}

type KeyHashMap = HashMap<KeyPress, MappingTrieNode>;

trait KeyMapGetter {
    fn get_next_mapping(&self, key: &KeyPress) -> &MappingTrieNode;
}

enum MappingTrieNode {
    Root(KeyHashMap),
    OneOff(Mapping, KeyHashMap),
    Repeatable(KeyPress, Mapping, HashSet<KeyPress>),
}

use MappingTrieNode::*;

pub struct MappingTrie {
    keys: MappingTrieNode,
}

impl MappingTrie {
    fn all_keys_available(keys: &KeyHashMap, key_presses: &KeyPresses) -> bool {
        for key_press in &key_presses.0 {
            if !keys.contains_key(key_press) {
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
                        Root(next) | OneOff(_, next) => {
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
                        Root(next) => {
                            if Self::all_keys_available(next, key_presses) {
                                for key_press in key_presses.clone().0 {
                                    next.insert(
                                        key_press.clone(),
                                        Repeatable(
                                            key_press,
                                            m.clone(),
                                            HashSet::from_iter(key_presses.clone().0.into_iter()),
                                        ),
                                    );
                                }
                            } else {
                                println!("Not all keys are available!");
                                break;
                            }
                        }
                        OneOff(..) => {
                            println!("Chuj!");
                            break;
                        }
                        Repeatable(..) => {
                            println!("Dupa!");
                            break;
                        }
                    },
                }
            }
        }

        Self { keys }
    }

    pub fn find_mapping(&self, buffer: &[KeyPress], key: &KeyPress) -> Option<&Mapping> {
        let mut keys = &self.keys;

        for key_press in buffer {
            let key = key_press;

            match keys {
                Root(next) | OneOff(_, next) => {
                    if next.contains_key(key) {
                        keys = &next.get(key).unwrap();
                    } else {
                        return None;
                    }
                }
                Repeatable(key, _, next) => {
                    if !next.contains(key) {
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
                        Root(_) => None,
                        OneOff(mapping, _) => Some(mapping),
                        Repeatable(_, mapping, _) => Some(mapping),
                    }
                } else {
                    None
                }
            }
            Repeatable(key, mapping, next) => {
                if !next.contains(key) {
                    None
                } else {
                    Some(mapping)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ALT_A;
    use crate::types::{KEY_1, KEY_2};
    use rstest::rstest;

    macro_rules! abt {
        ([$($keypresses:expr),* $(,)?], $action:expr) => {
            Mapping::ActionBeforeTimeout(KeyPresses(vec![$($keypresses),*]).choice(), $action)
        }
    }

    macro_rules! alt {
        ($key:expr) => {
            KeyPress::alt($key)
        };
    }

    #[rstest]
    #[case(&[t!(ALT_A)], &[], &key!(KeyX), None)]
    #[case(&[t!(ALT_A)], &[], &alt!(KeyA), Some(t!(ALT_A)))]
    #[case(&[a!(ALT_A, Bye)], &[], &alt!(KeyA), Some(a!(ALT_A, Bye)))]
    #[case(&[t!(ALT_A), t!(KEY_1)], &[alt!(KeyA)], &key!(Key1), Some(t!(KEY_1)))]
    #[case(&[t!(ALT_A), t!(KEY_1)], &[alt!(KeyA)], &key!(Key2), None)]
    #[case(&[t!(ALT_A), t!(KEY_1), a!(KEY_2, Bye)], &[alt!(KeyA), key!(Key2)], &key!(Key2), None)]
    #[case(&[t!(ALT_A), t!(KEY_1), a!(KEY_2, Bye)], &[alt!(KeyA), key!(Key1)], &key!(Key2), Some(a!(KEY_2, Bye)))]
    #[case(&[aat!([key!(Key1)], Bye)], &[], &key!(Key1), Some(aat!([key!(Key1)], Bye)))]
    #[case(&[aat!([key!(Key1)], Bye)], &[key!(Key1)], &key!(Key1), Some(aat!([key!(Key1)], Bye)))]
    #[case(&[aat!([key!(Key1), key!(Key2)], Bye)], &[], &key!(Key1), Some(aat!([key!(Key1), key!(Key2)], Bye)))]
    #[case(&[aat!([key!(Key1), key!(Key2)], Bye)], &[], &key!(Key2), Some(aat!([key!(Key1), key!(Key2)], Bye)))]
    #[case(&[aat!([key!(Key1), key!(Key2)], Bye)], &[key!(Key1)], &key!(Key2), Some(aat!([key!(Key1), key!(Key2)], Bye)))]
    #[case(&[abt!([key!(Key1)], Bye)], &[], &key!(Key1), Some(abt!([key!(Key1)], Bye)))]
    #[case(&[abt!([key!(Key1)], Bye)], &[key!(Key1)], &key!(Key1), Some(abt!([key!(Key1)], Bye)))]
    #[case(&[abt!([key!(Key1), key!(Key2)], Bye)], &[], &key!(Key1), Some(abt!([key!(Key1), key!(Key2)], Bye)))]
    #[case(&[abt!([key!(Key1), key!(Key2)], Bye)], &[], &key!(Key2), Some(abt!([key!(Key1), key!(Key2)], Bye)))]
    #[case(&[abt!([key!(Key1), key!(Key2)], Bye)], &[key!(Key1)], &key!(Key2), Some(abt!([key!(Key1), key!(Key2)], Bye)))]
    fn should_match_keys_to_mappings(
        #[case] mapping: &[Mapping],
        #[case] buffer: &[KeyPress],
        #[case] key: &KeyPress,
        #[case] expected: Option<Mapping>,
    ) {
        // Given
        let mappings = vec![mapping.to_vec()];
        let trie = MappingTrie::from_mappings(mappings);

        // When
        let result = trie.find_mapping(buffer, key);

        // Then
        assert_eq!(result, expected.as_ref())
    }
}
