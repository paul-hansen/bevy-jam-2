mod boids;
mod camera;
mod math;
mod ui;

use crate::boids::{
    calculate_alignment_inputs, calculate_cohesion_inputs, calculate_separation_inputs,
    clear_inputs, leader_defeated, leader_removed, propagate_boid_color, update_boid_color,
    update_boid_neighbors, update_boid_transforms, Boid, BoidAveragedInputs, BoidColor,
    BoidNeighborsCaptureRange, BoidNeighborsSeparation, BoidSettings, GameEvent, Leader, Velocity,
};
use crate::camera::{camera_zoom, update_camera_follow_system, Camera2dFollow};
use crate::math::how_much_right_or_left;
use crate::ui::UiAppPlugin;
use bevy::asset::AssetServerSettings;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::WindowMode;
use bevy_egui_kbgp::KbgpPlugin;
use bevy_inspector_egui::plugin::InspectorWindows;
use bevy_inspector_egui::{InspectorPlugin, RegisterInspectable};
use bevy_prototype_debug_lines::DebugLinesPlugin;
use leafwing_input_manager::prelude::*;
use std::f32::consts::PI;
use turborand::prelude::*;

const SCENE_HEIGHT: f32 = 500.0;
const BOID_COUNT: usize = 400;
const ARENA_RADIUS: f32 = 1200.0;
const ARENA_PADDING: f32 = 70.0;
const BOID_SCALE: Vec3 = Vec3::splat(0.01);
const LEADER_SCALE: Vec3 = Vec3::splat(0.014);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Intro,
    Setup,
    PauseMenu,
    Playing,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        fit_canvas_to_parent: true,
        mode: WindowMode::BorderlessFullscreen,
        ..Default::default()
    })
    .insert_resource(Msaa { samples: 4 })
    .insert_resource(AssetServerSettings {
        watch_for_changes: true,
        ..default()
    })
    .insert_resource(BoidSettings::default())
    .insert_resource(ClearColor(Color::BLACK))
    .add_plugins(DefaultPlugins)
    .add_plugin(InspectorPlugin::<BoidSettings>::new())
    .add_plugin(DebugLinesPlugin::default())
    .add_plugin(InputManagerPlugin::<Actions>::default())
    .add_plugin(InputManagerPlugin::<GlobalActions>::default())
    .add_plugin(UiAppPlugin)
    .add_plugin(KbgpPlugin)
    .register_inspectable::<BoidNeighborsCaptureRange>()
    .register_inspectable::<BoidNeighborsSeparation>()
    .register_inspectable::<Camera2dFollow>()
    .register_type::<BoidAveragedInputs>()
    .add_state::<AppState>(AppState::Intro)
    .add_event::<GameEvent>()
    .add_startup_system(setup)
    .add_system_set(SystemSet::on_enter(AppState::Setup).with_system(setup_game))
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
    .add_system_set(SystemSet::on_update(AppState::Playing).with_system(update_boid_transforms))
    .add_system_to_stage(CoreStage::Last, clear_inputs)
    .add_system(update_boid_color)
    .add_system(update_camera_follow_system)
    .add_system(camera_zoom)
    .add_system(leader_defeated)
    .add_system_to_stage(CoreStage::PreUpdate, propagate_boid_color)
    .add_system_to_stage(CoreStage::PostUpdate, leader_removed);

    // Might disable this for release builds in the future
    app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new())
        .insert_resource(bevy_inspector_egui::WorldInspectorParams {
            ignore_components: Default::default(),
            read_only_components: Default::default(),
            sort_components: false,
            enabled: false,
            highlight_changes: true,
            ..default()
        });

    app.run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Actions {
    Move,
    CameraZoom,
}

