use crate::math::Average;
use crate::{Camera2d, Leader, PlayerActions, Query, ScalingMode, SCENE_HEIGHT};
use bevy::math::Vec2Swizzles;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use std::time::Duration;

pub fn update_camera_follow_system(
    mut cameras: Query<(&Camera2dFollow, &mut Transform), With<Camera2d>>,
    transforms: Query<&GlobalTransform>,
) {
    for (camera_follow, mut transform) in cameras.iter_mut() {
        if let Ok(target_transform) = transforms.get(camera_follow.target) {
            let mut translation = target_transform.translation() + camera_follow.offset.xyy();
            // Keep the z position of the camera.
            translation.z = transform.translation.z;
            transform.translation = translation;
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Camera2dFollow {
    pub target: Entity,
    pub offset: Vec2,
}

impl FromWorld for Camera2dFollow {
    fn from_world(world: &mut World) -> Self {
        Self {
            target: world.entities().reserve_entity(),
            offset: default(),
        }
    }
}

pub fn camera_zoom(
    mut query: Query<(&Camera2dFollow, &mut OrthographicProjection)>,
    player_query: Query<(Entity, &ActionState<PlayerActions>)>,
    time: Res<Time>,
) {
    for (entity, action_state) in player_query.iter() {
        let amount = match action_state.just_pressed(PlayerActions::CameraZoom) {
            true => 50.0,
            false => match action_state.current_duration(PlayerActions::CameraZoom)
                > Duration::from_secs_f32(0.25)
            {
                true => 320.0 * time.delta_seconds(),
                false => 0.0,
            },
        };
        for (camera_follow, mut projection) in query.iter_mut() {
            if camera_follow.target == entity {
                if let ScalingMode::FixedVertical(x) = projection.scaling_mode {
                    if let Some(axis_pair) =
                        action_state.clamped_axis_pair(PlayerActions::CameraZoom)
                    {
                        projection.scaling_mode = ScalingMode::FixedVertical(
                            (x - axis_pair.y() * amount).clamp(200.0, 1200.0),
                        );
                    }
                }
            }
        }
    }
}

// Add to the camera
#[derive(Component)]
pub struct Camera2dFollowMany;

// Add to an entity to be followed by the Camera2dFollowMany camera
#[derive(Component)]
pub struct CameraFollowTarget;

pub fn update_camera_follow_many_system(
    mut cameras: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2dFollowMany>>,
    targets: Query<&GlobalTransform, With<CameraFollowTarget>>,
) {
    for (mut transform, mut projection) in cameras.iter_mut() {
        let targets_center: Vec2 = targets.iter().map(|t| t.translation().truncate()).avg();
        let max_distance: Option<f32> = targets
            .iter_combinations::<2>()
            .map(|[a, b]| a.translation().distance_squared(b.translation()))
            .max_by(|a, b| a.total_cmp(b));
        projection.scaling_mode = ScalingMode::FixedVertical(
            max_distance
                .map(|x| x.sqrt() + 500.0)
                .unwrap_or(SCENE_HEIGHT),
        );
        transform.translation = targets_center.extend(transform.translation.z);
    }
}

pub fn remove_camera_follow_target_on_capture(
    mut commands: Commands,
    query: Query<Entity, (Without<Leader>, With<CameraFollowTarget>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<CameraFollowTarget>();
    }
}
