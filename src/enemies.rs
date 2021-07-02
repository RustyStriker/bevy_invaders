use crate::{collisions::*, AppState};
use bevy::{core::FixedTimestep, prelude::*};

pub struct Enemies;

impl Plugin for Enemies {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<RoundOver>();

        app.add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(kill_on_collision.system())
                .with_system(round_over.system())
                .with_system(spawn_enemies_on_event.system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_run_criteria(FixedTimestep::step(0.2))
                .with_system(enemy_movement.system()),
        )
        .add_system_set(SystemSet::on_exit(AppState::Game).with_system(clear_enemies.system()))
        .add_startup_system(enemies_startup.system());
    }
}

/// Doing it this way because i dont know how to get the `Res<Assets<ColorMaterial>>` from the `AppBuilder`
fn enemies_startup(mut coms: Commands, mut colors: ResMut<Assets<ColorMaterial>>) {
    let enemy_color = colors.add(Color::RED.into());
    coms.insert_resource(DefaultEnemyColor(enemy_color));
    coms.insert_resource(SpawnEnemies::default());
}

pub struct SpawnEnemies {
    pub rows: i32,
    pub cols: i32,
    pub center: Vec2,
    pub round: i32,
}
impl Default for SpawnEnemies {
    fn default() -> Self {
        SpawnEnemies {
            rows: 1,
            cols: 4,
            center: Vec2::new(0.0, 100.0),
            round: 1,
        }
    }
}

struct DefaultEnemyColor(Handle<ColorMaterial>);

fn spawn_enemies_on_event(
    mut coms: Commands,
    mut round_over: EventReader<RoundOver>,
    enemy_color: Res<DefaultEnemyColor>,
    mut spawn: ResMut<SpawnEnemies>,
) {
    let enemy_size = Vec2::new(20.0, 20.0);
    let spacing = Vec2::new(20.0, 20.0);

    for _ in round_over.iter() {
        println!("Round {}!", spawn.round);

        let width = enemy_size.x * spawn.cols as f32 + spacing.x * (spawn.cols - 1) as f32;
        let height = enemy_size.y * spawn.rows as f32 + spacing.y * (spawn.rows - 1) as f32;

        let start = spawn.center - Vec2::new(width, height) * 0.5;

        // #TheTrueRustWay
        (0..spawn.rows).for_each(|i| {
            let i = i as f32;

            (0..spawn.cols).for_each(|k| {
                let k = k as f32;

                let position = start + Vec2::new(k, i) * (enemy_size + spacing);

                // Spawn the enemy
                coms.spawn_bundle(SpriteBundle {
                    sprite: Sprite::new(enemy_size),
                    material: enemy_color.0.clone(),
                    transform: Transform::from_translation(position.extend(0.0)),
                    ..Default::default()
                })
                .insert(Enemy)
                .insert(Aabb::new(enemy_size.extend(0.0) * 0.5));
            });
        });
        // Make the next wave harder
        spawn.cols += 1;
        spawn.rows += 1;
        spawn.round += 1;
    }
}

struct EnemyControl {
    dir: f32,
    step: f32,
    y_step: f32,
    just_switched: bool,
}
impl Default for EnemyControl {
    fn default() -> Self {
        Self {
            dir: 1.0,
            step: 10.0,
            y_step: 20.0,
            just_switched: false,
        }
    }
}
pub struct Enemy;

/// Kills `Enemy` entities on collision events
fn kill_on_collision(
    mut coms: Commands,
    enemies: Query<Entity, With<Enemy>>,
    mut colls: EventReader<Collision>,
) {
    for c in colls.iter() {
        if let Ok(e) = enemies.get(c.0) {
            coms.entity(e).despawn();
        } else if let Ok(e) = enemies.get(c.1) {
            coms.entity(e).despawn();
        }
    }
}

pub struct RoundOver;

fn round_over(
    mut writer: EventWriter<RoundOver>,
    enemies: Query<Entity, With<Enemy>>,
    mut round_started: Local<bool>,
) {
    let has_enemies = enemies.iter().next().is_some();

    if !*round_started && !has_enemies {
        *round_started = true;
        writer.send(RoundOver);
    } else if *round_started && has_enemies {
        *round_started = false;
    }
}

fn enemy_movement(
    mut enemies: Query<&mut Transform, With<Enemy>>,
    mut control: Local<EnemyControl>,
    wins: Res<Windows>,
    mut state: ResMut<State<AppState>>,
) {
    // We need this for the window size
    let win = wins.get_primary().expect("No window? wtf");

    // If we just switched we should go 1 time downwards
    if !control.just_switched {
        let mut switch_dir = false;
        // First loop, simply move each of them and check if one of them is out of bounds
        for mut t in enemies.iter_mut() {
            t.translation.x += control.dir * control.step;

            // Check if the next step will make this enemy go off screen
            if !switch_dir
                && (t.translation.x + control.dir * control.step).abs() >= 0.4 * win.width()
            {
                switch_dir = true;
            }
        }
        if switch_dir {
            control.just_switched = true;
            control.dir = -control.dir;
        }
    } else {
        control.just_switched = false;
        // Second loop, make everyone step downwards
        for mut t in enemies.iter_mut() {
            t.translation.y -= control.y_step;

            if t.translation.y <= -0.4 * win.height() {
                // Game over here
                println!("Game over");
                state.set(AppState::Menu).unwrap();
                return;
            }
        }
    }
}

fn clear_enemies(
    mut coms: Commands,
    enemies: Query<Entity, With<Enemy>>,
    mut spawn_enemies: ResMut<SpawnEnemies>,
) {
    for e in enemies.iter() {
        coms.entity(e).despawn();
    }

    // Reset spawn data
    *spawn_enemies = SpawnEnemies::default();
}
