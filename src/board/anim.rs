use bevy::prelude::*;

use super::UiBack;
use super::UiCard;
use super::player::PLAYER_COLOR_DATA;
use super::tile::TILE_COLOR_DATA;

pub fn animate_backs(mut ui_backs: Query<(&UiBack, &mut BackgroundColor)>) {
    for (ui_back, mut back_color) in ui_backs.iter_mut() {
        let player_index: usize = ui_back.player.clone().into();
        let (bg_color, _) = PLAYER_COLOR_DATA[player_index].clone();
        *back_color = bg_color.into();
    }
}

pub fn animate_cards(
    mut ui_cards: Query<(&UiCard, &Children, &mut BorderColor, &mut BackgroundColor), With<Button>>,
    mut buttons: Query<&mut ImageNode>,
) {
    for (ui_card, children, mut border_color, mut back_color) in ui_cards.iter_mut() {
        let tile_index: usize = ui_card.tile.clone().into();
        let (bg_color, fg_color, atlas_index) = TILE_COLOR_DATA[tile_index].clone();
        let bg_color: Color = bg_color.into();
        let fg_color: Color = fg_color.into();
        *border_color = fg_color.into();
        *back_color = bg_color.into();
        for child in children {
            let mut button = buttons.get_mut(*child).unwrap();
            button.color = fg_color;
            button.texture_atlas.as_mut().unwrap().index = atlas_index;
        }
    }
}

