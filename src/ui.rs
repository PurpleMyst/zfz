use std::io::{self, prelude::*};

use crate::console::{Color, Console, Style};
use crate::selector::{Match, Selector};
use crate::sliding_window::SlidingWindow;

pub struct UI<'a> {
    console: Console,

    prompt: String,

    selector: Selector<'a>,
    match_amount: usize,

    selected: usize,

    window: SlidingWindow,

    selected_style: Style,
    highlight_style: Style,
}

impl<'a> UI<'a> {
    pub fn new(selector: Selector<'a>) -> io::Result<Self> {
        Ok(Self {
            prompt: "> ".to_owned(),

            selector,
            match_amount: 0,

            selected: 0,

            window: SlidingWindow::new(20),

            selected_style: Style::Background(Color::Standard(1)),
            highlight_style: Style::Compound(vec![Style::Bold, Style::Underlined]),
            console: Console::new()?,
        })
    }

    fn print_prompt(&mut self) -> io::Result<()> {
        write!(self.console, "{}", self.prompt)?;
        Ok(())
    }

    /// Print out a match, taking care of highlighting, on the current line
    fn print_match(&self, selected: bool, Match { item, highlight }: &Match<'a>) -> io::Result<()> {
        // Erase anything that's in the line
        self.console.erase_line()?;

        let stdout = io::stdout();
        let mut stdout_lock = stdout.lock();

        let mut print = move |s| -> io::Result<()> {
            if selected {
                self.console.apply_style(&self.selected_style)?;
            }

            write!(stdout_lock, "{}", s)
        };

        let end = highlight
            .iter()
            .try_fold(0, |last, &(start, end)| -> io::Result<usize> {
                // Print out the stuff between highlight groups normally
                self.console.reset_all()?;
                print(&item[last..start])?;

                // Print the inside of the group with the highlight style
                self.console.apply_style(&self.highlight_style)?;
                print(&item[start..end])?;

                // Pass on the ball
                Ok(end)
            })?;

        // Print out what's leftover normally
        self.console.reset_all()?;
        print(&item[end..])?;

        // Regardless of what happened up there, reset all graphic attributes
        self.console.reset_all()?;

        Ok(())
    }

    /// Print out the current matcheson the line below the current one, restoring the cursor
    /// position afterwards
    fn print_items(&mut self) -> io::Result<()> {
        self.console.save_caret_position()?;
        self.console.move_down()?;

        let matches = self.window.apply(self.selector.matches());
        let match_amount = matches.len();

        if self.selected >= match_amount {
            self.selected = match_amount.saturating_sub(1);
        }

        for (index, match_) in matches.iter().enumerate() {
            // Erase any leftovers in the line
            self.print_match(index == self.selected, match_)?;
            self.console.move_down()?;
        }

        // Clear out any leftover lines
        for _ in 0..(self.match_amount.saturating_sub(match_amount)) {
            self.console.erase_line()?;
            self.console.move_down()?;
        }
        self.match_amount = match_amount;

        self.console.restore_caret_position()?;

        io::stdout().flush()?;

        Ok(())
    }

    pub fn mainloop(&mut self) -> io::Result<()> {
        {
            self.print_prompt()?;
            self.print_items()?;

            let stdout = io::stdout();

            let mut pattern = String::new();
            loop {
                let c = self.console.read_one_char()?;

                // this lock is reentrant, we can hold it as many times as we want
                let mut stdout_lock = stdout.lock();

                match c {
                    // If the user inputs Ctrl-C ...
                    Console::CTRL_C => {
                        // ... bail out!
                        break;
                    }

                    // If the user inputs a backspace ...
                    Console::BACKSPACE => {
                        // ... remove the latest character and relay the change to the selector ...
                        pattern.pop();
                        self.selector.set_pattern(&pattern);

                        // ... then clear out the prompt line ...
                        self.console.erase_line()?;

                        // ... and print it out again ...
                        self.print_prompt()?;

                        write!(stdout_lock, "{}", pattern)?;
                        stdout_lock.flush()?;

                        // ... then print out the new matches
                        self.print_items()?;
                    }

                    // ESCape sequence
                    //
                    // Arrow keys are represented as \033[ followed by any of A, B, C or D that each
                    // correspond to up, down, right and left
                    Console::ESC => {
                        assert_eq!(self.console.read_one_char()?, b'[');

                        let matches = self.window.apply(self.selector.matches());

                        // If we've pressed an arrow, vary the selected index accordingly ...
                        match self.console.read_one_char()? {
                            // We're going vertically
                            c @ b'A' | c @ b'B' => {
                                // Draw the previously selected line as unselected
                                self.console.save_caret_position()?;
                                self.console.move_down_n(self.selected + 1)?;
                                self.print_match(false, &matches[self.selected])?;
                                self.console.restore_caret_position()?;

                                // Move the selection
                                if c == b'A' {
                                    // We're going up
                                    if self.selected == 0 {
                                        // If we're already at the top of the screen, scroll up
                                        self.window.scroll_up();
                                        self.print_items()?;
                                        continue;
                                    }

                                    self.selected -= 1;
                                } else {
                                    // We're going down
                                    // If we're already at the end of the list, scroll down
                                    if self.selected == self.match_amount.saturating_sub(1) {
                                        self.window.scroll_down();
                                        self.print_items()?;
                                        continue;
                                    }

                                    self.selected += 1;
                                }

                                // Draw the new selected line
                                self.console.save_caret_position()?;
                                self.console.move_down_n(self.selected + 1)?;
                                self.print_match(true, &matches[self.selected])?;
                                self.console.restore_caret_position()?;

                                // Update the display
                                io::stdout().flush()?;
                            }

                            _ => {}
                        }
                    }

                    // Enter
                    b'\r' => {
                        break;
                    }

                    // If the character is printable ...
                    c if c >= b' ' && c <= b'~' => {
                        // ... push it to the pattern and relay the change to the selector ...
                        pattern.push(c as char);
                        self.selector.set_pattern(&pattern);

                        // ... echo it to the user ...
                        write!(stdout_lock, "{}", c as char)?;
                        stdout_lock.flush()?;

                        // ... and print out the new matches
                        self.print_items()?;
                    }

                    // Any other control characters are ignored
                    c => eprintln!("control char {:?}", c),
                }
            }
        } // exit raw mode

        if let Some(Match { item, .. }) = self.selector.matches().get(self.selected) {
            self.console.erase_line()?;
            self.console.save_caret_position()?;
            for _ in 0..self.match_amount {
                self.console.move_down()?;
                self.console.erase_line()?;
            }
            self.console.restore_caret_position()?;
            writeln!(self.console, "{}", item)?;
        }

        Ok(())
    }
}
