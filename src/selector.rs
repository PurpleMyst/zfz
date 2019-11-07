pub struct Match<'a> {
    pub item: &'a str,
    pub highlight: Vec<(usize, usize)>,
}

pub trait Selector {
    fn get_matches(&self) -> Vec<Match>;
    fn set_items(&mut self, items: Vec<String>);
    fn set_pattern(&mut self, pattern: &str);
}

// TODO: optimize this a lot, this is just a rough draft
#[derive(Default)]
pub struct FixedStringSelector {
    /// All of the items
    items: Vec<String>,

    /// A vector of matches, which are represented as an index into items and a range
    matches: Vec<(usize, (usize, usize))>,
}

impl Selector for FixedStringSelector {
    fn set_items(&mut self, items: Vec<String>) {
        self.items = items;

        // Initially all items match
        self.matches = (0..self.items.len()).map(|idx| (idx, (0, 0))).collect();
    }

    fn get_matches(&self) -> Vec<Match> {
        self.matches
            .iter()
            .map(|&(idx, highlight)| Match {
                item: &self.items[idx],
                highlight: vec![highlight],
            })
            .collect()
    }

    fn set_pattern(&mut self, pattern: &str) {
        self.matches = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                if pattern.is_empty() {
                    return Some((idx, (0, 0)));
                }

                let mut pattern_chars = pattern.chars();
                let mut end = None;

                for (j, c) in item.chars().enumerate() {
                    if let Some(pattern_c) = pattern_chars.next() {
                        if pattern_c != c {
                            pattern_chars = pattern.chars();
                        } else {
                            end = Some(j);
                        }
                    } else {
                        break;
                    }
                }

                if pattern_chars.next().is_none() {
                    let end = end.unwrap() + 1;
                    Some((idx, (end - pattern.len(), end)))
                } else {
                    None
                }
            })
            .collect();
    }
}

// FIXME: write tests
