use core::hash::Hash;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::fmt::Display;

use crate::types::Behaviours;
use crate::types::{Mapping, Mapping::Choice, Mapping::Single};
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
                            let set: HashSet<KeyPress> =
                                behaviours.0.iter().map(|b| b.get_key()).collect();

                            behaviours.0.iter().for_each(|b| {
                                let next_node = next
                                    .entry(b.get_key().clone())
                                    .or_insert(Repeatable(m.clone(), set.clone(), HashMap::new()));
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

        Self { root }
    }

    pub fn find_mapping(&self, key: &KeyPress, buffer: &[KeyPress]) -> Option<&Mapping<A, T>> {
        let mut node = &self.root;

        for key_press in buffer {
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
                    None => None,
                    Some(next_node) => match next_node {
                        Root(_) => None,
                        OneOff(mapping, _) | Repeatable(mapping, _, _) => Some(mapping),
                    },
                }
            }
        }
    }
}
