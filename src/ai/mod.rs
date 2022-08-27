mod systems;

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
            );
    }
}
