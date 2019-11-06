use crate::selector::{Match, Selector};

use std::io;

mod ansi;
mod termios;

pub struct Display {
    prompt: String,
    selector: Box<dyn Selector>,
    match_amount: Option<usize>,
}

impl Display {
    pub fn new(selector: Box<dyn Selector>) -> Self {
        Self {
            prompt: "> ".to_owned(),
            selector,
            match_amount: None,
        }
    }

    fn print_prompt(&self) -> io::Result<()> {
        print!("{}", self.prompt);
        Ok(())
    }

    fn print_items(&mut self) -> io::Result<()> {
        let matches = self.selector.get_matches();
        let match_amount = matches.len();
        for item in matches {
            print!("{}", item.item);
            ansi::cursor_next_line(1)?;
        }

        if let Some(old_match_amount) = self.match_amount {
            if old_match_amount > match_amount {
                for _ in 0..old_match_amount - match_amount {
                    ansi::erase_in_line(2)?;
                    ansi::cursor_next_line(1)?;
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
        ansi::save_cursor_position()?;

        // Print out the list of items on the line after the prompt
        ansi::cursor_next_line(1)?;
        self.print_items()?;

        // And make the user type at the end of the prompt
        ansi::restore_cursor_position()?;

        let mut pattern = String::new();
        loop {
            let c = self.read_char()?;

            match c {
                // If the user inputs Ctrl-C, bail out
                3 => {
                    break;
                }

                _ => {
                    pattern.push(c as char);
                    self.selector.set_pattern(&pattern);

                    print!("{}", c as char);
                    io::stdout().flush()?;

                    ansi::save_cursor_position()?;
                    ansi::cursor_next_line(1)?;
                    self.print_items()?;
                    ansi::restore_cursor_position()?;
                }
            }
        }

        Ok(())
    }
}
