mod player;
mod tile;

mod card_and_back;
mod debug_label;
mod player_block;
mod select_move;
mod sound_effect;
mod utils;

use player::Player;
use tile::Tile;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

use bevy::prelude::*;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                debug_label::populate,
                utils::populate_board,
                card_and_back::compute_neighborhoods,
                sound_effect::populate,
            )
                .chain(),
        );
        app.add_systems(
            Update,
            (
                card_and_back::update_counts_and_playable_tiles,
                select_move::update,
                select_move::click_move,
                card_and_back::play_and_resolve_move,
                card_and_back::update_backs,
                select_move::animate,
                player_block::animate_labels,
                player_block::animate_crowns,
                debug_label::animate,
                card_and_back::animate_backs,
                card_and_back::animate_cards,
            )
                .chain(),
        );

        app.add_systems(
            OnEnter(BoardState::Victory(Player::One)),
            sound_effect::play_yeah,
        );
        app.add_systems(
            OnEnter(BoardState::Victory(Player::Two)),
            sound_effect::play_yeah,
        );
        app.add_systems(
            OnEnter(BoardState::ResolvingMove(Player::One)),
            sound_effect::play_collision,
        );
        app.add_systems(
            OnEnter(BoardState::ResolvingMove(Player::Two)),
            sound_effect::play_collision,
        );

        app.init_resource::<BoardResource>();
        app.init_resource::<sound_effect::SoundEffectResource>();
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
    num_resolved_moves: usize,
}

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
enum BoardState {
    #[default]
    Init,
    WaitingForMove(Player),
    PlayingMove(Player, Tile),
    ResolvingMove(Player),
    Victory(Player),
}
