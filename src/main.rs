mod bullets;
mod collisions;
mod enemies;
mod menu;
mod player;

use bevy::prelude::*;
use bullets::BulletsPlugin;
use collisions::Physics;
use enemies::Enemies;
use menu::Menu;
use player::PlayerPlugin;

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
enum AppState {
    Menu,
    Game,
}

fn main() {
    App::build()
        .add_state(AppState::Menu)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugin(Physics)
        .add_plugin(Enemies)
        .add_plugin(PlayerPlugin)
        .add_plugin(BulletsPlugin)
        .add_plugin(Menu)
        .add_startup_system(cam_setup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}

fn cam_setup(mut coms: Commands) {
    // This can be done in the plugins themselves but im gonna do it here for simplicity sake
    coms.spawn_bundle(OrthographicCameraBundle::new_2d());
    coms.spawn_bundle(UiCameraBundle::default());
}
