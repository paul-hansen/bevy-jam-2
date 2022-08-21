use bevy::asset::AssetServerSettings;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_inspector_egui::{Inspectable, InspectorPlugin, WorldInspectorPlugin};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use std::f32::consts::PI;
use turborand::prelude::*;

#[derive(Component, Default)]
pub struct Bird {}

const SCENE_HEIGHT: f32 = 800.0;
const BOID_COUNT: usize = 80;

#[derive(Inspectable, Debug)]
pub struct AppConfig {
    #[inspectable(min = 0.0, max = 9999.0)]
    boid_speed: f32,
    #[inspectable(min = 0.0, max = PI * 180.0)]
    boid_max_turn_rate_per_second: f32,
    #[inspectable(min = 0.0, max = 1000.0)]
    boid_separation_distance: f32,
    #[inspectable(min = 0.0, max = 1000.0)]
    boid_cohesion_distance: f32,
    debug_lines: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            boid_speed: 100.0,
            boid_max_turn_rate_per_second: PI * 1.5,
            boid_separation_distance: 50.0,
            boid_cohesion_distance: 120.0,
            debug_lines: false,
        }
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            fit_canvas_to_parent: true,
            ..Default::default()
        })
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .insert_resource(AppConfig::default())
        .insert_resource(ClearColor(Color::hex("94aed6").unwrap()))
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(InspectorPlugin::<AppConfig>::new())
        .add_plugin(DebugLinesPlugin::default())
        .add_startup_system(setup)
        .add_system(update_boid_transforms)
        .run();
}

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical(SCENE_HEIGHT),
            ..Default::default()
        },
        ..Default::default()
    });

    let rand = Rng::new();

    for x in 0..BOID_COUNT {
        commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("bird.png"),
                transform: Transform::from_xyz(
                    300.0 * rand.f32_normalized(),
                    300.0 * rand.f32_normalized(),
                    x as f32,
                )
                .with_rotation(Quat::from_rotation_z(rand.f32_normalized() * PI * 2.0))
                .with_scale(Vec3::splat(0.03)),
                ..Default::default()
            })
            .insert(Name::new(format!("Boid {x}")))
            .insert(Bird::default());
    }
}

fn update_boid_transforms(
    mut transforms: Query<&mut Transform, With<Bird>>,
    time: Res<Time>,
    windows: Res<Windows>,
    mut lines: ResMut<DebugLines>,
    app_config: Res<AppConfig>,
) {
    let boid_positions: Vec<Vec3> = transforms.iter().map(|t| t.translation).collect();
    for mut transform in transforms.iter_mut() {
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

        let cohesion_turn_factor = boid_cohesion(
            &boid_positions,
            &transform,
            match app_config.debug_lines {
                true => Some(&mut lines),
                false => None,
            },
            app_config.boid_cohesion_distance,
            app_config.boid_separation_distance,
        );

        if !cohesion_turn_factor.is_nan() {
            turn_directions.push(cohesion_turn_factor);
        }

        turn_directions.append(&mut boid_separation(
            &transform,
            app_config.boid_separation_distance,
            &boid_positions,
            match app_config.debug_lines {
                true => Some(&mut lines),
                false => None,
            },
        ));

        let final_turn_direction = match turn_directions.is_empty() {
            true => 0.0,
            false => turn_directions.iter().sum::<f32>() / turn_directions.len() as f32,
        };

        // move forward
        let forward = transform.up();
        transform.rotate_z(
            final_turn_direction * app_config.boid_max_turn_rate_per_second * time.delta_seconds(),
        );
        transform.translation += forward * time.delta_seconds() * app_config.boid_speed;

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
    boid_positions: &[Vec3],
    transform: &Transform,
    lines: Option<&mut ResMut<DebugLines>>,
    boid_cohesion_distance: f32,
    boid_separation_distance: f32,
) -> f32 {
    // Move towards the average position of other boids

    let boid_positions: Vec<Vec3> = boid_positions
        .iter()
        .filter(|t| {
            let distance = transform.translation.distance(**t);
            distance < boid_cohesion_distance && distance > boid_separation_distance
        })
        // Exclude itself
        .filter(|t| **t != transform.translation)
        .cloned()
        .collect();
    let total: Vec3 = boid_positions.iter().sum();

    let average_position_of_near_boids: Vec3 = total / (boid_positions.len() as f32);

    let direction_to_target = (average_position_of_near_boids - transform.translation)
        .truncate()
        .normalize();
    let turn_direction_to_center_of_near = -transform.right().truncate().dot(direction_to_target);
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
    boid_positions: &[Vec3],
    mut lines: Option<&mut DebugLines>,
) -> Vec<f32> {
    let position = transform.translation;
    let separation_boids: Vec<Vec3> = boid_positions
        .iter()
        .filter(|t| t.distance(position) < boid_separation_distance)
        .filter(|t| **t != position)
        .cloned()
        .collect();

    separation_boids
        .iter()
        .map(|target| {
            let direction_to_target = (*target - position).truncate().normalize();

            // The dot product when used with normalized vectors tells you how parallel
            // a vector is to another.
            // Negative values means it is facing the opposite way,
            // so if we use the right facing vector, the result will be -1.0 to 1.0 based on
            // how much to the right the target is from the current boid.
            let direction = transform.right().truncate().dot(direction_to_target);
            if let Some(lines) = &mut lines {
                lines.line_gradient(
                    transform.translation,
                    ((*target - transform.translation) * 0.5) + transform.translation,
                    0.0,
                    Color::rgba(1.0, 0.0, 0.0, (direction + 1.0) / 2.0),
                    Color::rgba(1.0, 0.0, 0.0, 0.2),
                );
            }
            direction
        })
        .collect()
}
