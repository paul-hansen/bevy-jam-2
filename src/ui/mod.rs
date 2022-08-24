mod style;
mod systems;

use crate::AppState;
use bevy::prelude::*;
use systems::*;

pub struct UiAppPlugin;

impl Plugin for UiAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(set_ui_theme)
            .add_system_set(SystemSet::on_update(AppState::PauseMenu).with_system(draw_pause_menu))
            .add_system(toggle_pause_menu)
            .add_system(toggle_boid_settings)
            .add_system(toggle_world_inspector);
    }
}
