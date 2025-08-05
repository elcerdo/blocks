mod debug_label;
mod player_block;

mod anim;
mod make;

mod player;
mod tile;

use player::Player;
use tile::Tile;

use bevy::prelude::*;

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, debug_label::populate);
        app.add_systems(Startup, make::populate_items);
        app.add_systems(PostStartup, make::compute_neighborhoods);
        app.add_systems(
            Update,
            (
                update_counts_and_playable_tiles,
                update_selects,
                click_selects,
                play_selects,
                update_backs,
                player_block::animate,
                debug_label::animate,
                anim::animate_backs,
                anim::animate_cards,
                anim::animate_selects,
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

#[derive(Component, Clone, Debug)]
pub struct UiCard {
    pub tile: Tile,
    pub row: usize,
    pub column: usize,
}

#[derive(Component)]
pub struct UiBack {
    pub player: Player,
}

#[derive(Component)]
pub struct UiSelect {
    pub tile: Tile,
    pub playable: bool,
}

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

#[derive(Clone, Eq, PartialEq, PartialOrd)]
struct Priority {
    distance: usize,
    player: Player,
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.distance.cmp(&other.distance).reverse() {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => self.player.cmp(&other.player).reverse(),
        }
    }
}

fn update_counts_and_playable_tiles(
    ui_backs: Query<&UiBack>,
    ui_cards: Query<(&UiCard, Entity)>,
    mut board: ResMut<BoardResource>,
) {
    let mut player_to_counts = BTreeMap::new();
    for ui_back in ui_backs.iter() {
        if !player_to_counts.contains_key(&ui_back.player) {
            player_to_counts.insert(ui_back.player.clone(), 0);
        }
        *player_to_counts.get_mut(&ui_back.player).unwrap() += 1;
    }
    board.player_to_counts = player_to_counts;

    let mut player_to_playable_tiles = BTreeMap::new();
    player_to_playable_tiles.insert(Player::One, BTreeSet::new());
    player_to_playable_tiles.insert(Player::Two, BTreeSet::new());
    for (ui_card, card_entity) in ui_cards.iter() {
        let back_entity = *board.card_to_backs.get(&card_entity).unwrap();
        let ui_back = ui_backs.get(back_entity).unwrap();
        if let Some(playable_tiles) = player_to_playable_tiles.get_mut(&ui_back.player) {
            assert!(ui_card.tile != Tile::Undef);
            for card_entity_ in board.card_to_neighbors.get(&card_entity).unwrap() {
                let card_entity_ = *card_entity_;
                let back_entity_ = *board.card_to_backs.get(&card_entity_).unwrap();
                let ui_card_ = ui_cards.get(card_entity_).unwrap().0;
                let ui_back_ = ui_backs.get(back_entity_).unwrap();
                if ui_back_.player != ui_back.player && ui_card_.tile != Tile::Undef {
                    assert!(ui_card.tile != ui_card_.tile);
                    playable_tiles.insert(ui_card_.tile.clone());
                }
            }
        }
    }
    board.player_to_playable_tiles = player_to_playable_tiles;
}

fn update_selects(
    mut ui_selects: Query<&mut UiSelect>,
    board: Res<BoardResource>,
    state: Res<State<BoardState>>,
) {
    assert!(board.select_red_card.is_some());
    assert!(board.select_green_card.is_some());
    assert!(board.select_blue_card.is_some());

    let playable_tiles = if let BoardState::WaitingForMove(player) = state.get() {
        match board.player_to_playable_tiles.get(player) {
            Some(playable_tiles) => playable_tiles.clone(),
            None => BTreeSet::new(),
        }
    } else {
        BTreeSet::new()
    };

    let select_cards = [
        board.select_red_card.unwrap(),
        board.select_green_card.unwrap(),
        board.select_blue_card.unwrap(),
    ];
    for select_card in select_cards {
        let mut select_card = ui_selects.get_mut(select_card).unwrap();
        select_card.playable = playable_tiles.contains(&select_card.tile);
    }
}

fn click_selects(
    ui_selects: Query<(&UiSelect, &Interaction), (Changed<Interaction>, With<Button>)>,
    state: Res<State<BoardState>>,
    mut next_state: ResMut<NextState<BoardState>>,
) {
    if let BoardState::WaitingForMove(player) = state.get() {
        for (ui_select, interaction) in ui_selects {
            if matches!(interaction, Interaction::Pressed) {
                next_state.set(BoardState::SelectedMove(
                    player.clone(),
                    ui_select.tile.clone(),
                ));
            }
        }
    }
}

fn play_selects(
    mut ui_cards: Query<&mut UiCard>,
    board: Res<BoardResource>,
    state: Res<State<BoardState>>,
    mut next_state: ResMut<NextState<BoardState>>,
) {
    assert!(board.player_one_card.is_some());
    assert!(board.player_two_card.is_some());
    assert!(!board.card_to_neighbors.is_empty());
    assert!(!board.card_to_backs.is_empty());
    assert!(board.card_to_backs.len() == board.card_to_neighbors.len());

    if let BoardState::SelectedMove(player, tile) = state.get() {
        let player_card = match player {
            Player::One => board.player_one_card.unwrap(),
            Player::Two => board.player_two_card.unwrap(),
            _ => unreachable!(),
        };

        let mut done = HashSet::new();
        let mut queue = priority_queue::PriorityQueue::new();
        queue.push(
            player_card,
            Priority {
                distance: 0,
                player: player.clone(),
            },
        );
        let current_tile = ui_cards.get(player_card).unwrap().tile.clone();
        while let Some((current_card, current_priority)) = queue.pop() {
            assert!(!done.contains(&current_card));

            let next_cards = board.card_to_neighbors.get(&current_card).unwrap();
            for next_card in next_cards {
                if done.contains(next_card) {
                    continue;
                }
                let next_tile = ui_cards.get(*next_card).unwrap().tile.clone();
                if next_tile != current_tile {
                    continue;
                }
                let mut next_priority = current_priority.clone();
                next_priority.distance += 1;
                queue.push(*next_card, next_priority);
            }

            let mut ui_card = ui_cards.get_mut(current_card).unwrap();
            ui_card.tile = tile.clone();

            done.insert(current_card);
        }

        let next_player = match player {
            Player::One => Player::Two,
            Player::Two => Player::One,
            _ => unreachable!(),
        };
        next_state.set(BoardState::WaitingForMove(next_player));
    }
}

fn update_backs(
    mut ui_backs: Query<&mut UiBack>,
    ui_cards: Query<&UiCard>,
    board: Res<BoardResource>,
) {
    assert!(board.player_one_card.is_some());
    assert!(board.player_two_card.is_some());
    assert!(!board.card_to_neighbors.is_empty());
    assert!(!board.card_to_backs.is_empty());
    assert!(board.card_to_backs.len() == board.card_to_neighbors.len());

    for mut ui_back in ui_backs.iter_mut() {
        ui_back.player = Player::Undef;
    }

    let player_one_card = board.player_one_card.unwrap();
    let player_two_card = board.player_two_card.unwrap();

    let mut done = HashSet::new();
    let mut queue = priority_queue::PriorityQueue::new();
    queue.push(
        player_one_card,
        Priority {
            distance: 0,
            player: Player::One,
        },
    );
    queue.push(
        player_two_card,
        Priority {
            distance: 0,
            player: Player::Two,
        },
    );
    while let Some((current_card, current_priority)) = queue.pop() {
        assert!(!done.contains(&current_card));

        let current_tile = ui_cards.get(current_card).unwrap().tile.clone();
        let next_cards = board.card_to_neighbors.get(&current_card).unwrap();
        for next_card in next_cards {
            if done.contains(next_card) {
                continue;
            }
            let next_tile = ui_cards.get(*next_card).unwrap().tile.clone();
            if next_tile != current_tile {
                continue;
            }
            let mut next_priority = current_priority.clone();
            next_priority.distance += 1;
            queue.push(*next_card, next_priority);
        }

        let ui_back = board.card_to_backs.get(&current_card).unwrap();
        let mut ui_back = ui_backs.get_mut(*ui_back).unwrap();
        ui_back.player = current_priority.player.clone();

        done.insert(current_card);
    }
}
