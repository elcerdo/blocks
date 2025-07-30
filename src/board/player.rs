use bevy::color::Srgba;
use bevy::color::palettes::css::*;

pub const PLAYER_COLOR_DATA: &[Srgba] = &[WHITE, ORANGE, YELLOW];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Player {
    Undef,
    One,
    Two,
}

impl Into<usize> for Player {
    fn into(self) -> usize {
        match self {
            Self::Undef => 0usize,
            Self::One => 1,
            Self::Two => 2,
        }
    }
}

impl From<usize> for Player {
    fn from(index: usize) -> Self {
        match index % PLAYER_COLOR_DATA.len() {
            0 => Self::Undef,
            1 => Self::One,
            2 => Self::Two,
            _ => unreachable!(),
        }
    }
}
