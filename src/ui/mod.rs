mod components;
mod style;
mod systems;

pub use components::*;

use crate::AppState;
use bevy::prelude::*;
use systems::*;

pub struct UiAppPlugin;

impl Plugin for UiAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(set_ui_theme)
            .add_startup_system(lock_mouse)
            .add_system_set(SystemSet::on_update(AppState::PauseMenu).with_system(draw_pause_menu))
            .add_system_set(SystemSet::on_enter(AppState::PauseMenu).with_system(unlock_mouse))
            .add_system_set(SystemSet::on_exit(AppState::PauseMenu).with_system(lock_mouse))
            .add_system_set(SystemSet::on_update(AppState::GameOver).with_system(draw_game_over))
            .add_system_set(SystemSet::on_enter(AppState::GameOver).with_system(unlock_mouse))
            .add_system_set(SystemSet::on_exit(AppState::GameOver).with_system(lock_mouse))
            .add_system_set(SystemSet::on_update(AppState::Title).with_system(draw_title))
            .add_system_set(SystemSet::on_enter(AppState::Title).with_system(unlock_mouse))
            .add_system_set(SystemSet::on_exit(AppState::Title).with_system(lock_mouse))
            .add_system_set(SystemSet::on_enter(AppState::Title).with_system(on_title_enter))
            .add_system_set(SystemSet::on_exit(AppState::Title).with_system(on_title_exit))
            .add_system(toggle_pause_menu)
            .add_system(toggle_boid_settings)
            .add_system(toggle_world_inspector);
    }
}
