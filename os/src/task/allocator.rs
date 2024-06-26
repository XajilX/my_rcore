use alloc::vec::Vec;

pub struct IdAlloc {
    curr: usize,
    recycled: Vec<usize>
}

impl IdAlloc {
    pub fn new() -> Self {
        Self { curr: 0, recycled: Vec::new() }
    }
    pub fn alloc(&mut self) -> usize {
        if let Some(id) = self.recycled.pop() {
            id
        } else {
            self.curr += 1;
            self.curr - 1
        }
    }
    pub fn dealloc(&mut self, id: usize) {
        assert!(id < self.curr &&
                self.recycled.iter().find(|&&ph| ph == id).is_none(),
            "id {} not in use before dealloc", id
        );
        self.recycled.push(id);
    }
}