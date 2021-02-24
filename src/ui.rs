use std::{
    cmp::{max, min},
    io::{self, prelude::*},
};

use crate::selector::{Match, Selector};
use crate::sliding_window::SlidingWindow;

use crossterm::{
    cursor::{MoveToColumn, MoveToNextLine, MoveToPreviousLine, RestorePosition, SavePosition},
    event::{Event, KeyCode, KeyModifiers},
    queue,
    style::{Attribute, Color, ContentStyle, Print, PrintStyledContent},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};

pub struct UI<'a> {
    prompt: String,

    selector: Selector<'a>,
    match_amount: usize,

    selected: usize,

    window: SlidingWindow,

    selected_style: ContentStyle,
    highlight_style: ContentStyle,
}

fn merge(a: ContentStyle, b: ContentStyle) -> ContentStyle {
    ContentStyle {
        foreground_color: b.foreground_color.or(a.foreground_color),
        background_color: b.background_color.or(a.background_color),
        attributes: a.attributes | b.attributes,
    }
}

fn calculate_window_size() -> crossterm::Result<usize> {
    let (_, row) = crossterm::cursor::position()?;
    let (_, h) = crossterm::terminal::size()?;

    let below = (h - (row + 1)) as usize;

    let stderr_lock = io::stderr();
    let mut stderr = stderr_lock.lock();
    for _ in below..2 {
        queue!(stderr, MoveToPreviousLine(1), Clear(ClearType::CurrentLine))?;
    }
    stderr.flush()?;

    Ok(min(max(below, 2), 20))
}

impl<'a> UI<'a> {
    pub fn new(selector: Selector<'a>) -> crossterm::Result<Self> {
        Ok(Self {
            prompt: "> ".to_owned(),

            selector,
            match_amount: 0,

            selected: 0,

            window: SlidingWindow::new(calculate_window_size()?),

            selected_style: ContentStyle::new().background(Color::AnsiValue(1)),
            highlight_style: ContentStyle::new()
                .attribute(Attribute::Bold)
                .attribute(Attribute::Underlined),
        })
    }

    fn print_prompt(&mut self) -> crossterm::Result<()> {
        queue!(io::stderr(), Print(&self.prompt))
    }

    /// Print out a match, taking care of highlighting, on the current line
    fn print_match(
        &self,
        selected: bool,
        Match { item, highlight }: &Match<'a>,
    ) -> crossterm::Result<()> {
        let stderr_lock = io::stderr();
        let mut stderr = stderr_lock.lock();

        // Erase anything that's in the line
        queue!(stderr, Clear(ClearType::CurrentLine), MoveToColumn(0))?;

        let mut print = move |style: Option<ContentStyle>, s| -> crossterm::Result<()> {
            let style = merge(
                style.unwrap_or(ContentStyle::new()),
                if selected {
                    self.selected_style
                } else {
                    ContentStyle::new()
                },
            );

            queue!(stderr, PrintStyledContent(style.apply(s)))
        };

        let end =
            highlight
                .iter()
                .try_fold(0, |last, &(start, end)| -> crossterm::Result<usize> {
                    // Print out the stuff between highlight groups normally
                    print(None, &item[last..start])?;

                    // Print the inside of the group with the highlight style
                    print(Some(self.highlight_style), &item[start..end])?;

                    // Pass on the ball
                    Ok(end)
                })?;

        // Print out what's leftover normally
        print(None, &item[end..])?;

        Ok(())
    }

    /// Print out the current matcheson the line below the current one, restoring the cursor
    /// position afterwards
    fn print_items(&mut self) -> crossterm::Result<()> {
        let stderr_lock = io::stderr();
        let mut stderr = stderr_lock.lock();

        queue!(stderr, SavePosition, MoveToNextLine(1))?;

        let matches = self.window.apply(self.selector.matches());
        let match_amount = matches.len();

        if self.selected >= match_amount {
            self.selected = match_amount.saturating_sub(1);
        }

        for (index, match_) in matches.iter().enumerate() {
            // Erase any leftovers in the line
            self.print_match(index == self.selected, match_)?;
            queue!(stderr, MoveToNextLine(1))?;
        }

        // Clear out any leftover lines
        for _ in 0..(self.match_amount.saturating_sub(match_amount)) {
            queue!(stderr, Clear(ClearType::CurrentLine), MoveToNextLine(1))?;
        }
        self.match_amount = match_amount;

        queue!(stderr, RestorePosition)?;

        stderr.flush()?;

        Ok(())
    }

