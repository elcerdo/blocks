use bevy::color::palettes::tailwind::*;
use bevy::prelude::*;

use super::BOARD_BLOCK;
use super::BoardResource;
use super::BoardState;
use super::Player;
use super::player::PLAYER_COLOR_DATA;

pub struct MainBannerPlugin;

impl Plugin for MainBannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, populate);
        app.add_systems(Update, (animate_main, animate_score));
    }
}

#[derive(Component)]
struct MainBannerDiv;

#[derive(Component)]
struct MainBannerText;

#[derive(Component)]
struct ScoreBannerText;

const BANNER_BG_COLOR: Srgba = GRAY_100;
const BANNER_FG_COLOR: Srgba = GRAY_900;

fn populate(mut commands: Commands) {
    let mut frame = commands.spawn((
        MainBannerDiv,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(5.0),
            width: Val::Px(2.0 * BOARD_BLOCK),
            height: Val::Px(3.0 * BOARD_BLOCK / 4.0),
            border: UiRect::all(Val::Px(2.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(BANNER_BG_COLOR.into()),
        BorderColor(BANNER_FG_COLOR.into()),
        BorderRadius::all(Val::Px(8.0)),
        ZIndex(1),
    ));
    frame.with_child((
        MainBannerText,
        TextColor(BANNER_FG_COLOR.into()),
        Text::new("main_banner"),
    ));

    let mut frame = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(2.0 * 5.0 + 3.0 * BOARD_BLOCK / 4.0),
            right: Val::Px(10.0),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexStart,
            ..default()
        },
        ZIndex(1),
    ));
    frame.with_child((
        ScoreBannerText,
        TextColor(BANNER_BG_COLOR.into()),
        Text::new("P1 ??\nP2 ??"),
    ));
}

fn animate_score(
    mut score_text: Single<&mut Text, With<ScoreBannerText>>,
    board: Res<BoardResource>,
) {
    let scores = &board.player_to_counts;
    let mut scores = vec![
        ("Left", scores.get(&Player::Undef).unwrap_or(&0)),
        ("P1", scores.get(&Player::One).unwrap_or(&0)),
        ("P2", scores.get(&Player::Two).unwrap_or(&0)),
    ];
    scores.sort_by(|aa, bb| aa.1.cmp(bb.1).reverse());
    let scores: Vec<String> = scores
        .iter()
        .map(|score| format!("{:>4} {:>2}", score.0, score.1))
        .collect();
    **score_text = scores.join("\n").into();
}

fn animate_main(
    mut main_banner: Single<(&mut BackgroundColor, &mut BorderColor), With<MainBannerDiv>>,
    mut main_text: Single<(&mut Text, &mut TextColor), With<MainBannerText>>,
    state: Res<State<BoardState>>,
    time: Res<Time>,
) {
    let state = state.get();
    let make_label = |player: &Player, suffix: &str| -> String {
        let player = match player {
            Player::Undef => "??",
            Player::One => "P1",
            Player::Two => "P2",
        };
        format!("{} {}", player, suffix)
    };
    let make_win_label = |player: &Player| -> String {
        match player {
            Player::Undef => "Draw",
            Player::One => "P1 wins",
            Player::Two => "P2 wins",
        }
        .into()
    };
    let make_colors = |player: &Player| -> (Color, Color) {
        let index: usize = player.clone().into();
        let (bg_color, fg_color) = PLAYER_COLOR_DATA[index];
        (bg_color.into(), fg_color.into())
    };
    let label = match state {
        BoardState::Init => "Welcome".into(),
        BoardState::WaitingForMove(player) => make_label(player, "turn"),
        BoardState::PlayingMove(player, _) => make_label(player, "turn"),
        BoardState::ResolvingMove(player) => make_label(player, "turn"),
        BoardState::Victory(player) => make_win_label(player),
    };
    let (bg_color, fg_color) = match state {
        BoardState::Init => (BANNER_BG_COLOR.into(), BANNER_FG_COLOR.into()),
        BoardState::WaitingForMove(player) => make_colors(player),
        BoardState::PlayingMove(player, _) => make_colors(player),
        BoardState::ResolvingMove(player) => make_colors(player),
        BoardState::Victory(player) => {
            let time = time.elapsed().as_secs_f32();
            let (bg_color, _) = make_colors(player);
            let strobe = Hsva::new(360.0 * time.fract(), 0.8, 1.0, 1.0);
            (bg_color.into(), strobe.into())
        }
    };
    *main_banner.0 = bg_color.into();
    *main_banner.1 = fg_color.into();
    *main_text.0 = label.into();
    *main_text.1 = fg_color.into();
}
