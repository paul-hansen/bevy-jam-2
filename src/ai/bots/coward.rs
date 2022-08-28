use crate::math::direction_to_turn_away_from_target;
use crate::{BoidAveragedInputs, Leader};
use bevy::prelude::*;
use std::fmt::Formatter;

/// A bot that always boosts
#[derive(Default, Component)]
pub struct Coward {}

impl std::fmt::Display for Coward {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Coward")
    }
}

#[allow(clippy::type_complexity)]
pub fn update(
    mut query: Query<(Entity, &Transform, &mut BoidAveragedInputs), (With<Coward>, With<Leader>)>,
    leaders: Query<(Entity, &Transform), With<Leader>>,
) {
    let leaders: Vec<_> = leaders.iter().map(|(e, t)| (e, *t)).collect();

    for (entity, transform, mut inputs) in query.iter_mut() {
        if let Some(closest_leader) = leaders
            .iter()
            .filter(|(e, _)| *e != entity)
            .map(|(_, t)| (t.translation.distance_squared(transform.translation), t))
            .min_by(|(a, _), (b, _)| a.total_cmp(b))
        {
            if closest_leader.0 < 300.0f32.powf(2.0) {
                inputs.add_turn(direction_to_turn_away_from_target(
                    transform,
                    closest_leader.1.translation.truncate(),
                ));
                inputs.add_speed(1.0);
            } else {
                inputs.add_speed(-1.0);
            }
        }
    }
}
