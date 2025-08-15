use bevy::color::Srgba;
use bevy::color::palettes::css::*;

use super::BOARD_HEIGHT;
use super::BOARD_SEED;
use super::BOARD_WIDTH;

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

impl Tile {
    pub fn from_row_and_column(row: usize, column: usize) -> Self {
        let ii = row * (BOARD_HEIGHT - 1 - row);
        let jj = column * (BOARD_WIDTH - 1 - column);
        let mut seed = BOARD_SEED;
        for _ in 0..16 {
            seed ^= ii + 0x9e3779b9 + (seed << 6) + (seed >> 2);
            seed ^= jj + 0x9e3779b9 + (seed << 6) + (seed >> 2);
        }
        if column * 2 < BOARD_WIDTH {
            match seed % (TILE_COLOR_DATA.len() - 1) {
                0 => Tile::Red,
                1 => Tile::Green,
                2 => Tile::Blue,
                3 => Tile::Yellow,
                _ => unreachable!(),
            }
        } else {
            match seed % (TILE_COLOR_DATA.len() - 1) {
                2 => Tile::Red,
                3 => Tile::Green,
                0 => Tile::Blue,
                1 => Tile::Yellow,
                _ => unreachable!(),
            }
        }
    }
}
