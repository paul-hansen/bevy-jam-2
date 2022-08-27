use crate::math::direction_to_turn_away_from_target;
use crate::{
    how_much_right_or_left, Boid, BoidAveragedInputs, BoidColor, BoidNeighborsSeparation,
    BoidSettings, Leader, Velocity,
};
use bevy::prelude::*;
use bevy_prototype_debug_lines::DebugLines;

#[allow(clippy::type_complexity)]
pub fn calculate_cohesion_inputs(
    mut query: Query<
        (&Transform, &mut BoidAveragedInputs, &BoidColor, &Velocity),
        (With<Boid>, Without<Leader>),
    >,
    leader_query: Query<(&Transform, &BoidColor, &Velocity), With<Leader>>,
    mut lines: ResMut<DebugLines>,
    boid_settings: Res<BoidSettings>,
) {
    if !boid_settings.cohesion_enabled {
        return;
    }
    // Turn and move towards the leader's position if they have one.
    for (transform, mut inputs, color, velocity) in query.iter_mut() {
        if let Some((leader_transform, _, leader_velocity)) =
            leader_query.iter().find(|(_, c, _)| *c == color)
        {
            let leader_position = leader_transform.translation.truncate();

            let direction_to_target =
                (leader_position - transform.translation.truncate()).normalize();

            let turn_towards_leader_direction = how_much_right_or_left(
                transform,
                transform.translation.truncate() + direction_to_target,
            );
            let speed_up_down = match leader_velocity.forward - velocity.forward {
                x if x > 120.0 => 1.0,
                x if x > 0.0 => 0.5,
                x if x < -3.0 => -1.0,
                _ => 0.0,
            };
            if boid_settings.debug_lines {
                lines.line_gradient(
                    transform.translation,
                    (direction_to_target.extend(0.0) * 20.0) + transform.translation,
                    0.0,
                    Color::rgba(0.2, 1.0, 0.2, 1.0),
                    Color::rgba(0.2, 1.0, 0.2, 0.0),
                );
            }

            inputs.add_turn(turn_towards_leader_direction);
            inputs.add_speed(speed_up_down);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn calculate_separation_inputs(
    mut query: Query<
        (
            &Transform,
            &BoidNeighborsSeparation,
            &mut BoidAveragedInputs,
        ),
        (With<Boid>, Without<Leader>),
    >,
    transforms: Query<&Transform>,
    mut lines: ResMut<DebugLines>,
    boid_settings: Res<BoidSettings>,
) {
    if !boid_settings.separation_enabled {
        return;
    }
    for (transform, neighbors, mut inputs) in query.iter_mut() {
        transforms
            .iter_many(&neighbors.entities)
            .for_each(|target| {
                let direction =
                    (direction_to_turn_away_from_target(transform, target.translation.truncate())
                        * 2.0)
                        .clamp(-1.0, 1.0);
                // Turn away from neighbors within separation distance
                inputs.add_turn(direction);

                // Draw a line from the current entity to the target that is affecting the separation
                // Fades out farther from the current entity so it's easy to tell if both
                // entities are being affected.
                if boid_settings.debug_lines {
                    lines.line_gradient(
                        transform.translation,
                        ((target.translation - transform.translation) * 0.5)
                            + transform.translation,
                        0.0,
                        Color::rgba(1.0, 0.0, 0.0, (direction + 1.0) / 2.0),
                        Color::rgba(1.0, 0.0, 0.0, 0.2),
                    );
                }
            });
    }
}

#[allow(clippy::type_complexity)]
pub fn calculate_alignment_inputs(
    mut query: Query<
        (&Transform, &mut BoidAveragedInputs, &BoidColor),
        (With<Boid>, Without<Leader>),
    >,
    leader_query: Query<(&Transform, &BoidColor), With<Leader>>,
    mut lines: ResMut<DebugLines>,
    boid_settings: Res<BoidSettings>,
) {
    if !boid_settings.alignment_enabled {
        return;
    }
    for (transform, mut inputs, color) in query.iter_mut() {
        if let Some((leader_transform, _)) = leader_query.iter().find(|(_, c)| *c == color) {
            let average = leader_transform.up().truncate();
            if boid_settings.debug_lines {
                lines.line_colored(
                    transform.translation,
                    transform.translation + (average.extend(0.0) * 20.0),
                    0.0,
                    Color::VIOLET,
                );
            }
            inputs.add_turn(how_much_right_or_left(
                &Transform::from_rotation(transform.rotation),
                average,
            ));
        }
    }
}
