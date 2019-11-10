mod selector;

mod ui;

fn main() {
    ui::Display::new(selector::Selector::new(
        selector::SelectorMode::FixedString,
        &["a", "boo", "foo", "", "gloo"],
    ))
    .mainloop()
    .unwrap();
}
