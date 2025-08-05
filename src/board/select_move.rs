use bevy::prelude::*;

use super::BoardResource;
use super::BoardState;
use super::Tile;

use super::tile::TILE_COLOR_DATA;

use std::collections::BTreeSet;

#[derive(Component)]
pub struct UiSelectMove {
    tile: Tile,
    playable: bool,
}

pub fn make(
    texture: &Handle<Image>,
    atlas_layout: &Handle<TextureAtlasLayout>,
    slicer: &TextureSlicer,
    parent: &mut ChildSpawnerCommands,
    tile: Tile,
) -> Entity {
    let ui_select = UiSelectMove {
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

pub fn update(
    mut ui_selects: Query<&mut UiSelectMove>,
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

pub fn click(
    ui_selects: Query<(&UiSelectMove, &Interaction), (Changed<Interaction>, With<Button>)>,
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

pub fn animate(
    mut ui_selects: Query<
        (
            &UiSelectMove,
            &Interaction,
            &Children,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        With<Button>,
    >,
    mut image_nodes: Query<&mut ImageNode>,
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
            let mut image = image_nodes.get_mut(*child).unwrap();
            image.color = fg_color;
            image.texture_atlas.as_mut().unwrap().index = atlas_index;
        }
    }
}
