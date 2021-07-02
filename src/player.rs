use bevy::prelude::*;

use crate::{
    bullets::{Bullet, SpawnBulletEvent},
    collisions::{Aabb, Collision},
    AppState,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Game).with_system(spawn_player_startup.system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(player_input.system())
                .with_system(kill_player.system()),
        )
        .add_system_set(SystemSet::on_exit(AppState::Game).with_system(despawn_player.system()));
    }
}

fn spawn_player_startup(mut coms: Commands, mut colors: ResMut<Assets<ColorMaterial>>) {
    let player_color = colors.add(Color::ALICE_BLUE.into());
    let bullet = colors.add(Color::YELLOW.into());

    coms.spawn_bundle(SpriteBundle {
        sprite: Sprite::new(Vec2::splat(20.0)),
        material: player_color,
        transform: Transform::from_xyz(0.0, -300.0, 0.0),
        ..Default::default()
    })
    .insert(Player {
        move_speed: 120.0,
        min_pos: Vec2::splat(-0.4),
        max_pos: Vec2::new(0.4, -0.4),
        shoot_timer: 0.0,
        between_shots: 0.5,
        nuzzle_displacement: Vec2::new(0.0, 30.0),
        bullet_speed: Vec2::new(0.0, 140.0),
        bullet_color: bullet,
    })
    .insert(Aabb::new(Vec2::splat(10.0).extend(0.0)));
}

fn despawn_player(mut coms: Commands, players: Query<Entity, With<Player>>) {
    for e in players.iter() {
        coms.entity(e).despawn();
    }
}

pub struct Player {
    pub move_speed: f32,
    /// Relative to screen size
    pub min_pos: Vec2,
    pub max_pos: Vec2,

    shoot_timer: f32,
    pub between_shots: f32,

    pub nuzzle_displacement: Vec2,
    pub bullet_speed: Vec2,

    pub bullet_color: Handle<ColorMaterial>,
}

fn player_input(
    time: Res<Time>,
    wins: Res<Windows>,
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Player)>,
    mut bullet_writer: EventWriter<SpawnBulletEvent>,
) {
    // get window size
    let main_win = wins.get_primary().unwrap();
    let win_size = Vec2::new(main_win.width(), main_win.height());

    // Gather the input first
    let mut dir_vec = Vec2::ZERO;

    if input.pressed(KeyCode::A) {
        // Go left
        dir_vec.x -= 1.0;
    }
    if input.pressed(KeyCode::D) {
        dir_vec.x += 1.0;
    }

    let shoot = input.pressed(KeyCode::Space);

    let movement_vec = time.delta_seconds() * dir_vec.extend(0.0);

    for (mut t, mut p) in query.iter_mut() {
        t.translation += movement_vec * p.move_speed;

        let t2: Vec2 = t.translation.into();

        let t2 = t2.min(p.max_pos * win_size).max(p.min_pos * win_size);

        t.translation = t2.extend(t.translation.z);

        if p.shoot_timer > 0.0 {
            p.shoot_timer -= time.delta_seconds();
        } else if shoot {
            // reset the timer
            p.shoot_timer = p.between_shots;

            let bull_pos = t2 + p.nuzzle_displacement;

            let event = SpawnBulletEvent {
                bullet: Bullet {
                    speed: p.bullet_speed,
                },
                position: bull_pos,
                color: p.bullet_color.clone(),
                size: Vec2::splat(10.0),
            };
            bullet_writer.send(event);
        }
    }
}

fn kill_player(
    mut colls: EventReader<Collision>,
    mut state: ResMut<State<AppState>>,
    player: Query<Entity, With<Player>>,
) {
    for c in colls.iter() {
        if player.get(c.0).is_ok() || player.get(c.1).is_ok() {
            println!("Game over... again...");
            state.set(AppState::Menu).unwrap();
        }
    }
}
