use bevy::prelude::*;

use super::BoardState;
use super::Player;

use super::BOARD_WIDTH;
use super::player::PLAYER_COLOR_DATA;

#[derive(Component)]
pub struct UiPlayerBlock {
    player: Player,
}

#[derive(Component)]
pub struct UiCrownBlock {
    player: Player,
}

pub fn make_pair(
    texture_crown: &Handle<Image>,
    atlas_layout_crown: &Handle<TextureAtlasLayout>,
    parent: &mut ChildSpawnerCommands,
    left_player: Player,
    right_player: Player,
) {
    let block_node = Node {
        width: Val::Px(70.0),
        height: Val::Px(70.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    };

    let make_crown = |container: &mut EntityCommands, player: Player| {
        let ui_crown_block = UiCrownBlock { player };
        container.with_children(|parent| {
            parent.spawn((
                block_node.clone(),
                ui_crown_block,
                ImageNode::from_atlas_image(
                    texture_crown.clone(),
                    TextureAtlas {
                        index: 0,
                        layout: atlas_layout_crown.clone(),
                    },
                )
                .with_color(Srgba::new(1.0, 1.0, 1.0, 0.5).into()),
            ));
        });
    };

    let make_label = |container: &mut EntityCommands, player: Player| {
        let ui_player_label = UiPlayerBlock { player };
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
            parent
                .spawn((block_node.clone(), BackgroundColor(color_bg)))
                .with_child((
                    ui_player_label,
                    TextColor(color_fg.into()),
                    Text::new(label),
                ));
        });
    };

    let make_spacer = |container: &mut EntityCommands| {
        container.with_child(Node {
            width: Val::Px(70.0 * (BOARD_WIDTH - 4) as f32),
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

    make_label(&mut container, left_player.clone());
    make_crown(&mut container, left_player);
    make_spacer(&mut container);
    make_crown(&mut container, right_player.clone());
    make_label(&mut container, right_player);
}

pub fn animate_labels(
    mut ui_labels: Query<(&UiPlayerBlock, &mut TextColor)>,
    time: Res<Time>,
    state: Res<State<BoardState>>,
) {
    let time = time.elapsed().as_secs_f32();
    let strobe = Hsva::new(360.0 * time.fract(), 0.8, 1.0, 1.0);
    for (ui_player_label, mut text_color) in ui_labels.iter_mut() {
        let player_index: usize = ui_player_label.player.clone().into();
        let (_, fg_color) = PLAYER_COLOR_DATA[player_index].clone();
        let fg_color: Color = match state.get() {
            BoardState::WaitingForMove(player) => {
                if *player == ui_player_label.player {
                    strobe.into()
                } else {
                    fg_color.into()
                }
            }
            _ => fg_color.into(),
        };
        *text_color = fg_color.into();
    }
}

pub fn animate_crowns(
    mut ui_crowns: Query<(&UiCrownBlock, &mut ImageNode)>,
    state: Res<State<BoardState>>,
) {
    let state = state.get();
    for (ui_crown, mut image_node) in ui_crowns.iter_mut() {
        let is_winning = match state {
            BoardState::Victory(player) => *player == ui_crown.player,
            _ => false,
        };
        let alpha = if is_winning { 1.0 } else { 0.0 };
        let color = Srgba::new(1.0, 1.0, 1.0, alpha);
        image_node.color = color.into();
    }
}
