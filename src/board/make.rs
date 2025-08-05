use bevy::prelude::*;

use super::BOARD_HEIGHT;
use super::BOARD_SEED;
use super::BOARD_WIDTH;
use super::BoardResource;
use super::BoardState;
use super::UiBack;
use super::UiCard;
use super::UiPlayerLabel;
use super::UiSelect;
use super::player::{PLAYER_COLOR_DATA, Player};
use super::tile::{TILE_COLOR_DATA, Tile};

use std::collections::HashMap;
use std::collections::HashSet;

fn make_select(
    texture: &Handle<Image>,
    atlas_layout: &Handle<TextureAtlasLayout>,
    slicer: &TextureSlicer,
    parent: &mut ChildSpawnerCommands,
    tile: Tile,
) -> Entity {
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
}

fn make_card(
    texture: &Handle<Image>,
    atlas_layout: &Handle<TextureAtlasLayout>,
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
}

fn make_player_labels(
    parent: &mut ChildSpawnerCommands,
    left_player: Player,
    right_player: Player,
) {
    let make_label = |container: &mut EntityCommands, player: Player| {
        let ui_player_label = UiPlayerLabel { player };
        let index: usize = ui_player_label.player.clone().into();
        let (color_bg, color_fg) = PLAYER_COLOR_DATA[index];
        let color_bg: Color = color_bg.into();
        let color_fg: Color = color_fg.into();
        let label = match ui_player_label.player.clone() {
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
            div.with_child((
                ui_player_label,
                TextColor(color_fg.into()),
                Text::new(label),
            ));
        });
    };

    let make_spacer = |container: &mut EntityCommands| {
        container.with_child(Node {
            width: Val::Px(70.0 * (BOARD_WIDTH - 2) as f32),
            height: Val::Px(70.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        });
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
}

pub fn populate_items(
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
                        let (card_entity, back_entity) =
                            make_card(&texture, &atlas_layout, &slicer, parent, tile, row, column);
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
                let select_red_entity =
                    make_select(&texture, &atlas_layout, &slicer, parent, Tile::Red);
                let select_green_card =
                    make_select(&texture, &atlas_layout, &slicer, parent, Tile::Green);
                let select_blue_entity =
                    make_select(&texture, &atlas_layout, &slicer, parent, Tile::Blue);
                board.select_red_card = Some(select_red_entity);
                board.select_green_card = Some(select_green_card);
                board.select_blue_card = Some(select_blue_entity);
            });
    });
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
