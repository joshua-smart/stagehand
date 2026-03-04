use std::fmt::Display;

use sacn::packet::E131_MAX_MULTICAST_UNIVERSE;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Deserialize, Serialize, Hash)]
#[serde(try_from = "u16")]
#[serde(into = "u16")]
pub struct Universe(u16);

impl Universe {
    pub const fn new(universe: u16) -> Option<Self> {
        if universe < 1 || universe > E131_MAX_MULTICAST_UNIVERSE {
            None
        } else {
            Some(Self(universe))
        }
    }

    pub const ONE: Universe = Universe::new(1).expect("universe 1 is valid");
}

#[derive(Debug, thiserror::Error)]
pub enum IntoUniverseError {
    #[error("universe {0} is out of bounds")]
    OutOfBounds(u16),
}

impl TryFrom<u16> for Universe {
    type Error = IntoUniverseError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Universe::new(value).ok_or(IntoUniverseError::OutOfBounds(value))
    }
}

impl From<Universe> for u16 {
    fn from(value: Universe) -> Self {
        value.0
    }
}

impl Display for Universe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
