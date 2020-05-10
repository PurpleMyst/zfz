use std::io::{self, prelude::*};

/// A text color
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

/// A text style
#[derive(Debug, Clone)]
pub enum Style {
    Foreground(Color),
    Background(Color),
    Bold,
    Underlined,
    Compound(Vec<Style>),
}

#[cfg(not(windows))]
mod nix;

#[cfg(windows)]
mod win32;

#[cfg(windows)]
pub use win32::Console;

#[cfg(not(windows))]
pub use nix::Console;
