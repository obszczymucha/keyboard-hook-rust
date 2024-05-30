use std::collections::{HashMap, HashSet};

use crate::types::Key::*;
use crate::types::KeyPressType;
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
                        Root { next } => {
                            node = next.entry(key_press.clone()).or_insert(OneOff {
                                mapping: m,
                                next: HashMap::new(),
                            });
                        }
                        OneOff { mapping, next } => {
                            node = next.entry(key_press.clone()).or_insert(OneOff {
                                mapping: m,
                                next: HashMap::new(),
                            });
                        }
                        Repeatable { mapping, next } => {
                            println!(
                                "Can't add mapping {} because it conflicts with {}.",
                                m, mapping
                            );
                            break;
                        }
                    },
                    Choice(_) => todo!(),
                }
            }
        }

        Self { keys }
    }

    pub fn find_mapping(&self, buffer: &[KeyPress], key: &KeyPress) -> Option<&Mapping> {
        //     let mut keys = &self.keys;
        //
        //     for key_press in buffer {
        //         let key = &key_press.single();
        //
        //         if keys.contains_key(key) {
        //             keys = &keys.get(key).unwrap().next;
        //         } else {
        //             return None;
        //         }
        //     }
        //
        //     keys.get(&key.single()).map(|node| &node.mapping)
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Action::*;
    use crate::types::{KeyPresses, Mapping::*, KEY_1, KEY_2};
    use crate::types::{ALT_A, KEY_A, KEY_X};
    use rstest::rstest;

    #[rstest]
    #[case(&[Timeout(ALT_A)], &[], &KeyPress::nomod(KeyX), None)]
    #[case(&[Timeout(ALT_A)], &[], &KeyPress::nomod(KeyA), Some(Timeout(ALT_A)))]
    #[case(&[Action(ALT_A, Bye)], &[], &KeyPress::nomod(KeyA), Some(Action(ALT_A, Bye)))]
    #[case(&[ActionBeforeTimeout(KeyPresses(vec![KeyPress::alt(KeyA)]).choice(), Bye)], &[], &KeyPress::alt(KeyA), Some(ActionBeforeTimeout(KeyPresses(vec![KeyPress::alt(KeyA)]).choice(), Bye)))]
    // #[case(&[ActionAfterTimeout(KeyPresses(vec![KEY_1]), Bye)], &[], &KEY_1, Some(ActionAfterTimeout(KeyPresses(vec![KEY_1]), Bye)))]
    // #[case(&[ActionAfterTimeout(KeyPresses(vec![KEY_1, KEY_2]), Bye)], &[], &KEY_1, Some(ActionAfterTimeout(KeyPresses(vec![KEY_1, KEY_2]), Bye)))]
    // #[case(&[ActionAfterTimeout(KeyPresses(vec![KEY_1, KEY_2]), Bye)], &[KEY_1], &KEY_2, Some(ActionAfterTimeout(KeyPresses(vec![KEY_1, KEY_2]), Bye)))]
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
