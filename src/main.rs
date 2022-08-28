mod ai;
mod boids;
mod camera;
mod math;
mod ui;

use crate::ai::bots::Bot;
use crate::boids::{
    clear_inputs, leader_added, leader_defeated, leader_removed, propagate_boid_color,
    update_boid_color, update_boid_neighbors, update_boid_transforms, Boid, BoidAveragedInputs,
    BoidColor, BoidNeighborsCaptureRange, BoidNeighborsSeparation, BoidSettings, GameEvent, Leader,
    Velocity,
};
use crate::camera::{camera_zoom, update_camera_follow_system, Camera2dFollow};
use crate::math::how_much_right_or_left;
use crate::ui::Logo;
use bevy::asset::AssetServerSettings;
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::WindowMode;
use bevy_egui_kbgp::KbgpPlugin;
use bevy_inspector_egui::plugin::InspectorWindows;
use bevy_inspector_egui::{InspectorPlugin, RegisterInspectable};
use bevy_prototype_debug_lines::DebugLinesPlugin;
use leafwing_input_manager::buttonlike::MouseMotionDirection;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::user_input::InputKind;
use std::f32::consts::PI;
use turborand::prelude::*;

const SCENE_HEIGHT: f32 = 500.0;
const BOID_COUNT: usize = 400;
const ARENA_RADIUS: f32 = 1200.0;
const ARENA_PADDING: f32 = 100.0;
const BOID_SCALE: Vec3 = Vec3::splat(0.01);
const LEADER_SCALE: Vec3 = Vec3::splat(0.014);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Title,
    Setup,
    PauseMenu,
    GameOver,
    Playing,
}

#[derive(Debug, Copy, Clone)]
pub enum PlayerType {
    AnyDevice,
    Wasd,
    Arrowkeys,
    Mouse,
    GamePad(Option<Gamepad>),
    Bot(Bot),
}

impl PlayerType {
    pub fn is_local(&self) -> bool {
        !matches!(self, Self::Bot(_))
    }

    pub fn input_map(&self) -> Option<InputMap<PlayerActions>> {
        match self {
            PlayerType::AnyDevice => Some(
                PlayerType::Wasd
                    .input_map()
                    .unwrap()
                    .merge(&PlayerType::Arrowkeys.input_map().unwrap())
                    .merge(&PlayerType::Mouse.input_map().unwrap())
                    .merge(&PlayerType::GamePad(None).input_map().unwrap())
                    .build(),
            ),
            PlayerType::Wasd => Some(
                InputMap::<PlayerActions>::default()
                    .insert(VirtualDPad::wasd(), PlayerActions::Direction)
                    .insert(
                        VirtualDPad {
                            up: KeyCode::Equals.into(),
                            down: KeyCode::Minus.into(),
                            left: KeyCode::Minus.into(),
                            right: KeyCode::Equals.into(),
                        },
                        PlayerActions::CameraZoom,
                    )
                    .insert(
                        VirtualDPad {
                            up: KeyCode::R.into(),
                            down: KeyCode::F.into(),
                            left: KeyCode::R.into(),
                            right: KeyCode::F.into(),
                        },
                        PlayerActions::CameraZoom,
                    )
                    .insert(KeyCode::Space, PlayerActions::Boost)
                    .insert(KeyCode::LShift, PlayerActions::Boost)
                    .build(),
            ),
            PlayerType::Arrowkeys => Some(
                InputMap::<PlayerActions>::default()
                    .insert(VirtualDPad::arrow_keys(), PlayerActions::Rotate)
                    .insert(
                        VirtualDPad {
                            up: KeyCode::NumpadAdd.into(),
                            down: KeyCode::NumpadSubtract.into(),
                            left: KeyCode::NumpadSubtract.into(),
                            right: KeyCode::NumpadAdd.into(),
                        },
                        PlayerActions::CameraZoom,
                    )
                    .insert(KeyCode::Up, PlayerActions::Boost)
                    .build(),
            ),
            PlayerType::Mouse => Some(
                InputMap::<PlayerActions>::default()
                    .insert(
                        VirtualDPad {
                            up: InputKind::MouseMotion(MouseMotionDirection::Down),
                            down: InputKind::MouseMotion(MouseMotionDirection::Up),
                            left: InputKind::MouseMotion(MouseMotionDirection::Left),
                            right: InputKind::MouseMotion(MouseMotionDirection::Right),
                        },
                        PlayerActions::Direction,
                    )
                    .insert(VirtualDPad::mouse_wheel(), PlayerActions::CameraZoom)
                    .insert(MouseButton::Left, PlayerActions::Boost)
                    .build(),
            ),
            PlayerType::GamePad(_) => Some(
                InputMap::<PlayerActions>::default()
                    .insert(DualAxis::left_stick(), PlayerActions::Direction)
                    .insert(VirtualDPad::dpad(), PlayerActions::CameraZoom)
                    .insert(GamepadButtonType::South, PlayerActions::Boost)
                    .insert(GamepadButtonType::RightTrigger, PlayerActions::Boost)
                    .build(),
            ),
            PlayerType::Bot(_) => None,
        }
    }
}

