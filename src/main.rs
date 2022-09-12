mod ai;
mod boids;
mod camera;
mod math;
mod quadtree;
mod round;
mod ui;
mod viewports;

use crate::ai::bots::Bot;
use crate::boids::{
    clear_inputs, leader_added, leader_defeated, leader_removed, propagate_boid_color,
    update_boid_color, update_boid_neighbors, update_boid_transforms, update_quad_tree, Boid,
    BoidAveragedInputs, BoidColor, BoidNeighborsCaptureRange, BoidNeighborsSeparation,
    BoidSettings, GameEvent, Leader, Velocity,
};
use crate::camera::{
    camera_zoom, remove_camera_follow_target_on_capture, update_camera_follow_many_system,
    update_camera_follow_system, Camera2dFollow, Camera2dFollowMany, CameraFollowTarget,
};
use crate::math::how_much_right_or_left;
use crate::round::{MultiplayerMode, PlayerType, RoundSettings};
use crate::ui::Logo;
use crate::viewports::{
    set_camera_viewports, PlayerViewports, ViewportLayoutPreference, ViewportRelative,
};
use bevy::asset::AssetServerSettings;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::ecs::schedule::ShouldRun;
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
const ARENA_PADDING: f32 = 100.0;
const BOID_SCALE: Vec3 = Vec3::splat(0.01);
const LEADER_SCALE: Vec3 = Vec3::splat(0.014);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    Title,
    LoadRound,
    GameOver,
    Playing,
    CustomGameMenu,
    PauseMenu,
    SettingsMenu,
}

#[derive(Debug, Clone)]
pub struct Winner {
    pub color: BoidColor,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        fit_canvas_to_parent: true,
        mode: WindowMode::Windowed,
        ..Default::default()
    })
    .insert_resource(Msaa { samples: 4 })
    .insert_resource(AssetServerSettings {
        watch_for_changes: true,
        ..default()
    })
    .insert_resource(RoundSettings::default())
    .insert_resource(BoidSettings::default())
    .insert_resource(ClearColor(Color::BLACK))
    .add_plugins(DefaultPlugins)
    .add_plugin(InspectorPlugin::<BoidSettings>::new())
    .add_plugin(DebugLinesPlugin::default())
    .add_plugin(InputManagerPlugin::<PlayerActions>::default())
    .add_plugin(InputManagerPlugin::<GlobalActions>::default())
    .add_plugin(ui::UiAppPlugin)
    .add_plugin(ai::AiAppPlugin)
    .add_plugin(KbgpPlugin)
    .register_inspectable::<BoidNeighborsCaptureRange>()
    .register_inspectable::<BoidNeighborsSeparation>()
    .register_inspectable::<Camera2dFollow>()
    .register_inspectable::<BoidColor>()
    .register_inspectable::<Velocity>()
    .register_type::<BoidAveragedInputs>()
    .register_type::<ViewportRelative>()
    .add_state::<AppState>(AppState::default())
    .add_event::<GameEvent>()
    .add_startup_system(setup)
    .add_system_set(
        SystemSet::on_enter(AppState::LoadRound)
            .with_system(setup_game.after(despawn_game))
            .with_system(despawn_game),
    )
    .add_system_set(SystemSet::on_enter(AppState::Title).with_system(despawn_game))
    .add_system_to_stage(CoreStage::First, update_quad_tree)
    .add_system_to_stage(
        CoreStage::First,
        update_boid_neighbors.after(update_quad_tree),
    )
    .add_system_set(SystemSet::on_update(AppState::Playing).with_system(update_boid_transforms))
    .add_system_to_stage(CoreStage::Last, clear_inputs)
    .add_system(update_boid_color)
    .add_system(set_camera_viewports)
    .add_system(update_camera_follow_system)
    .add_system(update_camera_follow_many_system)
    .add_system(remove_camera_follow_target_on_capture)
    .add_system(camera_zoom)
    .add_system(leader_defeated)
    .add_system_set_to_stage(
        CoreStage::PreUpdate,
        SystemSet::new()
            .with_run_criteria(run_if_playing)
            .with_system(propagate_boid_color),
    )
    .add_system_to_stage(CoreStage::PostUpdate, leader_removed)
    .add_system_to_stage(CoreStage::PostUpdate, leader_added);

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
pub enum PlayerActions {
    Rotate,
    Direction,
    Throttle,
    Boost,
    CameraZoom,
}

/// Actions that any player can trigger
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum GlobalActions {
    ToggleMenu,
    ToggleBoidSettings,
    ToggleWorldInspector,
    ToggleFullScreen,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut inspector_windows: ResMut<InspectorWindows>,
    asset_server: ResMut<AssetServer>,
    round_settings: Res<RoundSettings>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("title.png"),
            transform: Transform::from_xyz(0.0, 100.0, 5.0).with_scale(Vec3::splat(0.3)),
            visibility: Visibility { is_visible: false },
            ..default()
        })
        .insert(Logo)
        .insert(Name::new("Logo"));
    let inspector_window_data = inspector_windows.window_data_mut::<BoidSettings>();
    inspector_window_data.visible = false;
    commands.spawn_bundle(ColorMesh2dBundle {
        mesh: meshes
            .add(Mesh::from(shape::Circle::new(
                round_settings.arena_radius + 2.0,
            )))
            .into(),
        material: materials.add(ColorMaterial::from(Color::WHITE)),
        ..default()
    });
    commands
        .spawn_bundle(ColorMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Circle::new(round_settings.arena_radius)))
                .into(),
            material: materials.add(ColorMaterial::from(asset_server.load("waves.png"))),
            transform: Transform::from_xyz(0.0, 0.0, 0.01),
            ..default()
        })
        .insert_bundle(InputManagerBundle {
            action_state: default(),
            input_map: {
                InputMap::<GlobalActions>::default()
                    .insert(KeyCode::Escape, GlobalActions::ToggleMenu)
                    .insert(KeyCode::Back, GlobalActions::ToggleMenu)
                    .insert(KeyCode::F1, GlobalActions::ToggleMenu)
                    .insert(KeyCode::F11, GlobalActions::ToggleFullScreen)
                    .insert(MouseButton::Right, GlobalActions::ToggleMenu)
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
                    .insert_chord(
                        [KeyCode::RAlt, KeyCode::Return],
                        GlobalActions::ToggleFullScreen,
                    )
                    .insert_chord(
                        [KeyCode::LAlt, KeyCode::Return],
                        GlobalActions::ToggleFullScreen,
                    )
                    .build()
            },
        });

    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical(SCENE_HEIGHT),
            ..Default::default()
        },
        camera: Camera {
            priority: 10,
            ..default()
        },
        ..Default::default()
    });
}

