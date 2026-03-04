use std::fmt::Display;

use crate::data_structures::{index::Index, universe::Universe};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address {
    pub universe: Universe,
    pub index: Index,
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", u16::from(self.universe), u16::from(self.index))
    }
}
#[derive(Debug)]
pub struct AddressRange {
    universe: Universe,
    start: Index,
    end: Index,
    step: Option<usize>,
}

impl AddressRange {
    pub fn new(universe: Universe, start: Index, end: Index) -> Option<Self> {
        if start > end {
            None
        } else {
            Some(Self {
                universe,
                start,
                end,
                step: None,
            })
        }
    }

    pub fn with_step(universe: Universe, start: Index, end: Index, step: usize) -> Option<Self> {
        if step == 0 {
            None
        } else {
            Self::new(universe, start, end).map(|base| Self {
                step: Some(step),
                ..base
            })
        }
    }

    pub fn universe(&self) -> Universe {
        self.universe
    }

    pub fn indexes(&self) -> impl Iterator<Item = Index> {
        (u16::from(self.start)..=u16::from(self.end))
            .step_by(self.step.unwrap_or(1))
            .map(|i| {
                Index::new(i).expect(
                "should always be valid as is generated from a range between two validated indexes",
            )
            })
    }
}

#[derive(Debug)]
pub enum AddressSet {
    Single(Address),
    Range(AddressRange),
}

impl AddressSet {
    pub fn universe(&self) -> Universe {
        match self {
            AddressSet::Single(Address { universe, index: _ }) => *universe,
            AddressSet::Range(address_range) => address_range.universe(),
        }
    }

    pub fn indexes(&self) -> Vec<Index> {
        match self {
            AddressSet::Single(Address { universe: _, index }) => vec![*index],
            AddressSet::Range(address_range) => address_range.indexes().collect(),
        }
    }

    pub fn all(universe: Universe) -> Self {
        AddressSet::Range(AddressRange::new(universe, Index::MIN, Index::MAX).unwrap())
    }
}
