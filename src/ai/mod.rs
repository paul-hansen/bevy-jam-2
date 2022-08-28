pub mod bots;
mod systems;

use crate::run_if_playing;
use bevy::prelude::*;
use systems::*;

pub struct AiAppPlugin;

impl Plugin for AiAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, calculate_cohesion_inputs)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                calculate_separation_inputs.after(calculate_cohesion_inputs),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                calculate_alignment_inputs.after(calculate_separation_inputs),
            )
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_run_criteria(run_if_playing)
                    .with_system(bots::speedy::update)
                    .with_system(bots::coward::update)
                    .with_system(bots::hunter::update),
            );
    }
}
