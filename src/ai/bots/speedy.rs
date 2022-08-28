use crate::{BoidAveragedInputs, Leader};
use bevy::prelude::*;
use std::fmt::Formatter;

/// A bot that always boosts
#[derive(Default, Component)]
pub struct Speedy {}

impl std::fmt::Display for Speedy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Speedy")
    }
}

pub fn update(mut query: Query<&mut BoidAveragedInputs, (With<Speedy>, With<Leader>)>) {
    for mut inputs in query.iter_mut() {
        inputs.add_speed(1.0);
    }
}
