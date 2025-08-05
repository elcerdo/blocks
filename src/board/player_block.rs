use bevy::prelude::*;

use super::BOARD_WIDTH;
use super::BoardState;
use super::player::{PLAYER_COLOR_DATA, Player};

#[derive(Component)]
pub(crate) struct UiPlayerBlock {
    player: Player,
}

pub fn make_pair(parent: &mut ChildSpawnerCommands, left_player: Player, right_player: Player) {
    let make_label = |container: &mut EntityCommands, player: Player| -> Entity {
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
        let mut entity: Option<Entity> = None;
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
            entity = div.id().into();
        });
        assert!(entity.is_some());
        entity.unwrap()
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

pub fn animate(
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
