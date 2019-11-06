mod selector;

mod ui;

fn main() {
    use selector::Selector;

    let mut selector = selector::FixedStringSelector::default();
    selector.set_items(vec![
        "a".to_owned(),
        "boo".to_owned(),
        "foo".to_owned(),
        "gloo".to_owned(),
    ]);

    ui::Display::new(Box::new(selector)).mainloop().unwrap();
}
