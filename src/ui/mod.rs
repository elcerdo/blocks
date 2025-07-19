use bevy::prelude::*;

mod colors;
mod combobox;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, populate_ui);
        app.add_systems(
            Update,
            (combobox::update_comboboxes/* , update_ui_01, update_ui_02, animate_ui_00*/).chain(),
        );
    }
}

fn populate_ui(mut commands: Commands, _meshes: ResMut<Assets<Mesh>>) {
    commands.spawn((
        Camera {
            order: 2,
            ..default()
        },
        Camera2d,
    ));

    let mut top_left_frame = commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        margin: UiRect::all(Val::Px(5.0)),
        align_items: AlignItems::Default,
        justify_content: JustifyContent::Default,
        flex_direction: FlexDirection::Column,
        ..default()
    });

    combobox::make_combobox(&mut top_left_frame, vec!["aa", "bb", "cc"]);
    combobox::make_combobox(&mut top_left_frame, vec!["x", "yy", "zzz", "wwww"]);
}
