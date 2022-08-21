use crate::{how_much_right_or_left, Average, SCENE_HEIGHT};
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_prototype_debug_lines::DebugLines;
use std::f32::consts::PI;

#[derive(Inspectable, Debug)]
pub struct BoidSettings {
    cohesion_enabled: bool,
    separation_enabled: bool,
    alignment_enabled: bool,
    #[inspectable(min = 0.0, max = 9999.0)]
    speed: f32,
    #[inspectable(min = 0.0, max = PI * 180.0)]
    max_turn_rate_per_second: f32,
    #[inspectable(min = 0.0, max = 1000.0)]
    separation_distance: f32,
    #[inspectable(min = 0.0, max = 1000.0)]
    cohesion_distance: f32,
    #[inspectable(min = 0.0, max = 1000.0)]
    alignment_distance: f32,
    debug_lines: bool,
}

impl Default for BoidSettings {
    fn default() -> Self {
        Self {
            cohesion_enabled: true,
            separation_enabled: true,
            alignment_enabled: true,
            speed: 80.0,
            max_turn_rate_per_second: PI * 5.8,
            separation_distance: 72.0,
            cohesion_distance: 120.0,
            alignment_distance: 120.0,
            debug_lines: false,
        }
    }
}

#[derive(Component, Default)]
pub struct Boid {}

pub fn update_boid_transforms(
    mut boid_transforms: Query<&mut Transform, With<Boid>>,
    time: Res<Time>,
    windows: Res<Windows>,
    mut lines: ResMut<DebugLines>,
    app_config: Res<BoidSettings>,
) {
    let boid_transforms_copy: Vec<Transform> = boid_transforms.iter().cloned().collect();
    for mut transform in boid_transforms.iter_mut() {
        let mut turn_directions: Vec<f32> = Vec::new();
        let position = transform.translation;

        if app_config.debug_lines {
            lines.line_colored(
                position,
                position + (transform.up() * 20.0),
                0.0,
                Color::MIDNIGHT_BLUE,
            );
        }
        // if app_config.debug_lines {
        //     let mut line_transform = *transform;
        //     line_transform.rotate_z((time.time_since_startup().as_secs_f32() % (PI * 2.0)) * 20.0);
        //     lines.line_colored(
        //         position,
        //         position + (line_transform.up() * app_config.boid_cohesion_distance),
        //         0.08,
        //         Color::rgba_u8(0, 255, 0, 80),
        //     );
        //
        //     lines.line_colored(
        //         position,
        //         position + (line_transform.up() * app_config.boid_separation_distance),
        //         0.08,
        //         Color::rgba_u8(255, 0, 0, 200),
        //     );
        // }

        if app_config.cohesion_enabled {
            let cohesion_turn_factor = boid_cohesion(
                &boid_transforms_copy,
                &transform,
                match app_config.debug_lines {
                    true => Some(&mut lines),
                    false => None,
                },
                app_config.cohesion_distance,
                app_config.separation_distance,
            );

            if !cohesion_turn_factor.is_nan() {
                turn_directions.push(cohesion_turn_factor);
            }
        }

        if app_config.separation_enabled {
            turn_directions.append(&mut boid_separation(
                &transform,
                app_config.separation_distance,
                &boid_transforms_copy,
                match app_config.debug_lines {
                    true => Some(&mut lines),
                    false => None,
                },
            ));
        }

        if app_config.alignment_enabled {
            if let Some(direction) = boid_alignment(
                &transform,
                &boid_transforms_copy,
                app_config.alignment_distance,
                match app_config.debug_lines {
                    true => Some(&mut lines),
                    false => None,
                },
            ) {
                turn_directions.push(direction);
            }
        }
        let final_turn_direction = match turn_directions.is_empty() {
            true => 0.0,
            false => turn_directions.into_iter().avg(),
        };

        // move forward
        let forward = transform.up();
        transform.rotate_z(
            final_turn_direction * app_config.max_turn_rate_per_second * time.delta_seconds(),
        );
        transform.translation += forward * time.delta_seconds() * app_config.speed;

        // Wrap around when a boid reaches the edge of the window
        let wnd = windows.get_primary().unwrap();
        let scene_width = SCENE_HEIGHT * wnd.width() as f32 / wnd.height() as f32;
        let scene_width_half = scene_width / 2.0;

        if position.x.abs() > scene_width_half {
            transform.translation.x =
                (position.x * -1.0).clamp(-scene_width_half, scene_width_half);
        }

        let scene_height_half = SCENE_HEIGHT / 2.0;

        if position.y.abs() > scene_height_half {
            transform.translation.y =
                (position.y * -1.0).clamp(-scene_height_half, scene_height_half);
        }
    }
}

fn boid_cohesion(
    boid_transforms: &[Transform],
    transform: &Transform,
    lines: Option<&mut ResMut<DebugLines>>,
    boid_cohesion_distance: f32,
    boid_separation_distance: f32,
) -> f32 {
    // Move towards the average position of other boids

    let average_position_of_near_boids: Vec2 = boid_transforms
        .iter()
        .filter(|t| {
            let distance = transform.translation.distance(t.translation);
            distance < boid_cohesion_distance && distance > boid_separation_distance
        })
        // Exclude itself
        .filter(|t| t.translation != transform.translation)
        .map(|t| t.translation.truncate())
        .avg();

    let direction_to_target =
        (average_position_of_near_boids - transform.translation.truncate()).normalize();

    let turn_direction_to_center_of_near = -how_much_right_or_left(transform, &direction_to_target);
    if let Some(lines) = lines {
        lines.line_gradient(
            transform.translation,
            (direction_to_target.extend(0.0) * 20.0) + transform.translation,
            0.0,
            Color::rgba(0.2, 1.0, 0.2, 1.0),
            Color::rgba(0.2, 1.0, 0.2, 0.0),
        );
    }
    turn_direction_to_center_of_near
}

fn boid_separation(
    transform: &Transform,
    boid_separation_distance: f32,
    boid_positions: &[Transform],
    mut lines: Option<&mut DebugLines>,
) -> Vec<f32> {
    let position = transform.translation;
    boid_positions
        .iter()
        .filter(|t| t.translation.distance(position) < boid_separation_distance)
        .filter(|t| t.translation != position)
        .map(|target| {
            let target = target.translation;
            let direction = how_much_right_or_left(transform, &target.truncate());
            if let Some(lines) = &mut lines {
                lines.line_gradient(
                    transform.translation,
                    ((target - transform.translation) * 0.5) + transform.translation,
                    0.0,
                    Color::rgba(1.0, 0.0, 0.0, (direction + 1.0) / 2.0),
                    Color::rgba(1.0, 0.0, 0.0, 0.2),
                );
            }
            direction
        })
        .collect()
}

fn boid_alignment(
    transform: &Transform,
    boid_transforms: &[Transform],
    boid_alignment_distance: f32,
    mut lines: Option<&mut DebugLines>,
) -> Option<f32> {
    let average: Vec2 = boid_transforms
        .iter()
        .filter(|t| {
            t.translation
                .truncate()
                .distance_squared(transform.translation.truncate())
                < boid_alignment_distance * boid_alignment_distance
        })
        .map(|t| t.up().truncate())
        .avg()
        .normalize();
    match average.is_nan() {
        true => None,
        false => {
            if let Some(lines) = &mut lines {
                lines.line_colored(
                    transform.translation,
                    transform.translation + (average.extend(0.0) * 20.0),
                    0.0,
                    Color::VIOLET,
                );
            }
            let p =
                -how_much_right_or_left(&Transform::from_rotation(transform.rotation), &average);

            Some(p)
        }
    }
}
