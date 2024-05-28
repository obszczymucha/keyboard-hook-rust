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
            let max_index = buffer.len().min(mapping.len() - 1);

            for (i, key_press_in_buffer) in buffer.iter().enumerate().take(max_index) {
                if !mapping[i].matches_key(key_press_in_buffer) {
                    continue 'mapping_loop;
                }
            }

            if let Some(next_mapping) = mapping.get(buffer.len()) {
                match next_mapping {
                    ActionAfterTimeout(key_presses, _) => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_none_if_the_key_doesnt_match_the_mapping() {
        // Given
        let mappings = vec![vec![Timeout(ALT_A)]];
        let handler = MappingHandler { mappings };

        // When
        let result = handler.handle_key_press(&[], &KEY_X);

        // Then
        assert_eq!(result, None)
    }

    #[test]
    fn should_recognize_one_key() {
        // Given
        let mappings = vec![vec![Timeout(ALT_A)]];
        let handler = MappingHandler { mappings };

        // When
        let result = handler.handle_key_press(&[], &ALT_A);

        // Then
        assert_eq!(result, Some(&Timeout(ALT_A)))
    }
}
