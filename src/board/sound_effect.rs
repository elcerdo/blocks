use bevy::prelude::*;

use log::info;

#[derive(Resource)]
pub struct SoundEffectResource {
    ding: AudioPlayer,
    yeah: AudioPlayer,
    ambiance_aa: AudioPlayer,
    ambiance_bb: AudioPlayer,
}

pub fn populate(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ding = asset_server.load("sounds/ding-notification-sound-383728.ogg");
    let yeah = asset_server.load("sounds/yeah-7106.ogg");
    let ambiance_aa = asset_server.load("sounds/forest-ambiance-30746.ogg");
    let ambiance_bb = asset_server.load("sounds/morning-forest-ambiance-17045.ogg");
    let sfx = SoundEffectResource {
        ding: AudioPlayer::new(ding),
        yeah: AudioPlayer::new(yeah),
        ambiance_aa: AudioPlayer::new(ambiance_aa),
        ambiance_bb: AudioPlayer::new(ambiance_bb),
    };
    commands.insert_resource(sfx);
}

pub fn play_ding(mut commands: Commands, sfx: Res<SoundEffectResource>) {
    info!("!!! collision !!!!");
    commands.spawn((sfx.ding.clone(), PlaybackSettings::ONCE));
}

pub fn play_yeah(mut commands: Commands, sfx: Res<SoundEffectResource>) {
    info!("!!! yeah !!!!");
    commands.spawn((sfx.yeah.clone(), PlaybackSettings::ONCE));
}
