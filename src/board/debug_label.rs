use bevy::prelude::*;

use super::BoardResource;
use super::BoardState;

#[derive(Component)]
pub(crate) struct UiDebugLabel;

pub fn populate(mut commands: Commands) {
    let mut debug_frame = commands.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(5.0),
        bottom: Val::Px(5.0),
        flex_direction: FlexDirection::ColumnReverse,
        align_items: AlignItems::FlexEnd,
        justify_content: JustifyContent::FlexStart,
        ..default()
    });
    debug_frame.with_child((UiDebugLabel, Text::new("status")));
}

pub fn animate(
    mut status: Single<&mut Text, With<UiDebugLabel>>,
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
