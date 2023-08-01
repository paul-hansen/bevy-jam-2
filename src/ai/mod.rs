pub mod bots;
mod systems;

use crate::AppState;
use bevy::prelude::*;
use systems::*;

pub struct AiAppPlugin;

impl Plugin for AiAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                calculate_cohesion_inputs,
                calculate_alignment_inputs.after(calculate_separation_inputs),
                calculate_separation_inputs.after(calculate_cohesion_inputs),
            )
                .in_base_set(CoreSet::PreUpdate),
        )
        .add_systems(
            (
                bots::speedy::update,
                bots::coward::update,
                bots::hunter::update,
            )
                .in_base_set(CoreSet::PreUpdate)
                .distributive_run_if(in_state(AppState::Playing)),
        );
    }
}
