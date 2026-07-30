use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct UndoRedo {
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    max_undo: usize,
}

#[wasm_bindgen]
impl UndoRedo {
    pub fn new() -> UndoRedo {
        UndoRedo {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo: 100,
        }
    }

    pub fn save(&mut self, current_state_json: &str) {
        self.undo_stack.push(current_state_json.to_string());
        if self.undo_stack.len() > self.max_undo {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self, current_state_json: &str) -> String {
        if self.undo_stack.is_empty() {
            return String::new();
        }
        self.redo_stack.push(current_state_json.to_string());
        self.undo_stack.pop().unwrap()
    }

    pub fn redo(&mut self, current_state_json: &str) -> String {
        if self.redo_stack.is_empty() {
            return String::new();
        }
        self.undo_stack.push(current_state_json.to_string());
        self.redo_stack.pop().unwrap()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for UndoRedo {
    fn default() -> Self {
        Self::new()
    }
}
