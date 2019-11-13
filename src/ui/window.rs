pub struct Window {
    size: usize,
    offset: usize,
}

impl Window {
    pub fn new(size: usize) -> Self {
        Self { size, offset: 0 }
    }

    pub fn scroll_down(&mut self) {
        self.offset += 1;
    }

    pub fn scroll_up(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }

    pub fn apply<'a, T>(&mut self, slice: &'a [T]) -> &'a [T] {
        let size = std::cmp::min(self.size, slice.len());
        self.offset = std::cmp::min(self.offset, slice.len() - size);
        &slice[self.offset..self.offset + size]
    }
}
