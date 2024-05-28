use crate::types::Action::*;
use crate::types::KeyPress;
use crate::types::KeyPresses;
use crate::types::Mapping;
use crate::types::Mapping::*;
use crate::types::ALT_A;
use crate::types::KEY_1;
use crate::types::KEY_2;
use crate::types::KEY_3;
use crate::types::KEY_4;
use crate::types::KEY_5;
use crate::types::KEY_A;
use crate::types::KEY_X;

pub fn define_mappings() -> Vec<Vec<Mapping>> {
    vec![
        vec![Timeout(ALT_A), Action(KEY_X, Bye)],
        vec![Timeout(ALT_A), Timeout(KEY_A), Action(KEY_X, Bye)],
        vec![Timeout(ALT_A), Timeout(KEY_A), Action(KEY_A, PrincessKenny)],
        vec![
            Timeout(ALT_A),
            ActionAfterTimeout(
                KeyPresses(vec![KEY_1, KEY_2, KEY_3, KEY_4, KEY_5]),
                PrincessKenny,
            ),
        ],
    ]
}

pub struct MappingHandler {
    mappings: Vec<Vec<Mapping>>,
}

impl MappingHandler {
    pub fn new(mappings: Vec<Vec<Mapping>>) -> Self {
        Self { mappings }
    }

    pub fn handle_key_press(&self, buffer: &[KeyPress], key_press: &KeyPress) -> Option<&Mapping> {
        'mapping_loop: for mapping in &self.mappings {
            // Iterate only up to the length of buffer + 1 (current key_press included)
            let max_index = buffer.len().min(mapping.len() - 1);

            for (i, key_press_in_buffer) in buffer.iter().enumerate().take(max_index) {
                if !mapping[i].matches_key(key_press_in_buffer) {
                    continue 'mapping_loop;
                }
            }

            // Check if the current key press matches the next expected key in the mapping sequence
            if let Some(next_mapping) = mapping.get(buffer.len()) {
                match next_mapping {
                    ActionAfterTimeout(key_presses, _) => {
                        // We need to check if the current key press matches the first in the `key_presses` if buffer is empty
                        // or matches the next in sequence after the last matched key in buffer
                        let next_key_index = if buffer.is_empty() {
                            0
                        } else {
                            buffer.len() - 1
                        };
                        if key_presses.0.get(next_key_index) == Some(key_press) {
                            return Some(next_mapping);
                        }
                    }
                    _ => {
                        if next_mapping.matches_key(key_press) {
                            return Some(next_mapping);
                        }
                    }
                }
            }
        }

        None
    }
}
