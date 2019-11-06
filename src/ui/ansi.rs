#![allow(dead_code)]

const CSI: &str = "\x1b[";

macro_rules! control_sequence {
    (#[doc=$doc:literal] $name:ident: CSI $first_parameter:ident $(; $other_parameter:ident)* $final_byte:ident) => {
        #[doc=$doc]
        pub(super) fn $name($first_parameter: usize$(, $other_parameter: usize)*) -> std::io::Result<()> {
            use std::io::Write;
            let stdout = std::io::stdout();
            let mut stdout_lock = stdout.lock();

            write!(stdout_lock, "{}", CSI)?;
            write!(stdout_lock, "{}", $first_parameter)?;
            $(write!(stdout_lock, ",{}", $other_parameter)?;)*
            write!(stdout_lock, "{}", stringify!($final_byte))?;
            stdout_lock.flush()?;

            Ok(())
        }
    };

    (#[doc=$doc:literal] $name:ident: CSI $final_byte:ident) => {
        #[doc=$doc]
        pub(super) fn $name() -> std::io::Result<()> {
            use std::io::Write;
            let stdout = std::io::stdout();
            let mut stdout_lock = stdout.lock();

            write!(stdout_lock, "{}{}", CSI, stringify!($final_byte))?;
            stdout_lock.flush()?;

            Ok(())
        }
    };
}

control_sequence! {
    #[doc="Moves the cursor n (default 1) cells in the given direction. If the cursor is already at the edge of the screen, this has no effect."]
    cursor_up: CSI n A
}

control_sequence! {
    #[doc="Moves the cursor n (default 1) cells in the given direction. If the cursor is already at the edge of the screen, this has no effect."]
    cursor_down: CSI n B
}

control_sequence! {
    #[doc="Moves the cursor n (default 1) cells in the given direction. If the cursor is already at the edge of the screen, this has no effect."]
    cursor_forward: CSI n C
}

control_sequence! {
    #[doc="Moves the cursor n (default 1) cells in the given direction. If the cursor is already at the edge of the screen, this has no effect."]
    cursor_back: CSI n D
}

control_sequence! {
    #[doc="Moves cursor to beginning of the line n (default 1) lines down. (not ANSI.SYS)"]
    cursor_next_line: CSI n E
}

control_sequence! {
    #[doc="Moves cursor to beginning of the line n (default 1) lines up. (not ANSI.SYS)"]
    cursor_previous_line: CSI n F
}

control_sequence! {
    #[doc="Moves the cursor to column n (default 1). (not ANSI.SYS)"]
    cursor_horizontal_absolute: CSI n G
}

control_sequence! {
    #[doc="Moves the cursor to row n, column m. The values are 1-based, and default to 1 (top left corner) if omitted. A sequence such as CSI ;5H is a synonym for CSI 1;5H as well as CSI 17;H is the same as CSI 17H and CSI 17;1H"]
    cursor_position: CSI n ; m H
}

control_sequence! {
    #[doc="Clears part of the screen. If n is 0 (or missing), clear from cursor to end of screen. If n is 1, clear from cursor to beginning of the screen. If n is 2, clear entire screen (and moves cursor to upper left on DOS ANSI.SYS). If n is 3, clear entire screen and delete all lines saved in the scrollback buffer (this feature was added for xterm and is supported by other terminal applications)."]
    erase_in_display: CSI n J
}

control_sequence! {
    #[doc="Erases part of the line. If n is 0 (or missing), clear from cursor to the end of the line. If n is 1, clear from cursor to beginning of the line. If n is 2, clear entire line. Cursor position does not change."]
    erase_in_line: CSI n K
}

control_sequence! {
    #[doc="Saves the cursor position/state."]
    save_cursor_position: CSI s
}

control_sequence! {
    #[doc="Restores the cursor position/state."]
    restore_cursor_position: CSI u
}
