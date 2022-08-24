mod systems;

use bevy::prelude::*;
use systems::*;

pub struct UiAppPlugin;

impl Plugin for UiAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(toggle_boid_settings)
            .add_system(toggle_world_inspector);
    }
}
