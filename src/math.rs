use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

/// Returns a value between -1.0 and 1.0 based on how left or right the target is from the transform.
/// Does not take into account how much forward or back the target is.
pub fn how_much_right_or_left(transform: &Transform, target: Vec2) -> f32 {
    let direction_to_target = (target - transform.translation.truncate()).normalize();

    // The dot product when used with normalized vectors tells you how parallel
    // a vector is to another.
    // Negative values means it is facing the opposite way,
    // so if we use the right facing vector, the result will be -1.0 to 1.0 based on
    // how much to the right the target is from the current boid.
    transform.right().truncate().dot(direction_to_target)
}

/// Returns a value between -1.0 and 1.0 based on which direction the transform must turn to turn
/// towards the target.
///
/// A value of 1.0 or -1.0 means the target is directly in front.
///
/// A value of 0.0 means the target is directly behind.
///
/// A value of -0.5 means the target is directly to the right.
pub fn direction_to_turn_away_from_target(transform: &Transform, target: Vec2) -> f32 {
    let angle_to_target = vec2_to_angle(target - transform.translation.truncate());
    let transform_z_angle = vec2_to_angle(transform.down().truncate());
    -angle_to(transform_z_angle, angle_to_target) / PI
}

pub trait Average<A, B>
where
    Self: Iterator<Item = A>,
{
    fn avg(self) -> B;
}

impl<'a, I: Iterator<Item = &'a Vec2>> Average<&'a Vec2, Vec2> for I {
    fn avg(self) -> Vec2 {
        let mut count = 0;
        let sum = self.fold(Vec2::default(), |a, b| {
            count += 1;
            a + *b
        });
        sum / count as f32
    }
}

impl<I: Iterator<Item = Vec2>> Average<Vec2, Vec2> for I {
    fn avg(self) -> Vec2 {
        let mut count = 0;
        let sum = self.fold(Vec2::default(), |a, b| {
            count += 1;
            a + b
        });
        sum / count as f32
    }
}

impl<I: Iterator<Item = f32>> Average<f32, f32> for I {
    fn avg(self) -> f32 {
        let mut count = 0;
        let sum = self.fold(f32::default(), |a, b| {
            count += 1;
            a + b
        });
        sum / count as f32
    }
}

/// returns the shortest rotation required to reach rotation b from rotation a in radians.
pub fn angle_to(a: f32, b: f32) -> f32 {
    wrap_f32(b - a, -PI, PI)
}

/// Returns `a` wrapped to the range 0 to max.
pub fn wrap_f32_zero(a: f32, max: f32) -> f32 {
    (max + (a % max)) % max
}

/// Returns `a` wrapped to the range min to max.
pub fn wrap_f32(a: f32, min: f32, max: f32) -> f32 {
    min + wrap_f32_zero(a - min, max - min)
}

pub fn vec2_to_angle(vector: Vec2) -> f32 {
    wrap_f32_zero(vector.y.atan2(vector.x), TAU)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_distance_between_two_angles() {
        assert_relative_eq!(
            angle_to(0.0f32.to_radians(), 10.0f32.to_radians()).to_degrees(),
            10.0,
            max_relative = 0.001
        );

        assert_relative_eq!(
            angle_to(10.0f32.to_radians(), 0.0f32.to_radians()).to_degrees(),
            -10.0,
            max_relative = 0.001
        );

        assert_relative_eq!(
            angle_to(350.0f32.to_radians(), 0.0f32.to_radians()).to_degrees(),
            10.0,
            max_relative = 0.001
        );

        assert_relative_eq!(
            angle_to(-350.0f32.to_radians(), 0.0f32.to_radians()).to_degrees(),
            -10.0,
            max_relative = 0.001
        );

        assert_relative_eq!(
            angle_to(0.0f32.to_radians(), 400.0f32.to_radians()).to_degrees(),
            40.0,
            max_relative = 0.001
        );
    }

    #[test]
    fn test_vec2_to_angle() {
        assert_relative_eq!(
            vec2_to_angle(Vec2::splat(1.0)).to_degrees(),
            45.0,
            max_relative = 0.001
        );
        assert_relative_eq!(
            vec2_to_angle(-Vec2::splat(1.0)).to_degrees(),
            225.0,
            max_relative = 0.001
        );

        assert_relative_eq!(
            vec2_to_angle(-Vec2::splat(1.0) * 10.0).to_degrees(),
            225.0,
            max_relative = 0.001
        );
    }
    #[test]
    fn test_wrap_f32() {
        assert_relative_eq!(wrap_f32(105.0, 50.0, 75.0), 55.0, max_relative = 0.001);
    }

    #[test]
    fn test_how_much_right_or_left_positive() {
        assert_relative_eq!(
            how_much_right_or_left(&Transform::default(), Vec2::X),
            1.0,
            max_relative = 0.001
        );
    }

    #[test]
    fn test_how_much_right_or_left_neg() {
        assert_relative_eq!(
            how_much_right_or_left(&Transform::default(), Vec2::NEG_X),
            -1.0,
            max_relative = 0.001
        );
    }

    #[test]
    fn test_direction_to_turn_away_from_target() {
        assert_relative_eq!(
            direction_to_turn_away_from_target(&Transform::default(), Vec2::NEG_Y).abs(),
            0.0,
            max_relative = 0.001
        );

        assert_relative_eq!(
            direction_to_turn_away_from_target(&Transform::default(), Vec2::Y).abs(),
            1.0,
            max_relative = 0.001
        );

        assert_relative_eq!(
            direction_to_turn_away_from_target(&Transform::default(), Vec2::NEG_X),
            0.5,
            max_relative = 0.001
        );

        assert_relative_eq!(
            direction_to_turn_away_from_target(&Transform::default(), Vec2::X),
            -0.5,
            max_relative = 0.001
        );
    }
}
