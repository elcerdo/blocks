use bevy::color::Srgba;
use bevy::color::palettes::css::*;

pub const TILE_COLOR_DATA: &[(Srgba, Srgba, usize)] = &[
    (LIGHT_GREY, BLACK, 26),
    (PINK, RED, 25),
    (LIGHT_GREEN, GREEN, 0),
    (LIGHT_BLUE, BLUE, 27),
    (LIGHT_YELLOW, ORANGE, 23),
];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tile {
    Undef,
    Red,
    Green,
    Blue,
    Yellow,
}

impl Into<usize> for Tile {
    fn into(self) -> usize {
        match self {
            Tile::Undef => 0usize,
            Tile::Red => 1,
            Tile::Green => 2,
            Tile::Blue => 3,
            Tile::Yellow => 4,
        }
    }
}

impl From<usize> for Tile {
    fn from(index: usize) -> Self {
        match index % TILE_COLOR_DATA.len() {
            0 => Tile::Undef,
            1 => Tile::Red,
            2 => Tile::Green,
            3 => Tile::Blue,
            4 => Tile::Yellow,
            _ => unreachable!(),
        }
    }
}
