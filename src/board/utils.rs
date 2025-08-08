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
    let texture_border = asset_server.load("textures/border_sheet.png");
    let atlas_layout_border =
        TextureAtlasLayout::from_grid(UVec2::new(50, 50), 6, 6, Some(UVec2::splat(2)), None);
    let atlas_layout_border = texture_atlas_layouts.add(atlas_layout_border);

    let texture_crown = asset_server.load("textures/crown_70x70.png");
    let atlas_layout_crown = TextureAtlasLayout::from_grid(UVec2::new(70, 70), 1, 1, None, None);
    let atlas_layout_crown = texture_atlas_layouts.add(atlas_layout_crown);

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
        player_block::make_pair(
            &texture_crown,
            &atlas_layout_crown,
            parent,
            Player::One,
            Player::Undef,
            true,
        );
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
                            &texture_border,
                            &atlas_layout_border,
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
        player_block::make_pair(
            &texture_crown,
            &atlas_layout_crown,
            parent,
            Player::Undef,
            Player::Two,
            false,
        );
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
                board.select_cards.clear();
                for tile in [Tile::Red, Tile::Green, Tile::Blue, Tile::Yellow] {
                    board.select_cards.push(select_move::make(
                        &texture_border,
                        &atlas_layout_border,
                        &slicer,
                        parent,
                        tile,
                    ));
                }
            });
    });
}
