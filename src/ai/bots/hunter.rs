use crate::math::direction_to_turn_towards_target;
use crate::{BoidAveragedInputs, BoidColor, Leader};
use bevy::prelude::*;
use bevy::utils::HashMap;
use std::fmt::Formatter;

const SIGHT_RANGE_SQUARED: f32 = 500.0 * 500.0;

/// A bot that always boosts
#[derive(Default, Component)]
pub struct Hunter {}

impl std::fmt::Display for Hunter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hunter")
    }
}

#[allow(clippy::type_complexity)]
pub fn update(
    mut query: Query<
        (Entity, &Transform, &mut BoidAveragedInputs, &BoidColor),
        (With<Hunter>, With<Leader>),
    >,
    leaders: Query<(Entity, &Transform, &BoidColor), With<Leader>>,
    boid_colors: Query<&BoidColor>,
) {
    let mut color_counts: HashMap<BoidColor, usize> = HashMap::new();
    for other_color in boid_colors.iter() {
        let count = color_counts.entry(*other_color).or_insert(0);
        *count += 1;
    }
    let leaders: Vec<_> = leaders.iter().map(|(e, t, c)| (e, *t, c)).collect();
    for (entity, transform, mut inputs, color) in query.iter_mut() {
        if let Some(closest_leader) = leaders
            .iter()
            // Don't consider self as a target
            .filter(|(e, _, _)| *e != entity)
            // Don't consider targets that have more followers than us
            .filter(|(_, _, c)| {
                color_counts.get(*c).cloned().unwrap_or_default()
                    < color_counts.get(color).cloned().unwrap_or_default()
            })
            .map(|(_, t, c)| (t.translation.distance_squared(transform.translation), t, c))
            // limit sight range
            .filter(|(d, _, _)| *d < SIGHT_RANGE_SQUARED)
            // find the leader with the least followers
            .min_by(|(_, _, a), (_, _, b)| color_counts[a].cmp(&color_counts[b]))
        {
            inputs.add_turn(direction_to_turn_towards_target(
                transform,
                closest_leader.1.translation.truncate(),
            ));
            inputs.add_speed(1.0);
        } else {
            inputs.add_speed(-1.0);
        }
    }
}
