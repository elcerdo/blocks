mod card_and_back;
mod debug_label;
mod player_block;
mod select_move;

mod anim;
mod utils;

mod player;
mod tile;

use player::Player;
use tile::Tile;

use bevy::prelude::*;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, debug_label::populate);
        app.add_systems(Startup, utils::populate_items);
        app.add_systems(PostStartup, utils::compute_neighborhoods);
        app.add_systems(
            Update,
            (
                card_and_back::update_counts_and_playable_tiles,
                select_move::update,
                select_move::click_move,
                card_and_back::play_move,
                card_and_back::update_backs,
                select_move::animate,
                player_block::animate,
                debug_label::animate,
                anim::animate_backs,
                anim::animate_cards,
            )
                .chain(),
        );
        app.init_resource::<BoardResource>();
        app.init_state::<BoardState>();
    }
}

const BOARD_WIDTH: usize = 14;
const BOARD_HEIGHT: usize = 7;
const BOARD_SEED: usize = 0xabf8f1afb5;

#[derive(Resource, Default)]
struct BoardResource {
    player_one_card: Option<Entity>,
    player_two_card: Option<Entity>,
    select_red_card: Option<Entity>,
    select_green_card: Option<Entity>,
    select_blue_card: Option<Entity>,
    card_to_neighbors: HashMap<Entity, HashSet<Entity>>,
    card_to_backs: HashMap<Entity, Entity>,
    player_to_counts: BTreeMap<Player, usize>,
    player_to_playable_tiles: BTreeMap<Player, BTreeSet<Tile>>,
}

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
enum BoardState {
    #[default]
    Init,
    WaitingForMove(Player),
    SelectedMove(Player, Tile),
}
