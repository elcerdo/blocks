//! board game

mod board;
mod ui;

use bevy::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(board::BoardPlugin);
    app.add_plugins(ui::UiPlugin);

    #[cfg(not(target_family = "wasm"))]
    {
        app.add_systems(Update, keyboard_shortcuts);
    }

    app.add_systems(Update, play_sfx);

    app.run();
}

#[cfg(not(target_family = "wasm"))]
fn keyboard_shortcuts(mut writer: EventWriter<AppExit>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        writer.write(AppExit::Success);
    }
    if keyboard.just_pressed(KeyCode::Space) {
        warn!("reseed");
    }
}

fn play_sfx(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
) {
    let audio = asset_server.load("sounds/yeah-7106.ogg");
    if keyboard.just_pressed(KeyCode::KeyC) {
        commands.spawn((AudioPlayer::new(audio), PlaybackSettings::ONCE));
    }
}
