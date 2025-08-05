use bevy::prelude::*;

use super::BoardResource;
use super::Player;
use super::Tile;

use super::card_and_back;
use super::player_block;
use super::select_move;

use super::BOARD_HEIGHT;
use super::BOARD_SEED;
use super::BOARD_WIDTH;

pub fn populate_board(
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
        player_block::make_pair(parent, Player::One, Player::Undef);
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
                        let (card_entity, back_entity) = card_and_back::make_pair(
                            &texture,
                            &atlas_layout,
                            &slicer,
                            parent,
                            tile,
                            row,
                            column,
                        );
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
        player_block::make_pair(parent, Player::Undef, Player::Two);
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
                    select_move::make(&texture, &atlas_layout, &slicer, parent, Tile::Red);
                let select_green_card =
                    select_move::make(&texture, &atlas_layout, &slicer, parent, Tile::Green);
                let select_blue_entity =
                    select_move::make(&texture, &atlas_layout, &slicer, parent, Tile::Blue);
                board.select_red_card = Some(select_red_entity);
                board.select_green_card = Some(select_green_card);
                board.select_blue_card = Some(select_blue_entity);
            });
    });
}
