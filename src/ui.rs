use crate::selector::{Match, Selector};

use std::io;

mod ansi;
mod termios;

use ansi::style::{Color, Style};

pub struct Display<'a> {
    prompt: String,
    selector: Selector<'a>,
    match_amount: usize,
    selected: usize,
    start: usize,
    rows: usize,

    selected_style: Style,
    highlight_style: Style,
}

impl<'a> Display<'a> {
    pub fn new(selector: Selector<'a>) -> Self {
        Self {
            prompt: "> ".to_owned(),
            selector,
            match_amount: 0,
            selected: 0,
            start: 0,
            rows: 20,

            selected_style: Style::Background(Color::Standard(1)),
            highlight_style: Style::Compound(vec![Style::Bold, Style::Underlined]),
        }
    }

    fn print_prompt(&self) -> io::Result<()> {
        print!("{}", self.prompt);
        Ok(())
    }

    /// Print out a match, taking care of highlighting, on the current line
    fn print_match(&self, index: usize, match_: &Match<'_>) -> io::Result<()> {
        // Erase anything that's in the line
        ansi::erase_line()?;

        let mut highlights = match_.highlight.iter().peekable();

        for (i, c) in match_.item.chars().enumerate() {
            // Get the current highlight group, if there is any
            if let Some(highlight) = highlights.peek() {
                // If the group starts here ...
                if highlight.0 == i {
                    // ... apply the highlight style until the end of the group
                    self.highlight_style.apply()?;
                }

                // If the group stops here ...
                if highlight.1 == i {
                    // ... reset all graphic attributes ...
                    Style::reset_all()?;

                    // ... and move on to the next group
                    highlights.next();
                }
            }

            // We must re-apply the selected style on every iteration due to the `reset_all` above
            if index == self.selected {
                self.selected_style.apply()?;
            }

            print!("{}", c);
        }

        // Regardless of what happened up there, reset all graphic attributes
        Style::reset_all()?;

        Ok(())
    }

    /// Print out the current matcheson the line below the current one, restoring the cursor
    /// position afterwards
    fn print_items(&mut self) -> io::Result<()> {
        ansi::cursor::save_position()?;
        ansi::cursor::move_down()?;

        let matches = &self.selector.matches()[self.start..self.start + self.rows];
        let match_amount = matches.len();

        if self.selected >= match_amount {
            self.selected = match_amount.saturating_sub(1);
        }

        for (index, match_) in matches.into_iter().enumerate() {
            // Erase any leftovers in the line
            self.print_match(index, match_)?;
            ansi::cursor::move_down()?;
        }

        // If we have less matches than we did before, clear out the leftover lines
        if self.match_amount > match_amount {
            // We move after erasing because the cursor starts on the first leftover line
            for _ in 0..self.match_amount - match_amount {
                ansi::erase_line()?;
                ansi::cursor::move_down()?;
            }
        }
        self.match_amount = match_amount;

        ansi::cursor::restore_position()
    }

    fn read_char(&self) -> io::Result<u8> {
        unsafe {
            let mut c = 0u8;
            if libc::read(libc::STDIN_FILENO, &mut c as *mut _ as *mut _, 1) == 1 {
                Ok(c)
            } else {
                Err(io::Error::from_raw_os_error(*libc::__errno_location()))
            }
        }
    }

    pub fn mainloop(&mut self) -> io::Result<()> {
        use io::Write;

        let _guard = termios::raw_mode()?;

        self.print_prompt()?;
        self.print_items()?;

        let mut pattern = String::new();
        loop {
            let c = self.read_char()?;

            match c {
                // If the user inputs Ctrl-C ...
                3 => {
                    // ... bail out!
                    break;
                }

                // If the user inputs a backspace ...
                127 => {
                    // ... remove the latest character and relay the change to the selector ...
                    pattern.pop();
                    self.selector.set_pattern(&pattern);

                    // ... then clear out the prompt line ...
                    ansi::erase_line()?;

                    // ... and print it out again ...
                    self.print_prompt()?;
                    print!("{}", pattern);
                    io::stdout().flush()?;

                    // ... then print out the new matches
                    self.print_items()?;
                }

                // ESCape sequence
                //
                // Arrow keys are represented as \033[ followed by any of A, B, C or D that each
                // correspond to up, down, right and left
                0o33 => {
                    assert_eq!(self.read_char()?, b'[');

                    let matches = &self.selector.matches()[self.start..self.start + self.rows];

                    // If we've pressed an arrow, vary the selected index accordingly ...
                    match self.read_char()? {
                        // We're going vertically
                        c @ b'A' | c @ b'B' => {
                            let old_selected = if c == b'A' {
                                // We're going up
                                if self.selected == 0 {
                                    // If we're already at the top of the screen, scroll up
                                    self.start = self.start.saturating_sub(1);
                                    self.print_items()?;
                                    continue;
                                }

                                self.selected -= 1;
                                self.selected + 1
                            } else {
                                // We're going down.
                                // If we're already at the end of the list, scroll down
                                if self.selected == self.match_amount.saturating_sub(1) {
                                    self.start += 1;
                                    self.print_items()?;
                                    continue;
                                }

                                self.selected += 1;
                                self.selected - 1
                            };

                            // Save our current cursor position, which is on the prompt line
                            ansi::cursor::save_position()?;

                            // Redraw the old selected line
                            ansi::cursor::move_down_n(old_selected + 1)?;
                            self.print_match(old_selected, &matches[old_selected])?;

                            // Draw the new selected line
                            ansi::cursor::restore_position()?;
                            ansi::cursor::move_down_n(self.selected + 1)?;
                            self.print_match(self.selected, &matches[self.selected])?;

                            // And go back to our prompt
                            ansi::cursor::restore_position()?;
                        }

                        _ => {}
                    }
                }

                // If the character is printable ...
                c if c >= 0x20 && c <= 0x7e => {
                    // ... push it to the pattern and relay the change to the selector ...
                    pattern.push(c as char);
                    self.selector.set_pattern(&pattern);

                    // ... echo it to the user ...
                    print!("{}", c as char);
                    io::stdout().flush()?;

                    // ... and print out the new matches
                    self.print_items()?;
                }

                // Any other control characters are ignored
                c => eprintln!("control char {:?}", c),
            }
        }

        Ok(())
    }
}
