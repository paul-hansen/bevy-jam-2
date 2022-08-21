mod boids;
mod math;

use crate::boids::{update_boid_transforms, Boid, BoidSettings};
use crate::math::how_much_right_or_left;
use bevy::asset::AssetServerSettings;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_inspector_egui::InspectorPlugin;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use math::Average;
use std::f32::consts::PI;
use turborand::prelude::*;

const SCENE_HEIGHT: f32 = 800.0;
const BOID_COUNT: usize = 500;

fn main() {
    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        fit_canvas_to_parent: true,
        ..Default::default()
    })
    .insert_resource(Msaa { samples: 4 })
    .insert_resource(AssetServerSettings {
        watch_for_changes: true,
        ..default()
    })
    .insert_resource(BoidSettings::default())
    .insert_resource(ClearColor(Color::hex("6c99c0").unwrap()))
    .add_plugins(DefaultPlugins)
    .add_plugin(InspectorPlugin::<BoidSettings>::new())
    .add_plugin(DebugLinesPlugin::default())
    .add_startup_system(setup)
    .add_system(update_boid_transforms);

    #[cfg(debug_assertions)]
    app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());

    app.run();
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
                .with_scale(Vec3::splat(0.01)),
                ..Default::default()
            })
            .insert(Name::new(format!("Boid {x}")))
            .insert(Boid::default());
    }
}
