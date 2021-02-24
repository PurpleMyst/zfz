mod selector;

mod sliding_window;
mod ui;

fn main() {
    let words = include_str!("wordlist.txt").lines().collect::<Vec<&str>>();

    ui::UI::new(selector::Selector::new(
        selector::SelectorMode::FixedString,
        words.as_slice(),
    ))
    .unwrap()
    .mainloop()
    .unwrap();
}
