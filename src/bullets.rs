use bevy::prelude::*;

use crate::{
    collisions::{Aabb, Collision},
    enemies::RoundOver,
    AppState,
};
pub struct BulletsPlugin;

impl Plugin for BulletsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<SpawnBulletEvent>();

        app.add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(spawn_bullet_on_event.system())
                .with_system(bullets_movement.system())
                .with_system(kill_bullets.system())
                .with_system(despawn_bullets_on_round_over.system()),
        )
        .add_system_set(SystemSet::on_exit(AppState::Game).with_system(despawn_bullets.system()));
    }
}

pub struct SpawnBulletEvent {
    pub bullet: Bullet,
    pub position: Vec2,
    pub color: Handle<ColorMaterial>,
    pub size: Vec2,
}

fn spawn_bullet_on_event(mut coms: Commands, mut events: EventReader<SpawnBulletEvent>) {
    for e in events.iter() {
        let bullet = e.bullet.clone();

        coms.spawn_bundle(SpriteBundle {
            sprite: Sprite::new(e.size),
            material: e.color.clone(),
            transform: Transform::from_translation(e.position.extend(0.0)),
            ..Default::default()
        })
        .insert(bullet)
        .insert(Aabb::new(e.size.extend(0.0) * 0.5));
    }
}

#[derive(Clone)]
pub struct Bullet {
    pub speed: Vec2,
}

fn bullets_movement(
    mut coms: Commands, // if we need to despawn here
    wins: Res<Windows>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &Bullet)>,
) {
    let main_win = wins.get_primary().unwrap();
    let win_size = Vec2::new(main_win.width(), main_win.height()) * 0.5;

    for (e, mut t, b) in query.iter_mut() {
        t.translation += b.speed.extend(0.0) * time.delta_seconds();

        let t2: Vec2 = t.translation.into();

        if t2.x.abs() > win_size.x || t2.y.abs() > win_size.y {
            coms.entity(e).despawn();
        }
    }
}

fn kill_bullets(mut coms: Commands, mut colls: EventReader<Collision>, bullets: Query<&Bullet>) {
    for c in colls.iter() {
        let c0 = bullets.get(c.0).is_ok();
        let c1 = bullets.get(c.1).is_ok();

        // Make sure bullet v bullet collision doesnt destroy the bullets :D
        if c0 && !c1 {
            coms.entity(c.0).despawn();
        } else if c1 && !c0 {
            coms.entity(c.1).despawn();
        }
    }
}

fn despawn_bullets(mut coms: Commands, bullets: Query<Entity, With<Bullet>>) {
    for e in bullets.iter() {
        coms.entity(e).despawn();
    }
}

fn despawn_bullets_on_round_over(
    mut coms: Commands,
    mut round_over: EventReader<RoundOver>,
    bullets: Query<Entity, With<Bullet>>,
) {
    for _ in round_over.iter() {
        for e in bullets.iter() {
            coms.entity(e).despawn();
        }
    }
}
