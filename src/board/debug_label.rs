use bevy::prelude::*;

use super::BoardResource;
use super::BoardState;

pub struct DebugLabelPlugin;

impl Plugin for DebugLabelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, populate);
        app.add_systems(Update, animate);
    }
}

#[derive(Component)]
struct DebugLabel;

fn populate(mut commands: Commands) {
    let mut debug_frame = commands.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(5.0),
        bottom: Val::Px(5.0),
        flex_direction: FlexDirection::ColumnReverse,
        align_items: AlignItems::FlexEnd,
        justify_content: JustifyContent::FlexStart,
        ..default()
    });
    debug_frame.with_child((DebugLabel, Text::new("debug_label")));
}

fn animate(
    mut debug_label: Single<&mut Text, With<DebugLabel>>,
    board: Res<BoardResource>,
    state: Res<State<BoardState>>,
) {
    let state = state.get();
    let label = format!(
        "{} moves\n{:?}\n{:?}\n{:?}",
        board.num_resolved_moves, board.player_to_counts, board.player_to_playable_tiles, state,
    );
    **debug_label = label.into();
}