#[derive(Component, Debug, Copy, Clone)]
pub struct SceneRoot;

fn despawn_game(mut commands: Commands, scene_root: Query<Entity, With<SceneRoot>>) {
    if let Ok(root) = scene_root.get_single() {
        info!("Restarting");
        commands.entity(root).despawn_recursive();
    }
}

fn setup_game(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut app_state: ResMut<bevy::prelude::State<AppState>>,
    round_settings: Res<RoundSettings>,
) {
    // Spawn a root node to attach everything to so we can recursively delete everything
    // when reloading.
    let scene_root = commands
        .spawn()
        .insert(Name::new("Root"))
        .insert(SceneRoot)
        .insert_bundle(SpatialBundle::default())
        .id();

    let shared_camera = match round_settings.multiplayer_mode {
        MultiplayerMode::SharedScreen if round_settings.local_player_count() > 1 => {
            let camera = commands
                .spawn_bundle(Camera2dBundle {
                    projection: OrthographicProjection {
                        scaling_mode: ScalingMode::FixedVertical(SCENE_HEIGHT),
                        ..Default::default()
                    },
                    camera_2d: Camera2d {
                        clear_color: ClearColorConfig::Custom(Color::BLACK),
                    },
                    camera: Camera {
                        priority: 1000,
                        ..default()
                    },
                    ..Default::default()
                })
                .insert(Camera2dFollowMany)
                .insert(Name::new("Camera"))
                .id();
            commands.entity(scene_root).add_child(camera);
            Some(camera)
        }
        _ => None,
    };

    let rand = Rng::new();
    for x in 0..BOID_COUNT {
        let r = (round_settings.arena_radius - ARENA_PADDING) * rand.f32();
        let theta = rand.f32() * 2.0 * PI;
        let entity = commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("bird.png"),
                transform: Transform::from_xyz(r * theta.cos(), r * theta.sin(), 5.0)
                    .with_rotation(Quat::from_rotation_z(rand.f32_normalized() * PI * 2.0))
                    .with_scale(BOID_SCALE),
                ..Default::default()
            })
            .insert(Name::new(format!("Boid {x}")))
            .insert(BoidNeighborsSeparation::default())
            .insert(BoidNeighborsCaptureRange::default())
            .insert(ActionState::<PlayerActions>::default())
            .insert(BoidAveragedInputs::default())
            .insert(Boid::default())
            .insert(Velocity::default())
            .id();

        let viewports = PlayerViewports::new(
            round_settings.local_player_count() as u8,
            match &round_settings.multiplayer_mode {
                MultiplayerMode::SplitScreenVertical => ViewportLayoutPreference::Vertical,
                _ => ViewportLayoutPreference::Horizontal,
            },
            2.0,
        );
        match shared_camera {
            Some(_) => {
                if let Some(player_settings) = round_settings.players.get(x) {
                    if player_settings.player_type.is_local() {
                        commands.entity(entity).insert(CameraFollowTarget);
                    }
                }
            }
            None => {
                if let Some(viewport_id) = round_settings.player_viewport_id(x) {
                    let camera = commands
                        .spawn_bundle(Camera2dBundle {
                            projection: OrthographicProjection {
                                scaling_mode: ScalingMode::FixedVertical(SCENE_HEIGHT),
                                ..Default::default()
                            },
                            camera_2d: Camera2d {
                                clear_color: match viewport_id == 0 {
                                    true => ClearColorConfig::Custom(Color::BLACK),
                                    false => ClearColorConfig::None,
                                },
                            },
                            camera: Camera {
                                priority: (1000 + viewport_id) as isize,
                                ..default()
                            },
                            ..Default::default()
                        })
                        .insert(Camera2dFollow {
                            target: entity,
                            offset: Default::default(),
                        })
                        .insert(viewports.get(viewport_id))
                        .insert(Name::new(format!("Camera {viewport_id}")))
                        .id();
                    commands.entity(scene_root).add_child(camera);
                }
            }
        }

        if x < round_settings.players.len() {
            let player_settings = &round_settings.players[x];
            commands
                .entity(entity)
                .insert(player_settings.color)
                .insert(Leader);

            if let Some(input_map) = player_settings.player_type.input_map() {
                commands.entity(entity).insert(input_map);
            }

            if let PlayerType::Bot(selected_bot) = player_settings.player_type {
                selected_bot.insert(&mut commands.entity(entity));
            }
        }

        commands.entity(scene_root).add_child(entity);
    }
    if let Err(e) = app_state.overwrite_set(AppState::Playing) {
        error!("Error while starting game: {e}")
    } else {
        info!("App state transitioned to Playing")
    };
}

pub fn run_if_playing(app_state: Res<bevy::prelude::State<AppState>>) -> ShouldRun {
    if *app_state.current() == AppState::Playing {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}
