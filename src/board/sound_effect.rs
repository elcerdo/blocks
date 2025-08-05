use bevy::prelude::*;

use log::info;

#[derive(Resource, Default)]
pub struct SoundEffectResource {
    collision: Option<AudioPlayer>,
    yeah: Option<AudioPlayer>,
}

pub fn populate(asset_server: Res<AssetServer>, mut sfx: ResMut<SoundEffectResource>) {
    let collision = asset_server.load("sounds/breakout_collision.ogg");
    let yeah = asset_server.load("sounds/yeah-7106.ogg");
    sfx.collision = Some(AudioPlayer::new(collision));
    sfx.yeah = Some(AudioPlayer::new(yeah));
}

pub fn play_collision(mut commands: Commands, sfx: Res<SoundEffectResource>) {
    info!("!!! collision !!!!");
    let player = sfx.collision.clone();
    assert!(player.is_some());
    commands.spawn((player.unwrap(), PlaybackSettings::ONCE));
}

pub fn play_yeah(mut commands: Commands, sfx: Res<SoundEffectResource>) {
    info!("!!! yeah !!!!");
    let player = sfx.yeah.clone();
    assert!(player.is_some());
    commands.spawn((player.unwrap(), PlaybackSettings::ONCE));
}
