use std::str::FromStr;

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

#[derive(Debug, Clone, Copy)]
pub enum SelectorMode {
    FixedString,
    Fuzzy,
}

impl FromStr for SelectorMode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "fixed" => Ok(Self::FixedString),
            "fuzzy" => Ok(Self::Fuzzy),
            _ => Err("expected fixed or fuzzy"),
        }
    }
}

// TODO: It might be interesting to use Pin<_> to make this own its items.
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
            matches: Vec::new(),
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
                    item.find(pattern).map(|start| Match {
                        item,
                        highlight: vec![(start, start + pattern.len())],
                    })
                })
                .collect(),

            SelectorMode::Fuzzy => {
                let matcher = SkimMatcherV2::default();

                self.items
                    .iter()
                    .filter_map(|item| {
                        let (_, indices) = matcher.fuzzy_indices(item, pattern)?;

                        Some(Match {
                            item,
                            highlight: indices.into_iter().map(|idx| (idx, idx + 1)).collect(),
                        })
                    })
                    .collect()
            }
        }
    }
}

// FIXME: write tests
