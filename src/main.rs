mod selector;

mod ui;

fn main() {
    let words_contents = std::fs::read_to_string("/usr/share/dict/words").unwrap();

    let words = words_contents.lines().collect::<Vec<&str>>();

    ui::Display::new(selector::Selector::new(
        selector::SelectorMode::FixedString,
        words.as_slice(),
    ))
    .mainloop()
    .unwrap();
}
