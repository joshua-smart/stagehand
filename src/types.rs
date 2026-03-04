use std::{collections::BTreeMap, fmt::Display, ops::Deref};

use sacn::packet::{E131_MAX_MULTICAST_UNIVERSE, UNIVERSE_CHANNEL_CAPACITY};

#[derive(Clone, Debug)]
pub struct State {
    pub levels: BTreeMap<Universe, [Level; UNIVERSE_CHANNEL_CAPACITY - 1]>,
}

impl Default for State {
    fn default() -> Self {
        let mut levels = BTreeMap::new();
        levels.insert(
            Universe::new(1).unwrap(),
            [Level::out(); UNIVERSE_CHANNEL_CAPACITY - 1],
        );
        Self { levels }
    }
}
