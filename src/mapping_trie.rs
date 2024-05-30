use std::collections::{HashMap, HashSet};

use crate::types::Key::*;
use crate::types::KeyPressType;
use crate::types::KeyPresses;
use crate::types::Modifier::*;
use crate::types::{KeyPress, KeyPressType::Choice, KeyPressType::Single, Mapping};

type KeyHashMap = HashMap<KeyPress, MappingTrieNode>;

trait KeyMapGetter {
    fn get_next_mapping(&self, key: &KeyPress) -> &MappingTrieNode;
}

enum MappingTrieNode {
    Root {
        next: KeyHashMap,
    },
    OneOff {
        mapping: Mapping,
        next: KeyHashMap,
    },
    Repeatable {
        key: KeyPress,
        mapping: Mapping,
        next: HashSet<KeyPress>,
    },
}

use MappingTrieNode::*;

struct MappingTrie {
    keys: MappingTrieNode,
}

struct DifferentTypeAlreadyMapped {
    mapping: Mapping,
    already_mapped_mapping: Mapping,
}

impl MappingTrie {
    fn all_keys_available(keys: &KeyHashMap, key_presses: &KeyPresses) -> bool {
        true
    }

    fn from_mappings(mappings: Vec<Vec<Mapping>>) -> Self {
        let mut keys: MappingTrieNode = Root {
            next: HashMap::new(),
        };

        for mapping in mappings {
            let mut node = &mut keys;

            for m in mapping {
                let key = m.get_key();

                match key {
                    Single(key_press) => match node {
                        Root { next } | OneOff { mapping: _, next } => {
                            node = next.entry(key_press.clone()).or_insert(OneOff {
                                mapping: m,
                                next: HashMap::new(),
                            });
                        }
                        Repeatable { key, mapping, next } => {
                            println!(
                                "Can't add mapping {} because it conflicts with {}.",
                                m, mapping
                            );
                            break;
                        }
                    },
                    Choice(key_presses) => match node {
                        Root { next } => {
                            if Self::all_keys_available(next, key_presses) {
                                for key_press in key_presses.clone().0 {
                                    next.insert(
                                        key_press.clone(),
                                        Repeatable {
                                            key: key_press,
                                            mapping: m.clone(),
                                            next: HashSet::from_iter(
                                                key_presses.clone().0.into_iter(),
                                            ),
                                        },
                                    );
                                }
                            } else {
                                println!("Not all keys are available!");
                                break;
                            }
                        }
                        OneOff { mapping, next } => {
                            println!("Chuj!");
                            break;
                        }
                        Repeatable { key, mapping, next } => {
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
                Root { next } | OneOff { mapping: _, next } => {
                    if next.contains_key(key) {
                        keys = &next.get(key).unwrap();
                    } else {
                        return None;
                    }
                }
                Repeatable { key, mapping, next } => {
                    if !next.contains(key) {
                        return None;
                    }
                }
            }
        }

        match keys {
            Root { next } | OneOff { mapping: _, next } => {
                if next.contains_key(key) {
                    let keys = next.get(key).unwrap();
                    match keys {
                        Root { next } => return None,
                        OneOff { mapping, next } => return Some(mapping),
                        Repeatable { key, mapping, next } => return Some(mapping),
                    }
                } else {
                    return None;
                }
            }
            Repeatable { key, mapping, next } => {
                if !next.contains(key) {
                    return None;
                } else {
                    return Some(mapping);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Action::*;
    use crate::types::KeyPress as KP;
    use crate::types::KeyPresses as KPS;
    use crate::types::Mapping::ActionAfterTimeout as AAT;
    use crate::types::Mapping::ActionBeforeTimeout as ABT;
    use crate::types::{KeyPresses, Mapping::*, KEY_1, KEY_2};
    use crate::types::{ALT_A, KEY_A, KEY_X};
    use rstest::rstest;

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
    #[case(&[aat!([key!(Key1), key!(Key2)], Bye)], &[key!(Key1)], &key!(Key2), Some(aat!([key!(Key1), key!(Key2)], Bye)))]
    #[case(&[abt!([alt!(KeyA)], Bye)], &[], &alt!(KeyA), Some(abt!([alt!(KeyA)], Bye)))]
    // #[case(&[ActionAfterTimeout(KPS(vec![KEY_1, KEY_2]), Bye)], &[], &KEY_1, Some(ActionAfterTimeout(KPS(vec![KEY_1, KEY_2]), Bye)))]
    // #[case(&[ActionAfterTimeout(KPS(vec![KEY_1, KEY_2]), Bye)], &[KEY_1], &KEY_2, Some(ActionAfterTimeout(KPS(vec![KEY_1, KEY_2]), Bye)))]
    // #[case(&[Timeout(ALT_A)], &[ALT_A], &ALT_A, None)]
    // #[case(&[Timeout(ALT_A), Timeout(KEY_A)], &[ALT_A], &ALT_A, None)]
    // #[case(&[Timeout(ALT_A), Timeout(KEY_A)], &[ALT_A], &KEY_A, Some(Timeout(KEY_A)))]
    // #[case(&[Timeout(ALT_A), Timeout(KEY_A)], &[ALT_A, KEY_A], &KEY_A, None)]
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
