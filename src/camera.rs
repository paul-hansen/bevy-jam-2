use crate::{Camera2d, Query};
use bevy::math::Vec2Swizzles;
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

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
