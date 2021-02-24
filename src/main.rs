use std::{
    fs,
    io::{self, prelude::*},
    path::PathBuf,
};

use structopt::StructOpt;

mod selector;

mod sliding_window;
mod ui;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "fuzzy")]
    mode: selector::SelectorMode,

    #[structopt(parse(from_os_str), default_value = "-")]
    words: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    let contents = if opt.words == PathBuf::from("-") {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap();
        buf
    } else {
        fs::read_to_string(opt.words).unwrap()
    };

    let words = contents.lines().collect::<Vec<&str>>();

    ui::UI::new(selector::Selector::new(opt.mode, &words))
        .unwrap()
        .mainloop()
        .unwrap();
}
