use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct Physics;

impl Plugin for Physics {
    fn build(&self, app: &mut AppBuilder) {
        app.add_stage("COLLISION", SystemStage::single_threaded());
        app.add_system_to_stage("COLLISION", collision_system.system());
        app.add_event::<Collision>();
    }
}

#[derive(Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct Aabb {
    pub extents: Vec3,
}

impl Aabb {
    pub fn new(extents: Vec3) -> Aabb {
        Aabb { extents }
    }
}

pub struct Collision(pub Entity, pub Entity);

/// Simple overlap detection
pub fn collision_system(
    shapes: Query<(Entity, &Aabb, &Transform)>,
    mut writer: EventWriter<Collision>,
) {
    let mut passed: Vec<(Entity, Vec3, Vec3)> = Vec::new(); // I dont care about performance that much tbh
                                                            //						      min , max

    for (e, s, t) in shapes.iter() {
        let min = t.translation - s.extents;
        let max = t.translation + s.extents;

        for (e2, min2, max2) in passed.iter() {
            if min.x < max2.x && min.y < max2.y && min2.x < max.x && min2.y < max.y {
                writer.send(Collision(e, *e2));
            }
        }

        passed.push((e, min, max));
    }
}
