use crate::types::Action::*;
use crate::types::Key::*;
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
                KeyPresses(vec![
                    KeyPress::nomod(KeyA),
                    KeyPress::nomod(Key2),
                    KeyPress::nomod(Key3),
                    KeyPress::nomod(Key4),
                    KeyPress::nomod(Key5),
                ])
                .choice(),
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
        None
    }
}

