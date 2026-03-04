use sacn::packet::UNIVERSE_CHANNEL_CAPACITY;

pub mod address;
pub mod index;
pub mod level;
pub mod show;
pub mod universe;

pub const CHANNELS_PER_UNIVERSE: usize = UNIVERSE_CHANNEL_CAPACITY - 1;
