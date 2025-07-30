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

#[derive(Resource, Default)]
struct Game {
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
enum GameState {
    #[default]
    Init,
    WaitingForMove(Player),
    SelectedMove(Player),
}

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, populate_cards);
        app.add_systems(PostStartup, populate_neighbors);
        app.add_systems(
            Update,
            (
                click_cards,
                update_backs,
                update_game_stats,
                animate_status,
                animate_backs,
                animate_cards,
            )
                .chain(),
        );
        app.init_resource::<Game>();
        app.init_state::<GameState>();
    }
}

const BOARD_WIDTH: usize = 12;
const BOARD_HEIGHT: usize = 8;

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

#[derive(Clone, Eq, PartialEq)]
struct Priority {
    distance: usize,
    player: Player,
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
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

fn populate_cards(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut game: ResMut<Game>,
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

    commands.spawn(Camera2d);

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

    let mut body_frame = commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    });

    body_frame.with_children(|parent| {
        let mut seed: usize = 0xabff1aab45;
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
                        game.card_to_backs.insert(card_entity, back_entity);
                        if row == 0 && column == 0 {
                            game.player_one_card = Some(card_entity);
                        }
                        if row + 1 == BOARD_HEIGHT && column + 1 == BOARD_WIDTH {
                            game.player_two_card = Some(card_entity);
                        }
                    }
                });
        }
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::top(Val::Px(10.0)),
                ..default()
            })
            .with_children(|parent| {
                let select_red_entity = make_select(parent, Tile::Red);
                let select_green_card = make_select(parent, Tile::Green);
                let select_blue_entity = make_select(parent, Tile::Blue);
                game.select_red_card = Some(select_red_entity);
                game.select_green_card = Some(select_green_card);
                game.select_blue_card = Some(select_blue_entity);
            });
        parent.spawn((
            UiStatus,
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Text::new("status"),
            TextColor(WHITE.into()),
        ));
    });
}

fn populate_neighbors(
    ui_cards: Query<(&UiCard, Entity)>,
    mut game: ResMut<Game>,
    mut state: ResMut<NextState<GameState>>,
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
    game.card_to_neighbors = card_to_neighbors;

    state.set(GameState::WaitingForMove(Player::One));
}

fn update_backs(mut ui_backs: Query<&mut UiBack>, ui_cards: Query<&UiCard>, game: Res<Game>) {
    assert!(game.player_one_card.is_some());
    assert!(game.player_two_card.is_some());
    assert!(!game.card_to_neighbors.is_empty());
    assert!(!game.card_to_backs.is_empty());
    assert!(game.card_to_backs.len() == game.card_to_neighbors.len());

    for mut ui_back in ui_backs.iter_mut() {
        ui_back.player = Player::Undef;
    }

    let player_one_card = game.player_one_card.unwrap();
    let player_two_card = game.player_two_card.unwrap();

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
        let next_cards = game.card_to_neighbors.get(&current_card).unwrap();
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

        let current_back = game.card_to_backs.get(&current_card).unwrap();
        let mut current_back = ui_backs.get_mut(*current_back).unwrap();
        current_back.player = current_priority.player.clone();

        done.insert(current_card);
    }
}

fn update_game_stats(
    ui_backs: Query<&UiBack>,
    ui_cards: Query<(&UiCard, Entity)>,
    mut game: ResMut<Game>,
) {
    let mut player_to_counts = BTreeMap::new();
    for ui_back in ui_backs.iter() {
        if !player_to_counts.contains_key(&ui_back.player) {
            player_to_counts.insert(ui_back.player.clone(), 0);
        }
        *player_to_counts.get_mut(&ui_back.player).unwrap() += 1;
    }
    game.player_to_counts = player_to_counts;

    let mut player_to_playable_tiles = BTreeMap::new();
    player_to_playable_tiles.insert(Player::One, BTreeSet::new());
    player_to_playable_tiles.insert(Player::Two, BTreeSet::new());
    for (ui_card, card_entity) in ui_cards.iter() {
        let back_entity = *game.card_to_backs.get(&card_entity).unwrap();
        let ui_back = ui_backs.get(back_entity).unwrap();
        if let Some(playable_tiles) = player_to_playable_tiles.get_mut(&ui_back.player) {
            assert!(ui_card.tile != Tile::Undef);
            for card_entity_ in game.card_to_neighbors.get(&card_entity).unwrap() {
                let card_entity_ = *card_entity_;
                let back_entity_ = *game.card_to_backs.get(&card_entity_).unwrap();
                let ui_card_ = ui_cards.get(card_entity_).unwrap().0;
                let ui_back_ = ui_backs.get(back_entity_).unwrap();
                if ui_back_.player != ui_back.player && ui_card_.tile != Tile::Undef {
                    assert!(ui_card.tile != ui_card_.tile);
                    playable_tiles.insert(ui_card_.tile.clone());
                }
            }
        }
    }
    game.player_to_playable_tiles = player_to_playable_tiles;
}

fn animate_status(
    mut status: Single<&mut Text, With<UiStatus>>,
    game: Res<Game>,
    state: Res<State<GameState>>,
) {
    let state = state.get();
    let label = format!(
        "{:?} // {:?} // {:?}",
        game.player_to_counts, game.player_to_playable_tiles, state,
    );
    **status = label.into();
}

fn animate_backs(mut ui_backs: Query<(&UiBack, &mut BackgroundColor)>) {
    for (ui_back, mut back_color) in ui_backs.iter_mut() {
        let player_index: usize = ui_back.player.clone().into();
        let bg_color = PLAYER_COLOR_DATA[player_index].clone();
        *back_color = bg_color.into();
    }
}

fn animate_cards(
    mut ui_cards: Query<
        (
            &UiCard,
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
    for (ui_card, interaction, children, mut border_color, mut back_color) in ui_cards.iter_mut() {
        let tile_index: usize = ui_card.tile.clone().into();
        let (bg_color, fg_color, atlas_index) = TILE_COLOR_DATA[tile_index].clone();
        let bg_color: Color = bg_color.into();
        let fg_color: Color = fg_color.into();
        let color: Color = match interaction {
            Interaction::Hovered => strobe.into(),
            _ => fg_color,
        };
        *border_color = color.into();
        *back_color = bg_color.into();
        for child in children {
            let mut button = buttons.get_mut(*child).unwrap();
            button.color = color;
            button.texture_atlas.as_mut().unwrap().index = atlas_index;
        }
    }
}

fn click_cards() {
    // mut ui_cards: Query<(&mut UiCard, &Interaction), (Changed<Interaction>, With<Button>)>,
    /*
    for (mut ui_card, interaction) in ui_cards.iter_mut() {
        if matches!(*interaction, Interaction::Pressed) {
            ui_card.tile = match ui_card.tile {
                Tile::Undef => Tile::Undef,
                Tile::Red => Tile::Green,
                Tile::Green => Tile::Blue,
                Tile::Blue => Tile::Red,
            };
        }
    }
    */
}
