use std::cmp::min;

/// A sliding window over a slice
#[derive(Debug, Clone, Copy, Default)]
pub struct SlidingWindow {
    size: usize,
    offset: usize,
}

impl SlidingWindow {
    /// Create a new sliding window of a given size
    /// By default it starts at offset 0
    pub fn new(size: usize) -> Self {
        Self { size, offset: 0 }
    }

    /// Scroll the window down by one
    pub fn scroll_down(&mut self) {
        self.offset += 1;
    }

    /// Scroll the window up by one
    pub fn scroll_up(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }

    /// Apply the window to a given slice
    pub fn apply<'a, T>(&mut self, slice: &'a [T]) -> &'a [T] {
        let size = min(self.size, slice.len());
        self.offset = min(self.offset, slice.len() - size);
        &slice[self.offset..self.offset + size]
    }
}
