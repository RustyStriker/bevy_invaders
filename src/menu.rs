use crate::AppState;
use bevy::prelude::*;

pub struct Menu;

impl Plugin for Menu {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(SystemSet::on_enter(AppState::Menu).with_system(setup_menu.system()))
            .add_system_set(SystemSet::on_update(AppState::Menu).with_system(start_game.system()))
            .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(clear_menu.system()));
    }
}

struct MenuItem;

fn setup_menu(mut coms: Commands, asset_server: Res<AssetServer>) {
    let text_style = TextStyle {
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        font_size: 32.0,
        color: Color::WHITE,
    };

    coms.spawn_bundle(TextBundle {
        text: Text::with_section(
            "Press `Space` to start",
            text_style,
            TextAlignment::default(),
        ),
        ..Default::default()
    })
    .insert(MenuItem);
}

fn clear_menu(mut coms: Commands, items: Query<Entity, With<MenuItem>>) {
    for e in items.iter() {
        coms.entity(e).despawn();
    }
}

fn start_game(mut state: ResMut<State<AppState>>, keyboard: Res<Input<KeyCode>>) {
    if keyboard.just_released(KeyCode::Space) {
        println!("Game starting *bip*...*bip*...*bip*...");
        state.set(AppState::Game).unwrap();
    }
}
