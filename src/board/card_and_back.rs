use bevy::prelude::*;

use super::BoardResource;
use super::BoardState;
use super::Player;
use super::State;
use super::Tile;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;

use std::cmp::Ordering;

#[derive(Component)]
pub struct UiCard {
    tile: Tile,
    row: usize,
    column: usize,
}

#[derive(Component)]
pub struct UiBack {
    player: Player,
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

pub fn update_counts_and_playable_tiles(
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

pub fn play_move(
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

pub fn update_backs(
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