pub struct PlayerSettings {
    pub player_type: PlayerType,
    pub color: BoidColor,
}

pub struct MatchSettings {
    pub players: Vec<PlayerSettings>,
}

impl Default for MatchSettings {
    fn default() -> Self {
        Self {
            players: vec![
                PlayerSettings {
                    player_type: PlayerType::AnyDevice,
                    color: BoidColor::Red,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::BrainDead),
                    color: BoidColor::Green,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::BrainDead),
                    color: BoidColor::Blue,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::Coward),
                    color: BoidColor::Yellow,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::Speedy),
                    color: BoidColor::Purple,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::Hunter),
                    color: BoidColor::Orange,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::BrainDead),
                    color: BoidColor::Pink,
                },
                PlayerSettings {
                    player_type: PlayerType::Bot(Bot::BrainDead),
                    color: BoidColor::Cyan,
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Winner {
    pub color: BoidColor,
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
    .insert_resource(MatchSettings::default())
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
    .add_state::<AppState>(AppState::Title)
    .add_event::<GameEvent>()
    .add_startup_system(setup)
    .add_system_set(
        SystemSet::on_enter(AppState::Setup)
            .with_system(setup_game.after(despawn_game))
            .with_system(despawn_game),
    )
    .add_system_set(SystemSet::on_enter(AppState::Title).with_system(despawn_game))
    .add_system_to_stage(CoreStage::First, update_boid_neighbors)
    .add_system_set(SystemSet::on_update(AppState::Playing).with_system(update_boid_transforms))
    .add_system_to_stage(CoreStage::Last, clear_inputs)
    .add_system(update_boid_color)
    .add_system(update_camera_follow_system)
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
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut inspector_windows: ResMut<InspectorWindows>,
    asset_server: ResMut<AssetServer>,
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
            .add(Mesh::from(shape::Circle::new(ARENA_RADIUS + 2.0)))
            .into(),
        material: materials.add(ColorMaterial::from(Color::WHITE)),
        ..default()
    });
    commands
        .spawn_bundle(ColorMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Circle::new(ARENA_RADIUS)))
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

    match_settings: Res<MatchSettings>,
) {
    // Spawn a root node to attach everything to so we can recursively delete everything
    // when reloading.

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

        if x < match_settings.players.len() {
            let player_settings = &match_settings.players[x];
            commands
                .entity(entity)
                .insert(player_settings.color)
                .insert(Leader);
            if player_settings.player_type.is_local() {
                let camera = commands
                    .spawn_bundle(Camera2dBundle {
                        projection: OrthographicProjection {
                            scaling_mode: ScalingMode::FixedVertical(SCENE_HEIGHT),
                            ..Default::default()
                        },
                        camera: Camera {
                            priority: 1000,
                            ..default()
                        },
                        ..Default::default()
                    })
                    .insert(Camera2dFollow {
                        target: entity,
                        offset: Default::default(),
                    })
                    .id();
                commands.entity(scene_root).add_child(camera);
            }

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