/// Actions that any player can trigger
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum GlobalActions {
    ToggleMenu,
    ToggleBoidSettings,
    ToggleWorldInspector,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut inspector_windows: ResMut<InspectorWindows>,
    mut app_state: ResMut<bevy::prelude::State<AppState>>,
) {
    let inspector_window_data = inspector_windows.window_data_mut::<BoidSettings>();
    inspector_window_data.visible = false;
    commands
        .spawn_bundle(ColorMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Circle::new(ARENA_RADIUS)))
                .into(),
            material: materials.add(ColorMaterial::from(Color::hex("6c99c0").unwrap())),
            ..default()
        })
        .insert_bundle(InputManagerBundle {
            action_state: default(),
            input_map: {
                InputMap::<GlobalActions>::default()
                    .insert(KeyCode::Escape, GlobalActions::ToggleMenu)
                    .insert(GamepadButtonType::East, GlobalActions::ToggleMenu)
                    .insert(GamepadButtonType::Select, GlobalActions::ToggleMenu)
                    .insert(GamepadButtonType::Start, GlobalActions::ToggleMenu)
                    .insert_chord(
                        [KeyCode::LAlt, KeyCode::B],
                        GlobalActions::ToggleBoidSettings,
                    )
                    .insert_chord(
                        [KeyCode::RAlt, KeyCode::B],
                        GlobalActions::ToggleBoidSettings,
                    )
                    .insert_chord(
                        [KeyCode::LAlt, KeyCode::N],
                        GlobalActions::ToggleWorldInspector,
                    )
                    .insert_chord(
                        [KeyCode::RAlt, KeyCode::N],
                        GlobalActions::ToggleWorldInspector,
                    )
                    .build()
            },
        });

    if let Err(e) = app_state.overwrite_set(AppState::Setup) {
        error!("Error while setting up game: {e}")
    } else {
        info!("App state transitioned to Playing")
    };
}

#[derive(Component, Debug, Copy, Clone)]
pub struct SceneRoot;

fn setup_game(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut app_state: ResMut<bevy::prelude::State<AppState>>,
    scene_root: Query<Entity, With<SceneRoot>>,
) {
    // Spawn a root node to attach everything to so we can recursively delete everything
    // when reloading.
    if let Ok(root) = scene_root.get_single() {
        info!("Restarting");
        commands.entity(root).despawn_recursive();
    }
    let scene_root = commands
        .spawn()
        .insert(Name::new("Root"))
        .insert(SceneRoot)
        .insert_bundle(SpatialBundle::default())
        .id();

    let rand = Rng::new();
    for x in 0..BOID_COUNT {
        let r = (ARENA_RADIUS - ARENA_PADDING) * rand.f32();
        let theta = rand.f32() * 2.0 * PI;
        let entity = commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("bird.png"),
                transform: Transform::from_xyz(r * theta.cos(), r * theta.sin(), 5.0)
                    .with_rotation(Quat::from_rotation_z(rand.f32_normalized() * PI * 2.0))
                    .with_scale(match x {
                        x if x < 4 => LEADER_SCALE,
                        _ => BOID_SCALE,
                    }),
                ..Default::default()
            })
            .insert(Name::new(format!("Boid {x}")))
            .insert(BoidNeighborsSeparation::default())
            .insert(BoidNeighborsCaptureRange::default())
            .insert(ActionState::<Actions>::default())
            .insert(BoidAveragedInputs::default())
            .insert(Boid::default())
            .insert(Velocity::default())
            .id();

        if let Some(color) = BoidColor::from_index(x) {
            commands.entity(entity).insert(color).insert(Leader);
        }

        if x == 0 {
            let camera = commands
                .spawn_bundle(Camera2dBundle {
                    projection: OrthographicProjection {
                        scaling_mode: ScalingMode::FixedVertical(SCENE_HEIGHT),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Camera2dFollow {
                    target: entity,
                    offset: Default::default(),
                })
                .id();
            commands.entity(scene_root).add_child(camera);
            commands.entity(entity).insert(
                InputMap::<Actions>::default()
                    .insert(VirtualDPad::wasd(), Actions::Move)
                    .insert(VirtualDPad::arrow_keys(), Actions::Move)
                    .insert(DualAxis::left_stick(), Actions::Move)
                    .insert(VirtualDPad::dpad(), Actions::CameraZoom)
                    .insert(VirtualDPad::mouse_wheel(), Actions::CameraZoom)
                    .build(),
            );
        }

        commands.entity(scene_root).add_child(entity);
    }
    if let Err(e) = app_state.overwrite_set(AppState::Playing) {
        error!("Error while starting game: {e}")
    } else {
        info!("App state transitioned to Playing")
    };
}
