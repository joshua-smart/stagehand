use std::fmt::Display;

#[derive(Debug, Copy, Clone)]
pub struct Level(u8);

impl Level {
    pub const fn new(level: u8) -> Self {
        Self(level)
    }

    pub const OUT: Level = Level::new(u8::MIN);
    pub const FULL: Level = Level::new(u8::MAX);
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            u8::MIN => f.write_str("O"),
            u8::MAX => f.write_str("F"),
            l => write!(f, "{l}"),
        }
    }
}

impl From<Level> for u8 {
    fn from(value: Level) -> Self {
        value.0
    }
}

#[derive(Debug)]
pub enum LevelSet {
    Single(Level),
    Range(LevelRange),
}

#[derive(Debug)]
pub struct LevelRange {
    pub start: Level,
    pub end: Level,
}

impl LevelRange {
    pub fn interpolate(&self, t: f64) -> Level {
        let interval_length = u8::from(self.end) as f64 - u8::from(self.start) as f64;
        let level = u8::from(self.start) as f64 + (interval_length * t);
        Level::new(level.round() as u8)
    }
}
