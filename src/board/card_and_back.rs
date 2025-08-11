use bevy::prelude::*;

use super::BoardResource;
use super::BoardState;
use super::Player;
use super::Tile;
use super::Direction;

use super::player::PLAYER_COLOR_DATA;
use super::tile::TILE_COLOR_DATA;

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Component)]
pub struct UiCard {
    pub tile: Tile,
    pub row: usize,
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

pub fn make_pair(
    texture_border: &Handle<Image>,
    atlas_layout_border: &Handle<TextureAtlasLayout>,
    slicer: &TextureSlicer,
    parent: &mut ChildSpawnerCommands,
    tile: Tile,
    row: usize,
    column: usize,
) -> (Entity, Entity) {
    let ui_card = UiCard { tile, row, column };
    let ui_back = UiBack {
        player: Player::Undef,
    };

    let tile_index: usize = ui_card.tile.clone().into();
    let (bg_color, fg_color, atlas_index) = TILE_COLOR_DATA[tile_index].clone();
    let bg_color: Color = bg_color.into();
    let fg_color: Color = fg_color.into();

    let mut card_entity = None;
    let mut back = parent.spawn((
        ui_back,
        Node {
            width: Val::Px(70.0),
            height: Val::Px(70.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            margin: UiRect::all(Val::Px(0.0)),
            border: UiRect::all(Val::Px(0.0)),
            padding: UiRect::all(Val::Px(0.0)),
            ..default()
        },
        BackgroundColor::default(),
    ));

    back.with_children(|parent| {
        let mut card = parent.spawn((
            Button,
            ui_card,
            Node {
                width: Val::Px(64.0),
                height: Val::Px(64.0),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexStart,
                margin: UiRect::all(Val::Px(2.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(bg_color.clone()),
            BorderColor(fg_color.clone()),
            BorderRadius::all(Val::Px(8.0)),
        ));

        card.with_children(|parent| {
            let _button = parent.spawn((
                ImageNode::from_atlas_image(
                    texture_border.clone(),
                    TextureAtlas {
                        index: atlas_index,
                        layout: atlas_layout_border.clone(),
                    },
                )
                .with_color(fg_color.clone())
                .with_mode(NodeImageMode::Sliced(slicer.clone())),
                Node {
                    width: Val::Px(64.0 - 6.0),
                    height: Val::Px(64.0 - 6.0),
                    margin: UiRect::all(Val::Px(0.0)),
                    padding: UiRect::all(Val::Px(16.0)),
                    border: UiRect::all(Val::Px(0.0)),
                    ..default()
                },
            ));
        });

        card_entity = Some(card.id());
    });

    (card_entity.unwrap(), back.id())
}

pub fn compute_neighborhoods(
    ui_cards: Query<(&UiCard, Entity)>,
    mut board: ResMut<BoardResource>,
    mut next_state: ResMut<NextState<BoardState>>,
) {
    let mut coord_to_cards = HashMap::new();
    for (ui_card, card) in ui_cards.iter() {
        coord_to_cards.insert((ui_card.row, ui_card.column), card);
    }

    let mut card_to_neighbors = HashMap::new();
    for (ui_card, entity) in ui_cards.iter() {
        let mut neighbors = HashMap::new();
        if let Some(neighbor) = coord_to_cards.get(&(ui_card.row + 1, ui_card.column)) {
            neighbors.insert(Direction::South, *neighbor);
        }
        if let Some(neighbor) = coord_to_cards.get(&(ui_card.row - 1, ui_card.column)) {
            neighbors.insert(Direction::North,*neighbor);
        }
        if let Some(neighbor) = coord_to_cards.get(&(ui_card.row, ui_card.column + 1)) {
            neighbors.insert(Direction::East, *neighbor);
        }
        if let Some(neighbor) = coord_to_cards.get(&(ui_card.row, ui_card.column - 1)) {
            neighbors.insert(Direction::West, *neighbor);
        }
        card_to_neighbors.insert(entity, neighbors);
    }
    board.card_to_neighbors = card_to_neighbors;

    next_state.set(BoardState::WaitingForMove(Player::One));
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
            for (_, card_entity_) in board.card_to_neighbors.get(&card_entity).unwrap().iter() {
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

    {
        let mut remove_other_player_tile = |player: Player, player_: Player| {
            let card_entity = match player {
                Player::One => board.player_one_card.unwrap(),
                Player::Two => board.player_two_card.unwrap(),
                Player::Undef => unreachable!(),
            };
            let back_entity = *board.card_to_backs.get(&card_entity).unwrap();
            let ui_card = ui_cards.get(card_entity).unwrap().0;
            let ui_back = ui_backs.get(back_entity).unwrap();
            assert!(ui_back.player == player || ui_back.player == Player::Undef);
            let playable_tiles = player_to_playable_tiles.get_mut(&player_).unwrap();
            playable_tiles.remove(&ui_card.tile);
        };
        remove_other_player_tile(Player::One, Player::Two);
        remove_other_player_tile(Player::Two, Player::One);
    }

    for (player, playable_tiles) in player_to_playable_tiles.iter() {
        assert!(*player != Player::Undef);
        assert!(!playable_tiles.contains(&Tile::Undef));
    }
    board.player_to_playable_tiles = player_to_playable_tiles;
}

pub fn play_and_resolve_move(
    mut ui_cards: Query<&mut UiCard>,
    mut board: ResMut<BoardResource>,
    state: Res<State<BoardState>>,
    mut next_state: ResMut<NextState<BoardState>>,
) {
    assert!(board.player_one_card.is_some());
    assert!(board.player_two_card.is_some());
    assert!(!board.card_to_neighbors.is_empty());
    assert!(!board.card_to_backs.is_empty());
    assert!(board.card_to_backs.len() == board.card_to_neighbors.len());

    if let BoardState::PlayingMove(player, tile) = state.get() {
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

            for (_, next_card) in board.card_to_neighbors.get(&current_card).unwrap().iter() {
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

        next_state.set(BoardState::ResolvingMove(player.clone()));
    }

    if let BoardState::ResolvingMove(player) = state.get() {
        let next_player = match player {
            Player::One => Player::Two,
            Player::Two => Player::One,
            _ => unreachable!(),
        };
        board.num_resolved_moves += 1;
        let state =
            if let Some(next_playable_tiles) = board.player_to_playable_tiles.get(&next_player) {
                if !next_playable_tiles.is_empty() {
                    BoardState::WaitingForMove(next_player)
                } else {
                    BoardState::Victory(player.clone())
                }
            } else {
                BoardState::Victory(player.clone())
            };
        next_state.set(state);
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
        for (_, next_card) in board.card_to_neighbors.get(&current_card).unwrap() {
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

pub fn animate_backs(mut ui_backs: Query<(&UiBack, &mut BackgroundColor)>) {
    for (ui_back, mut back_color) in ui_backs.iter_mut() {
        let player_index: usize = ui_back.player.clone().into();
        let (bg_color, _) = PLAYER_COLOR_DATA[player_index].clone();
        *back_color = bg_color.into();
    }
}

pub fn animate_cards(
    mut ui_cards: Query<(&UiCard, &Children, &mut BorderColor, &mut BackgroundColor), With<Button>>,
    mut buttons: Query<&mut ImageNode>,
) {
    for (ui_card, children, mut border_color, mut back_color) in ui_cards.iter_mut() {
        let tile_index: usize = ui_card.tile.clone().into();
        let (bg_color, fg_color, atlas_index) = TILE_COLOR_DATA[tile_index].clone();
        let bg_color: Color = bg_color.into();
        let fg_color: Color = fg_color.into();
        *border_color = fg_color.into();
        *back_color = bg_color.into();
        for child in children {
            let mut button = buttons.get_mut(*child).unwrap();
            button.color = fg_color;
            button.texture_atlas.as_mut().unwrap().index = atlas_index;
        }
    }
}
