use crate::KeyPress;

pub struct KeyPressBuffer {
    buffer: Vec<KeyPress>,
}

impl KeyPressBuffer {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn push(&mut self, key: KeyPress) {
        self.buffer.push(key);
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn get_keypresses(&self) -> &Vec<KeyPress> {
        &self.buffer
    }
}
