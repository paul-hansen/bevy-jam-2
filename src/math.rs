use bevy::prelude::*;

pub fn how_much_right_or_left(transform: &Transform, target: &Vec2) -> f32 {
    let direction_to_target = (*target - transform.translation.truncate()).normalize();

    // The dot product when used with normalized vectors tells you how parallel
    // a vector is to another.
    // Negative values means it is facing the opposite way,
    // so if we use the right facing vector, the result will be -1.0 to 1.0 based on
    // how much to the right the target is from the current boid.
    transform.right().truncate().dot(direction_to_target)
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
