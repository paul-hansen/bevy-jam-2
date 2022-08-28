use bevy::prelude::*;
use std::fmt::Formatter;

/// A bot that just goes forward
#[derive(Default, Component)]
pub struct BrainDead {}

impl std::fmt::Display for BrainDead {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Brain Dead")
    }
}
