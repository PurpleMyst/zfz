use crate::selector::{Match, Selector};

use std::io;

mod ansi;
mod termios;

use ansi::style::{Color, Style};

pub struct Display {
    prompt: String,
    selector: Box<dyn Selector>,
    match_amount: Option<usize>,
    selected: usize,

    selected_style: Style,
    highlight_style: Style,
}

impl Display {
    pub fn new(selector: Box<dyn Selector>) -> Self {
        Self {
            prompt: "> ".to_owned(),
            selector,
            match_amount: None,
            selected: 0,

            selected_style: Style::Background(Color::Standard(1)),
            highlight_style: Style::Compound(vec![Style::Bold, Style::Underlined]),
        }
    }

    fn print_prompt(&self) -> io::Result<()> {
        print!("{}", self.prompt);
        Ok(())
    }

    /// Print out a match taking care of highlighting
    fn print_match(&self, index: usize, match_: Match) -> io::Result<()> {
        let mut highlights = match_.highlight.into_iter().peekable();

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

    /// Print out the current matches
    fn print_items(&mut self) -> io::Result<()> {
        let matches = self.selector.get_matches();
        let match_amount = matches.len();

        if self.selected >= match_amount {
            self.selected = match_amount - 1;
        }

        for (index, match_) in matches.into_iter().enumerate() {
            // Erase any leftovers in the line
            ansi::erase_line()?;
            self.print_match(index, match_)?;
            ansi::cursor::move_down()?;
        }

        // If we have less matches than we did before, clear out the leftover lines
        if let Some(old_match_amount) = self.match_amount {
            if old_match_amount > match_amount {
                // We move after erasing because the cursor starts on the first leftover line
                for _ in 0..old_match_amount - match_amount {
                    ansi::erase_line()?;
                    ansi::cursor::move_down()?;
                }
            }
        }
        self.match_amount = Some(match_amount);

        Ok(())
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

        // Print out our prompt
        self.print_prompt()?;

        // Then save the current cursor position such that we can come back here
        ansi::cursor::save_position()?;

        // Print out the list of items on the line after the prompt
        ansi::cursor::move_down()?;
        self.print_items()?;

        // And make the user type at the end of the prompt
        ansi::cursor::restore_position()?;

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
                    ansi::cursor::save_position()?;
                    ansi::cursor::move_down()?;
                    self.print_items()?;
                    ansi::cursor::restore_position()?;
                }

                // ESCape sequence
                //
                // Arrow keys are represented as \033[ followed by any of A, B, C or D that each
                // correspond to up, down, right and left
                0o33 => {
                    assert_eq!(self.read_char()?, b'[');

                    // If we've pressed an arrow, vary the selected index accordingly ...
                    match self.read_char()? {
                        // Up
                        b'A' => self.selected = self.selected.saturating_sub(1),

                        // Down
                        b'B' => self.selected = self.selected.saturating_add(1),

                        _ => {}
                    }

                    // ... then redraw the matches
                    ansi::cursor::save_position()?;
                    ansi::cursor::move_down()?;
                    self.print_items()?;
                    ansi::cursor::restore_position()?;
                }

                // If the character is printable ...
                c if c >= 0x20 && c <= 0x7e => {
                    // ... push it to the pattern and relay the change to the selector ...
                    pattern.push(c as char);
                    self.selector.set_pattern(&pattern);

                    // ... echo it to the user ...
                    print!("{}", c as char);
                    io::stdout().flush()?;

                    // ... and print out the new matches, moving back to the prompt once done
                    ansi::cursor::save_position()?;
                    ansi::cursor::move_down()?;
                    self.print_items()?;
                    ansi::cursor::restore_position()?;
                }

                // Any other control characters are ignored
                c => eprintln!("control char {:?}", c),
            }
        }

        Ok(())
    }
}
