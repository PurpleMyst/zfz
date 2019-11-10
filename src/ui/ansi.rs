use std::io;

mod control_sequence {
    use std::{
        fmt,
        io::{self, Write},
    };

    const CSI: &str = "\x1b[";

    /// Print out a control sequence by joining the given iterator with semicolons
    pub fn print<I>(params: I, final_byte: char) -> io::Result<()>
    where
        I: IntoIterator,
        I::Item: fmt::Display,
    {
        let stdout = io::stdout();
        let mut stdout_lock = stdout.lock();

        // Write out the control sequence introducer
        write!(stdout_lock, "{}", CSI)?;

        // Print out the params separated by semicolons
        params
            .into_iter()
            .enumerate()
            .map(|(i, param)| {
                if i == 0 {
                    write!(stdout_lock, "{}", param)
                } else {
                    write!(stdout_lock, ";{}", param)
                }
            })
            .collect::<io::Result<()>>()?;

        // Print out the final bytes
        write!(stdout_lock, "{}", final_byte)?;

        stdout_lock.flush()
    }
}

/// Wrapper around SGR CSI sequences
pub mod style {
    use super::control_sequence;
    use std::{io, iter};

    const SGR_FINAL_BYTE: char = 'm';

    /// A color that can be applied via a SGR CSI sequence
    #[derive(Debug, Clone, Copy)]
    pub enum Color {
        /// Standard 4-bit color in range 0..8
        Standard(u8),

        /// High-intensity 4-bit color in range 8..15
        Bold(u8),

        /// Color in 6x6x6 cube
        /// Formula: 16 + 36 × r + 6 × g + b
        /// With each component in range 0..6
        Cube(u8, u8, u8),

        /// Grayscale color in range 232..255 (24 steps)
        Grayscale(u8),

        /// True 24-bit color
        True(u8, u8, u8),
    }

    impl Color {
        // Due to `control_sequence::print` taking a single iterator, we use this function to avoid
        // having to allocate a Vec<u8> and then iterate over it.
        // Having a method that returns an `impl Iterator` would not work because different Color
        // variants have different parameter sizes
        fn print(self, first_byte: usize) -> io::Result<()> {
            /// Print out \033[{first_byte};{params}m
            macro_rules! doit {
                [$($param:expr),*] => {
                    control_sequence::print(iter::once(first_byte).chain([$($param),*].iter().copied()), SGR_FINAL_BYTE)
                };
            }

            match self {
                Self::Standard(n) => {
                    assert!(n <= 7);
                    doit![5, n as usize]
                }

                Self::Bold(n) => {
                    assert!(n <= 7);
                    doit![5, n as usize + 8]
                }

                Self::Cube(r, g, b) => {
                    assert!(r <= 5);
                    assert!(g <= 5);
                    assert!(b <= 5);
                    doit![5, 16 + 36 * r as usize + 6 * g as usize + b as usize]
                }

                Self::Grayscale(n) => {
                    assert!(n <= 23);
                    doit![5, n as usize + 232]
                }

                Self::True(r, g, b) => doit![2, r as usize, g as usize, b as usize],
            }
        }

        fn set_foreground(self) -> io::Result<()> {
            self.print(38)
        }

        fn set_background(self) -> io::Result<()> {
            self.print(48)
        }
    }

    /// A text style
    #[derive(Debug, Clone)]
    pub enum Style {
        Foreground(Color),
        Background(Color),
        Bold,
        Underlined,
        Compound(Vec<Style>),
    }

    impl Style {
        /// Apply a style to the following text
        pub fn apply(&self) -> io::Result<()> {
            match self {
                Self::Foreground(color) => color.set_foreground(),
                Self::Background(color) => color.set_background(),
                Self::Bold => control_sequence::print(iter::once(1), SGR_FINAL_BYTE),
                Self::Underlined => control_sequence::print(iter::once(4), SGR_FINAL_BYTE),
                Self::Compound(styles) => styles.iter().map(Style::apply).collect(),
            }
        }

        pub fn reset_all() -> io::Result<()> {
            control_sequence::print(iter::once(0), SGR_FINAL_BYTE)
        }
    }
}

/// Erase the current line and move the cursor to the beginning of it
pub fn erase_line() -> io::Result<()> {
    use std::iter;
    control_sequence::print(iter::once(1), 'G')?; // CHA: Cursor Horizontal Absolute
    control_sequence::print(iter::once(2), 'K') // EL: Erase in line
}

pub mod cursor {
    use super::control_sequence;
    use std::{io, iter};

    /// Move the cursor to beginning of the next line
    pub fn move_down() -> io::Result<()> {
        move_down_n(1)
    }

    pub fn move_down_n(n: usize) -> io::Result<()> {
        control_sequence::print(iter::once(n), 'E')
    }

    /// Save the current cursor position
    pub fn save_position() -> io::Result<()> {
        control_sequence::print::<iter::Empty<u8>>(iter::empty(), 's')
    }

    /// Restore a saved cursor position
    pub fn restore_position() -> io::Result<()> {
        control_sequence::print::<iter::Empty<u8>>(iter::empty(), 'u')
    }
}
