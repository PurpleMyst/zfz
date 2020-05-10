use super::*;

pub struct Console {}

impl Console {
    pub const CTRL_C: u8 = 3;
    pub const BACKSPACE: u8 = 127;
    pub const ESC: u8 = 0o33;

    pub fn new() -> io::Result<Self> {
        todo!()
    }

    fn apply_color(&self, _foreground: bool, _color: &Color) -> io::Result<()> {
        todo!()
    }

    pub fn apply_style(&self, _style: &Style) -> io::Result<()> {
        todo!()
    }

    pub fn reset_all(&self) -> io::Result<()> {
        todo!()
    }

    /// Erase the current line and move the cursor to the beginning of it
    pub fn erase_line(&self) -> io::Result<()> {
        todo!()
    }

    pub fn move_down(&self) -> io::Result<()> {
        self.move_down_n(1)
    }

    /// Move the cursor to the start of the line N down
    pub fn move_down_n(&self, _n: usize) -> io::Result<()> {
        todo!()
    }

    /// Save the current caret position
    pub fn save_caret_position(&self) -> io::Result<()> {
        todo!()
    }

    /// Restore a saved caret position
    pub fn restore_caret_position(&self) -> io::Result<()> {
        todo!()
    }

    pub fn read_one_char(&self) -> io::Result<u8> {
        todo!()
    }
}

impl Write for Console {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}
