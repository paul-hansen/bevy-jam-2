mod ai;
mod boids;
mod camera;
mod inspector;
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
use crate::inspector::InspectorPlugin;
use crate::math::how_much_right_or_left;
use crate::round::{MultiplayerMode, PlayerType, RoundSettings};
use crate::ui::Logo;
use crate::viewports::{
    set_camera_viewports, PlayerViewports, ViewportLayoutPreference, ViewportRelative,
};
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::WindowMode;
use bevy_egui_kbgp::KbgpPlugin;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use leafwing_input_manager::prelude::*;
use std::f32::consts::PI;
use turborand::prelude::*;

const SCENE_HEIGHT: f32 = 500.0;
const BOID_COUNT: usize = 400;
const ARENA_PADDING: f32 = 100.0;
const BOID_SCALE: Vec3 = Vec3::splat(0.01);
const LEADER_SCALE: Vec3 = Vec3::splat(0.014);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub enum AppState {
    #[default]
    Title,
    LoadRound,
    GameOver,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Reflect, Resource)]
pub struct Winner {
    pub color: BoidColor,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa::Sample8)
        .insert_resource(RoundSettings::default())
        .insert_resource(BoidSettings::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        mode: WindowMode::Windowed,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_state::<AppState>()
        .add_plugin(InspectorPlugin)
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(InputManagerPlugin::<PlayerActions>::default())
        .add_plugin(InputManagerPlugin::<GlobalActions>::default())
        .add_plugin(ui::UiAppPlugin)
        .add_plugin(ai::AiAppPlugin)
        .add_plugin(KbgpPlugin)
        .register_type::<BoidNeighborsCaptureRange>()
        .register_type::<BoidNeighborsSeparation>()
        .register_type::<Camera2dFollow>()
        .register_type::<BoidColor>()
        .register_type::<Velocity>()
        .register_type::<BoidAveragedInputs>()
        .register_type::<ViewportRelative>()
        .register_type::<BoidSettings>()
        .add_event::<GameEvent>()
        .add_startup_system(setup)
        .add_systems(
            (setup_game.after(despawn_game), despawn_game)
                .in_schedule(OnEnter(AppState::LoadRound)),
        )
        .add_system(despawn_game.in_schedule(OnEnter(AppState::Title)))
        .add_systems(
            (
                update_quad_tree,
                update_boid_neighbors.after(update_quad_tree),
            )
                .in_base_set(CoreSet::First),
        )
        .add_system(update_boid_transforms.in_set(OnUpdate(AppState::Playing)))
        .add_system(clear_inputs.in_base_set(CoreSet::Last))
        .add_system(update_boid_color)
        .add_system(set_camera_viewports)
        .add_system(update_camera_follow_system)
        .add_system(update_camera_follow_many_system)
        .add_system(remove_camera_follow_target_on_capture)
        .add_system(camera_zoom)
        .add_system(leader_defeated)
        .add_system(
            propagate_boid_color
                .run_if(in_state(AppState::Playing))
                .in_base_set(CoreSet::PreUpdate),
        )
        .add_systems((leader_removed, leader_added).in_base_set(CoreSet::PostUpdate));

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
    asset_server: ResMut<AssetServer>,
    round_settings: Res<RoundSettings>,
) {
    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("title.png"),
            transform: Transform::from_xyz(0.0, 100.0, 5.0).with_scale(Vec3::splat(0.3)),
            visibility: Visibility::Hidden,
            ..default()
        })
        .insert(Logo)
        .insert(Name::new("Logo"));
    commands.spawn(ColorMesh2dBundle {
        mesh: meshes
            .add(Mesh::from(shape::Circle::new(
                round_settings.arena_radius + 2.0,
            )))
            .into(),
        material: materials.add(ColorMaterial::from(Color::WHITE)),
        ..default()
    });
    commands
        .spawn(ColorMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Circle::new(round_settings.arena_radius)))
                .into(),
            material: materials.add(ColorMaterial::from(asset_server.load("waves.png"))),
            transform: Transform::from_xyz(0.0, 0.0, 0.01),
            ..default()
        })
        .insert(InputManagerBundle {
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

    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical(SCENE_HEIGHT),
            ..Default::default()
        },
        camera: Camera {
            order: 10,
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
    mut app_state: ResMut<NextState<AppState>>,
    round_settings: Res<RoundSettings>,
) {
    // Spawn a root node to attach everything to so we can recursively delete everything
    // when reloading.
    let scene_root = commands
        .spawn((Name::new("Root"), SceneRoot, SpatialBundle::default()))
        .id();

    let shared_camera = match round_settings.multiplayer_mode {
        MultiplayerMode::SharedScreen if round_settings.local_player_count() > 1 => {
            let camera = commands
                .spawn(Camera2dBundle {
                    projection: OrthographicProjection {
                        scaling_mode: ScalingMode::FixedVertical(SCENE_HEIGHT),
                        ..Default::default()
                    },
                    camera_2d: Camera2d {
                        clear_color: ClearColorConfig::Custom(Color::BLACK),
                    },
                    camera: Camera {
                        order: 1000,
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
            .spawn(SpriteBundle {
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
                        .spawn(Camera2dBundle {
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
                                order: (1000 + viewport_id) as isize,
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
    app_state.set(AppState::Playing);
}