    pub fn mainloop(mut self) -> crossterm::Result<()> {
        enable_raw_mode()?;

        let stderr_lock = io::stderr();
        let mut stderr = stderr_lock.lock();

        self.print_prompt()?;
        self.print_items()?;

        let mut pattern = String::new();
        loop {
            let key = match crossterm::event::read()? {
                Event::Key(evt) => evt,
                Event::Mouse(_) | Event::Resize(_, _) => continue,
            };

            match key.code {
                KeyCode::Enter | KeyCode::Esc => {
                    break;
                }

                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    break;
                }

                // If the user inputs a backspace ...
                KeyCode::Backspace => {
                    // ... remove the latest character and relay the change to the selector ...
                    pattern.pop();
                    self.selector.set_pattern(&pattern);

                    // ... then clear out the prompt line ...
                    queue!(stderr, Clear(ClearType::CurrentLine), MoveToColumn(0))?;

                    // ... and print it out again ...
                    self.print_prompt()?;

                    queue!(stderr, Print(&pattern))?;

                    // ... then print out the new matches
                    self.print_items()?;

                    stderr.flush()?;
                }

                key @ KeyCode::Up | key @ KeyCode::Down => {
                    let matches = self.window.apply(self.selector.matches());

                    if matches.is_empty() {
                        continue;
                    }

                    // Draw the previously selected line as unselected
                    queue!(
                        io::stderr(),
                        SavePosition,
                        MoveToNextLine((self.selected + 1) as u16)
                    )?;
                    self.print_match(false, &matches[self.selected])?;
                    queue!(io::stderr(), RestorePosition)?;

                    // Move the selection
                    if key == KeyCode::Up {
                        // We're going up
                        if self.selected == 0 {
                            // If we're already at the top of the screen, scroll up
                            self.window.scroll_up();
                            self.print_items()?;
                            stderr.flush()?;
                            continue;
                        }

                        self.selected -= 1;
                    } else {
                        // We're going down
                        // If we're already at the end of the list, scroll down
                        if self.selected == self.match_amount.saturating_sub(1) {
                            self.window.scroll_down();
                            self.print_items()?;
                            stderr.flush()?;
                            continue;
                        }

                        self.selected += 1;
                    }

                    // Draw the new selected line
                    queue!(
                        stderr,
                        SavePosition,
                        MoveToNextLine((self.selected + 1) as u16)
                    )?;
                    self.print_match(true, &matches[self.selected])?;
                    queue!(stderr, RestorePosition)?;

                    // Update the display
                    stderr.flush()?;
                }

                // If the character is printable ...
                KeyCode::Char(ch) => {
                    // ... push it to the pattern and relay the change to the selector ...
                    pattern.push(ch);
                    self.selector.set_pattern(&pattern);

                    // ... echo it to the user ...
                    queue!(stderr, Print(ch))?;

                    // ... and print out the new matches
                    self.print_items()?;

                    stderr.flush()?;
                }

                KeyCode::Left
                | KeyCode::Right
                | KeyCode::Home
                | KeyCode::End
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Tab
                | KeyCode::BackTab
                | KeyCode::Delete
                | KeyCode::Insert
                | KeyCode::F(_)
                | KeyCode::Null => {}
            }
        }

        disable_raw_mode()?;

        if let Some(Match { item, .. }) =
            (self.window.apply(self.selector.matches())).get(self.selected)
        {
            queue!(
                stderr,
                Clear(ClearType::CurrentLine),
                MoveToColumn(0),
                SavePosition
            )?;
            for _ in 0..self.match_amount {
                queue!(stderr, MoveToNextLine(1), Clear(ClearType::CurrentLine))?;
            }
            queue!(stderr, RestorePosition)?;
            stderr.flush()?;

            // NB: This prints to stdout, not to the console
            println!("{}", item);
        }

        Ok(())
    }
}
