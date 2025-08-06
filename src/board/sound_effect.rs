use bevy::audio::Volume;
use bevy::prelude::*;

use super::BoardState;
use super::Player;
use super::select_move::UiSelectMove;

use log::info;

pub struct SoundEffectPlugin;

impl Plugin for SoundEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, populate);
        app.add_systems(OnEnter(BoardState::Victory(Player::One)), play_yeah);
        app.add_systems(OnEnter(BoardState::Victory(Player::Two)), play_yeah);
        app.add_systems(OnEnter(BoardState::ResolvingMove(Player::One)), play_ding);
        app.add_systems(OnEnter(BoardState::ResolvingMove(Player::Two)), play_ding);
        app.add_systems(Update, mix_ambience);
    }
}

#[derive(Resource)]
pub struct SoundEffectResource {
    ding: AudioPlayer,
    yeah: AudioPlayer,
    ambiance_mix: f32,
}

#[derive(Component)]
struct SoundAmbianceAA;

#[derive(Component)]
struct SoundAmbianceBB;

fn populate(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ding = asset_server.load("sounds/ding-notification-sound-383728.ogg");
    let yeah = asset_server.load("sounds/yeah-7106.ogg");
    let ambiance_aa = asset_server.load("sounds/forest-ambiance-30746.ogg");
    let ambiance_bb = asset_server.load("sounds/morning-forest-ambiance-17045.ogg");
    let sfx = SoundEffectResource {
        ding: AudioPlayer::new(ding),
        yeah: AudioPlayer::new(yeah),
        ambiance_mix: 0.0,
    };
    commands.insert_resource(sfx);
    commands.spawn((
        AudioPlayer::new(ambiance_aa),
        PlaybackSettings::LOOP,
        SoundAmbianceAA,
    ));
    commands.spawn((
        AudioPlayer::new(ambiance_bb),
        PlaybackSettings::LOOP,
        SoundAmbianceBB,
    ));
}

fn play_ding(mut commands: Commands, sfx: Res<SoundEffectResource>) {
    info!("!!! ding !!!!");
    commands.spawn((sfx.ding.clone(), PlaybackSettings::ONCE));
}

fn play_yeah(mut commands: Commands, sfx: Res<SoundEffectResource>) {
    info!("!!! yeah !!!!");
    commands.spawn((sfx.yeah.clone(), PlaybackSettings::ONCE));
}

fn mix_ambience(
    ui_selects: Query<(&UiSelectMove, &Interaction), With<Button>>,
    mut sfx: ResMut<SoundEffectResource>,
    mut ambiance_aa: Single<&mut AudioSink, (With<SoundAmbianceAA>, Without<SoundAmbianceBB>)>,
    mut ambiance_bb: Single<&mut AudioSink, (Without<SoundAmbianceAA>, With<SoundAmbianceBB>)>,
) {
    let target_ambiance_mix = {
        let mut alpha = 0.0;
        for (ui_select, interaction) in ui_selects.iter() {
            if ui_select.is_playable && !matches!(interaction, Interaction::None) {
                alpha = 1.0;
                break;
            }
        }
        alpha
    };

    let current_ambiance_mix = {
        let alpha = 0.98 * sfx.ambiance_mix + 0.02 * target_ambiance_mix;
        let alpha = alpha.clamp(0.0, 1.0);
        alpha
    };

    ambiance_aa.set_volume(Volume::Decibels(6.0) * Volume::Linear(1.0 - current_ambiance_mix));
    ambiance_bb.set_volume(Volume::Decibels(0.0) * Volume::Linear(current_ambiance_mix));

    sfx.ambiance_mix = current_ambiance_mix;
}
