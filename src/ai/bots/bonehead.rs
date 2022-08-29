use bevy::prelude::*;
use std::fmt::Formatter;

/// A bot that just goes forward
#[derive(Default, Component)]
pub struct BoneHead {}

impl std::fmt::Display for BoneHead {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bonehead")
    }
}
