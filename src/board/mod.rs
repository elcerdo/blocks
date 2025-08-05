mod player;
mod tile;

use player::{PLAYER_COLOR_DATA, Player};
use tile::{TILE_COLOR_DATA, Tile};

use bevy::color::palettes::css::*;
use bevy::prelude::*;

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, populate_cards_and_labels);
        app.add_systems(PostStartup, populate_neighbors);
        app.add_systems(
            Update,
            (
                update_counts_and_playable_tiles,
                update_selects,
                click_selects,
                play_selects,
                update_backs,
                animate_status,
                animate_backs,
                animate_cards,
                animate_selects,
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

#[derive(Component, Clone, Debug)]
pub struct UiCard {
    tile: Tile,
    row: usize,
    column: usize,
}

#[derive(Component)]
pub struct UiBack {
    player: Player,
}

#[derive(Component)]
pub struct UiSelect {
    tile: Tile,
    playable: bool,
}

#[derive(Component)]
pub struct UiStatus;

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

fn populate_cards_and_labels(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut board: ResMut<BoardResource>,
    asset_server: Res<AssetServer>,
) {
    let texture = asset_server.load("textures/border_sheet.png");

    let atlas_layout =
        TextureAtlasLayout::from_grid(UVec2::new(50, 50), 6, 6, Some(UVec2::splat(2)), None);
    let atlas_layout = texture_atlas_layouts.add(atlas_layout);

    let slicer = TextureSlicer {
        border: BorderRect::all(24.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    };

    let make_card = |parent: &mut ChildSpawnerCommands,
                     tile: Tile,
                     row: usize,
                     column: usize|
     -> (Entity, Entity) {
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
            BackgroundColor(Srgba::new(0.0, 1.0, 1.0, 0.2).into()),
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
                        texture.clone(),
                        TextureAtlas {
                            index: atlas_index,
                            layout: atlas_layout.clone(),
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
    };

    let make_select = |parent: &mut ChildSpawnerCommands, tile: Tile| -> Entity {
        let ui_select = UiSelect {
            tile,
            playable: false,
        };

        let tile_index: usize = ui_select.tile.clone().into();
        let (bg_color, fg_color, atlas_index) = TILE_COLOR_DATA[tile_index].clone();
        let bg_color: Color = bg_color.into();
        let fg_color: Color = fg_color.into();

        let mut card_entity = None;
        let mut back = parent.spawn((Node {
            width: Val::Px(70.0),
            height: Val::Px(70.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            margin: UiRect::all(Val::Px(0.0)),
            border: UiRect::all(Val::Px(0.0)),
            padding: UiRect::all(Val::Px(0.0)),
            ..default()
        },));

        back.with_children(|parent| {
            let mut card = parent.spawn((
                ui_select,
                Button,
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
                        texture.clone(),
                        TextureAtlas {
                            index: atlas_index,
                            layout: atlas_layout.clone(),
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

        card_entity.unwrap()
    };

    let make_player_labels =
        |parent: &mut ChildSpawnerCommands, left_player: Player, right_player: Player| {
            let make_label = |container: &mut EntityCommands, player: Player| {
                let index: usize = player.clone().into();
                let (color_bg, color_fg) = PLAYER_COLOR_DATA[index];
                let color_bg: Color = color_bg.into();
                let color_fg: Color = color_fg.into();
                let label = match player.clone() {
                    Player::One => "P1",
                    Player::Two => "P2",
                    Player::Undef => "??",
                };
                container.with_children(|parent| {
                    let mut div = parent.spawn((
                        Node {
                            width: Val::Px(70.0),
                            height: Val::Px(70.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(color_bg),
                    ));
                    div.with_child((TextColor(color_fg.into()), Text::new(label)));
                });
            };

            let make_spacer = |container: &mut EntityCommands| {
                container.with_child((
                    Node {
                        width: Val::Px(70.0 * (BOARD_WIDTH - 2) as f32),
                        height: Val::Px(70.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    // BackgroundColor(PURPLE.into()),
                ));
            };

            let mut container = parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            });

            make_label(&mut container, left_player);
            make_spacer(&mut container);
            make_label(&mut container, right_player);
        };

    let mut body_frame = commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    });

    body_frame.with_children(|parent| {
        let mut seed = BOARD_SEED;
        make_player_labels(parent, Player::One, Player::Undef);
        for row in 0..BOARD_HEIGHT {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::FlexStart,
                    ..default()
                })
                .with_children(|parent| {
                    for column in 0..BOARD_WIDTH {
                        let tile = Tile::from(seed);
                        seed ^= 0x9e3779b9 + (seed << 6) + (seed >> 2);
                        let (card_entity, back_entity) = make_card(parent, tile, row, column);
                        board.card_to_backs.insert(card_entity, back_entity);
                        if row == 0 && column == 0 {
                            board.player_one_card = Some(card_entity);
                        }
                        if row + 1 == BOARD_HEIGHT && column + 1 == BOARD_WIDTH {
                            board.player_two_card = Some(card_entity);
                        }
                    }
                });
        }
        make_player_labels(parent, Player::Undef, Player::Two);
        parent
            .spawn(Node {
                position_type: PositionType::Relative,
                top: Val::Px(-35.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|parent| {
                let select_red_entity = make_select(parent, Tile::Red);
                let select_green_card = make_select(parent, Tile::Green);
                let select_blue_entity = make_select(parent, Tile::Blue);
                board.select_red_card = Some(select_red_entity);
                board.select_green_card = Some(select_green_card);
                board.select_blue_card = Some(select_blue_entity);
            });
    });

    let mut debug_frame = commands.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(5.0),
        bottom: Val::Px(5.0),
        flex_direction: FlexDirection::ColumnReverse,
        align_items: AlignItems::FlexEnd,
        justify_content: JustifyContent::FlexStart,
        ..default()
    });

    debug_frame.with_child((UiStatus, Text::new("status"), TextColor(WHITE.into())));
}

fn populate_neighbors(
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
        let mut neighbors = HashSet::new();
        if let Some(neighbor) = coord_to_cards.get(&(ui_card.row + 1, ui_card.column)) {
            neighbors.insert(*neighbor);
        }
        if let Some(neighbor) = coord_to_cards.get(&(ui_card.row - 1, ui_card.column)) {
            neighbors.insert(*neighbor);
        }
        if let Some(neighbor) = coord_to_cards.get(&(ui_card.row, ui_card.column + 1)) {
            neighbors.insert(*neighbor);
        }
        if let Some(neighbor) = coord_to_cards.get(&(ui_card.row, ui_card.column - 1)) {
            neighbors.insert(*neighbor);
        }
        card_to_neighbors.insert(entity, neighbors);
    }
    board.card_to_neighbors = card_to_neighbors;

    next_state.set(BoardState::WaitingForMove(Player::One));
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

fn animate_status(
    mut status: Single<&mut Text, With<UiStatus>>,
    board: Res<BoardResource>,
    state: Res<State<BoardState>>,
) {
    let state = state.get();
    let label = format!(
        "{:?}\n{:?}\n{:?}",
        board.player_to_counts, board.player_to_playable_tiles, state,
    );
    **status = label.into();
}

fn animate_backs(mut ui_backs: Query<(&UiBack, &mut BackgroundColor)>) {
    for (ui_back, mut back_color) in ui_backs.iter_mut() {
        let player_index: usize = ui_back.player.clone().into();
        let (bg_color, _) = PLAYER_COLOR_DATA[player_index].clone();
        *back_color = bg_color.into();
    }
}

fn animate_cards(
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

fn animate_selects(
    mut ui_selects: Query<
        (
            &UiSelect,
            &Interaction,
            &Children,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        With<Button>,
    >,
    mut buttons: Query<&mut ImageNode>,
    time: Res<Time>,
) {
    let time = time.elapsed().as_secs_f32();
    let strobe = Hsva::new(360.0 * time.fract(), 0.8, 1.0, 1.0);
    for (ui_select, interaction, children, mut border_color, mut back_color) in
        ui_selects.iter_mut()
    {
        let tile_index: usize = ui_select.tile.clone().into();
        let tile_index_: usize = if ui_select.playable { tile_index } else { 0 };
        let (_, _, atlas_index) = TILE_COLOR_DATA[tile_index].clone();
        let (bg_color, fg_color, _) = TILE_COLOR_DATA[tile_index_].clone();
        let bg_color: Color = bg_color.into();
        let fg_color: Color = if ui_select.playable && matches!(interaction, Interaction::Hovered) {
            strobe.into()
        } else {
            fg_color.into()
        };
        *border_color = fg_color.into();
        *back_color = bg_color.into();
        for child in children {
            let mut button = buttons.get_mut(*child).unwrap();
            button.color = fg_color;
            button.texture_atlas.as_mut().unwrap().index = atlas_index;
        }
    }
}
