use crate::{Actions, Camera2d, Query, ScalingMode};
use bevy::math::Vec2Swizzles;
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
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

#[derive(Component, Inspectable)]
pub struct Camera2dFollow {
    pub target: Entity,
    pub offset: Vec2,
}

pub fn camera_zoom(
    mut query: Query<(&Camera2dFollow, &mut OrthographicProjection)>,
    player_query: Query<(Entity, &ActionState<Actions>)>,
    time: Res<Time>,
) {
    for (entity, action_state) in player_query.iter() {
        let amount = match action_state.just_pressed(Actions::CameraZoom) {
            true => 50.0,
            false => match action_state.current_duration(Actions::CameraZoom)
                > Duration::from_secs_f32(0.25)
            {
                true => 320.0 * time.delta_seconds(),
                false => 0.0,
            },
        };
        for (camera_follow, mut projection) in query.iter_mut() {
            if camera_follow.target == entity {
                if let ScalingMode::FixedVertical(x) = projection.scaling_mode {
                    if let Some(axis_pair) = action_state.clamped_axis_pair(Actions::CameraZoom) {
                        projection.scaling_mode = ScalingMode::FixedVertical(
                            (x - axis_pair.y() * amount).clamp(200.0, 1200.0),
                        );
                    }
                }
            }
        }
    }
}
