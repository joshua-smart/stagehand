use std::fmt::Display;

use crate::data_structures::CHANNELS_PER_UNIVERSE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index(u16);

impl Index {
    pub const fn new(index: u16) -> Option<Index> {
        if index < 1 || index as usize > CHANNELS_PER_UNIVERSE {
            None
        } else {
            Some(Self(index))
        }
    }

    pub fn range() -> impl Iterator<Item = Index> {
        (1..=CHANNELS_PER_UNIVERSE)
            .map(|i| Index::new(i as u16).expect("should always be valid index"))
    }

    pub const MIN: Index = Index::new(1).expect("index 1 is valid");
    pub const MAX: Index = Index::new(CHANNELS_PER_UNIVERSE as u16).expect("index 512 is valid");
}

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Index> for u16 {
    fn from(value: Index) -> Self {
        value.0
    }
}
