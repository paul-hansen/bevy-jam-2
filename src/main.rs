mod boids;
mod math;

use crate::boids::{
    calculate_alignment_inputs, calculate_cohesion_inputs, calculate_separation_inputs,
    update_boid_neighbors, update_boid_transforms, Boid, BoidNeighborsAlignment,
    BoidNeighborsCohesion, BoidNeighborsSeparation, BoidSettings, BoidTurnDirectionInputs,
};
use crate::math::how_much_right_or_left;
use bevy::asset::AssetServerSettings;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_inspector_egui::{InspectorPlugin, RegisterInspectable};
use bevy_prototype_debug_lines::DebugLinesPlugin;
use math::Average;
use std::f32::consts::PI;
use turborand::prelude::*;

const SCENE_HEIGHT: f32 = 500.0;
const BOID_COUNT: usize = 400;

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
    .register_inspectable::<BoidNeighborsAlignment>()
    .register_inspectable::<BoidNeighborsCohesion>()
    .register_inspectable::<BoidNeighborsSeparation>()
    .register_type::<BoidTurnDirectionInputs>()
    .add_startup_system(setup)
    .add_system_to_stage(CoreStage::First, update_boid_neighbors)
    .add_system_to_stage(CoreStage::PreUpdate, calculate_cohesion_inputs)
    .add_system_to_stage(
        CoreStage::PreUpdate,
        calculate_separation_inputs.after(calculate_cohesion_inputs),
    )
    .add_system_to_stage(
        CoreStage::PreUpdate,
        calculate_alignment_inputs.after(calculate_separation_inputs),
    )
    .add_system_to_stage(CoreStage::Last, update_boid_transforms);

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
                    (SCENE_HEIGHT / 2.0) * rand.f32_normalized(),
                    (SCENE_HEIGHT / 2.0) * rand.f32_normalized(),
                    x as f32,
                )
                .with_rotation(Quat::from_rotation_z(rand.f32_normalized() * PI * 2.0))
                .with_scale(Vec3::splat(0.01)),
                ..Default::default()
            })
            .insert(Name::new(format!("Boid {x}")))
            .insert(BoidNeighborsAlignment::default())
            .insert(BoidNeighborsSeparation::default())
            .insert(BoidNeighborsCohesion::default())
            .insert(BoidTurnDirectionInputs::default())
            .insert(Boid::default());
    }
}
