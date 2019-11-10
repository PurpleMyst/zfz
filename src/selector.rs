#[derive(Debug, Clone, Copy)]
pub enum SelectorMode {
    FixedString,
}

#[derive(Debug)]
pub struct Selector<'a> {
    mode: SelectorMode,

    /// All of the items
    items: &'a [&'a str],

    /// A vector of matches, which are represented as an index into items and a range
    matches: Vec<Match<'a>>,
}

#[derive(Debug)]
pub struct Match<'a> {
    pub item: &'a str,
    pub highlight: Vec<(usize, usize)>,
}

impl<'a> Selector<'a> {
    pub fn new(mode: SelectorMode, items: &'a [&'a str]) -> Self {
        let mut this = Self {
            mode,
            items,
            matches: Default::default(),
        };
        this.set_pattern("");
        this
    }

    pub fn matches(&'a self) -> &'a [Match<'a>] {
        self.matches.as_ref()
    }

    pub fn set_pattern(&mut self, pattern: &str) {
        self.matches = match self.mode {
            SelectorMode::FixedString => self
                .items
                .iter()
                .filter_map(|item| {
                    let mut pattern_chars = pattern.chars();
                    let mut end = 0;

                    for (idx, c) in item.chars().enumerate() {
                        if let Some(pattern_c) = pattern_chars.next() {
                            if pattern_c != c {
                                pattern_chars = pattern.chars();
                            } else {
                                end = idx + 1;
                            }
                        } else {
                            break;
                        }
                    }

                    if pattern_chars.next().is_none() {
                        Some(Match {
                            item,
                            highlight: vec![(end - pattern.len(), end)],
                        })
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

// FIXME: write tests
