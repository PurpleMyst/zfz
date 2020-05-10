use std::io::{self, prelude::*};
use std::iter;

use super::*;

pub struct Console {
    prev_termios: libc::termios,
}

impl Console {
    const SGR_FINAL_BYTE: char = 'm';
    pub const CTRL_C: u8 = 3;
    pub const BACKSPACE: u8 = 127;
    pub const ESC: u8 = 0o33;

    /// Get the current terminal attributes
    fn get_termios() -> io::Result<libc::termios> {
        use std::mem::MaybeUninit;

        unsafe {
            let mut buf = MaybeUninit::<libc::termios>::uninit();
            if libc::tcgetattr(libc::STDIN_FILENO, buf.as_mut_ptr()) == 0 {
                Ok(buf.assume_init())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    /// Set terminal attributes
    fn set_termios(termios: libc::termios) -> io::Result<()> {
        unsafe {
            if libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &termios) == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    pub fn new() -> io::Result<Self> {
        let mut termios = Self::get_termios()?;
        let prev_termios = termios;
        termios.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
        termios.c_oflag &= !(libc::OPOST);
        termios.c_cflag |= libc::CS8;
        termios.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
        Self::set_termios(termios)?;

        Ok(Self { prev_termios })
    }

    /// Print an ANSI control sequence
    fn print_ansi<I>(&self, params: I, final_byte: char) -> io::Result<()>
    where
        I: IntoIterator,
        I::Item: std::fmt::Display,
    {
        /// Introduces a control sequence
        const CSI: &str = "\x1b[";

        // Lock stdout so that the control sequence comes out right 100% of the time
        let stdout = io::stdout();
        let mut stdout_lock = stdout.lock();

        // Write out the control sequence introducer
        write!(stdout_lock, "{}", CSI)?;

        // Write the first parameter normally and write every other parameter
        // preceded by a semicolon
        let params = params.into_iter();
        if let Some(param) = params.next() {
            write!(stdout_lock, "{}", param)?;
        }
        params.try_for_each(|param| write!(stdout, ";{}", param))?;

        // Print out the final byte that indicates what sequence we want to use
        write!(stdout_lock, "{}", final_byte)?;

        Ok(())
    }

    fn apply_color(&self, foreground: bool, color: &Color) -> io::Result<()> {
        let first_byte = match foreground {
            true => 38,
            false => 48,
        };

        /// Print out \033[{first_byte};{params}m
        macro_rules! doit {
            [$($param:expr),*] => (self.print_ansi([first_byte $(,$param)*].iter().copied(), Self::SGR_FINAL_BYTE));
        }

        match *color {
            Color::Standard(n) => {
                assert!(n <= 7);
                doit![5, n as usize]
            }

            Color::Bold(n) => {
                assert!(n <= 7);
                doit![5, n as usize + 8]
            }

            Color::Cube(r, g, b) => {
                assert!(r <= 5);
                assert!(g <= 5);
                assert!(b <= 5);
                doit![5, 16 + 36 * r as usize + 6 * g as usize + b as usize]
            }

            Color::Grayscale(n) => {
                assert!(n <= 23);
                doit![5, n as usize + 232]
            }

            Color::True(r, g, b) => doit![2, r as usize, g as usize, b as usize],
        }
    }

    pub fn apply_style(&self, style: &Style) -> io::Result<()> {
        match style {
            Style::Foreground(color) => self.apply_color(true, color),
            Style::Background(color) => self.apply_color(false, color),
            Style::Bold => self.print_ansi(iter::once(1), Self::SGR_FINAL_BYTE),
            Style::Underlined => self.print_ansi(iter::once(4), Self::SGR_FINAL_BYTE),
            Style::Compound(styles) => styles.iter().map(|style| self.apply_style(style)).collect(),
        }
    }

    pub fn reset_all(&self) -> io::Result<()> {
        self.print_ansi(iter::once(0), Self::SGR_FINAL_BYTE)
    }

    /// Erase the current line and move the cursor to the beginning of it
    pub fn erase_line(&self) -> io::Result<()> {
        self.print_ansi(iter::once(1), 'G')?; // CHA: Cursor Horizontal Absolute
        self.print_ansi(iter::once(2), 'K')?; // EL: Erase in line
        Ok(())
    }

    pub fn move_down(&self) -> io::Result<()> {
        self.move_down_n(1)
    }

    /// Move the cursor to the start of the line N down
    pub fn move_down_n(&self, n: usize) -> io::Result<()> {
        self.print_ansi(iter::once(n), 'E')
    }

    /// Save the current caret position
    pub fn save_caret_position(&self) -> io::Result<()> {
        self.print_ansi(iter::empty::<u8>(), 's')
    }

    /// Restore a saved caret position
    pub fn restore_caret_position(&self) -> io::Result<()> {
        self.print_ansi(iter::empty::<u8>(), 'u')
    }

    pub fn read_one_char(&self) -> io::Result<u8> {
        unsafe {
            let mut c = 0u8;
            if libc::read(libc::STDIN_FILENO, &mut c as *mut _ as *mut _, 1) == 1 {
                Ok(c)
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

impl Write for Console {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        io::stdout().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}

impl Drop for Console {
    fn drop(&self) {
        Self::set_termios(self.termios).expect("Could not restore termios")
    }
}
