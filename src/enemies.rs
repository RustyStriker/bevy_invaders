use crate::{AppState, bullets::{Bullet, SpawnBulletEvent}, collisions::*};
use bevy::{core::FixedTimestep, prelude::*};
use rand::Rng;

pub struct Enemies;

impl Plugin for Enemies {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<RoundOver>();

        app.add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(kill_on_collision.system())
                .with_system(round_over.system())
                .with_system(spawn_enemies_on_event.system())
                .with_system(enemy_shooting.system()),
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
    let bullet_color = colors.add(Color::ORANGE_RED.into());
    coms.insert_resource(DefaultEnemyColor(enemy_color, bullet_color));
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
            center: Vec2::new(0.0, 250.0),
            round: 1,
        }
    }
}

/// First color is enemy itself, second is the bullet
struct DefaultEnemyColor(Handle<ColorMaterial>, Handle<ColorMaterial>);

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

        // We gonna keep a vec of the entities so we could access them
        // when we spawn an entity on the row beneath
        let mut row_above : Vec<Entity> = Vec::with_capacity(spawn.cols as usize);
        
        // #TheTrueRustWay
        (0..spawn.rows).for_each(|i| {
            let i = (spawn.rows - i -1) as f32; // we gonna spawn the bottom first

            (0..spawn.cols).for_each(|k| {
                let k = k as f32;

                let position = start + Vec2::new(k, i) * (enemy_size + spacing);


                let mut rng = rand::thread_rng();

                let cycle = ShootCycle {
                    timer: rng.gen::<f32>() * 10.0,
                    reset : rng.gen::<f32>() * 15.0 + 3.0,
                };

                // Spawn the enemy
                let e = coms.spawn_bundle(SpriteBundle {
                    sprite: Sprite::new(enemy_size),
                    material: enemy_color.0.clone(),
                    transform: Transform::from_translation(position.extend(0.0)),
                    ..Default::default()
                })
                .insert(Enemy::default())
                .insert(Aabb::new(enemy_size.extend(0.0) * 0.5))
                .insert(cycle)
                .id()
                ;

                // try to get the entity above
                if let Some(&above) = row_above.get(k as usize) {
                    coms.entity(above).insert(Enemy { enemy_above : Some(e), hold_fire : true });
                    // we know that we have a k position in row_above so we replace it
                    row_above[k as usize] = e;
                }
                else {
                    // no k position, we are probably(hopefully) only at the first line
                    row_above.push(e);
                }
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

#[derive(Default, Debug, Clone, Reflect)]
pub struct Enemy {
    pub enemy_above : Option<Entity>,
    pub hold_fire : bool
}
struct ShootCycle {
    pub timer : f32,
    pub reset : f32,
}

/// Kills `Enemy` entities on collision events
fn kill_on_collision(
    mut coms: Commands,
    enemy_entities: Query<Entity, With<Enemy>>,
    mut colls: EventReader<Collision>,
    mut enemies: Query<&mut Enemy>, // so we could tell the enemies to shoot once more
) {
    for c in colls.iter() {
        if let Ok(e) = enemy_entities.get(c.0) {
            // Allow the one above to shoot pretty much
            let above = match enemies.get_component::<Enemy>(e) {
                Ok(enemy) => enemy.enemy_above,
                Err(_) => None
            };
            if let Some(above) = above {
                if let Ok(mut enemy) = enemies.get_mut(above) {
                    enemy.hold_fire = false; // allow the one above to shoot
                }
            }

            coms.entity(e).despawn();
        } else if let Ok(e) = enemy_entities.get(c.1) {
            // Allow the one above to shoot pretty much
            let above = match enemies.get_component::<Enemy>(e) {
                Ok(enemy) => enemy.enemy_above,
                Err(_) => None
            };
            if let Some(above) = above {
                if let Ok(mut enemy) = enemies.get_mut(above) {
                    enemy.hold_fire = false; // allow the one above to shoot
                }
            }

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

fn enemy_shooting(
    time: Res<Time>,
    enemy_colors : Res<DefaultEnemyColor>,
    mut query: Query<(&Enemy, &Transform, &mut ShootCycle)>,
    mut writer: EventWriter<SpawnBulletEvent>,
) {
    let delta = time.delta_seconds();

    for (e, t, mut cycle) in query.iter_mut() {
        if !e.hold_fire {
            cycle.timer -= delta;
            if cycle.timer <= 0.0 {
                cycle.timer = cycle.reset;
                // spawn a bullet for me

                let bullet = Bullet { speed : Vec2::new(0.0,-90.0) };
                let position = Vec2::new(t.translation.x, t.translation.y);
                let position = position + Vec2::new(0.0, -25.0);


                writer.send(SpawnBulletEvent {
                    bullet,
                    position,
                    color: enemy_colors.1.clone(),
                    size: Vec2::splat(10.0),
                })
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
