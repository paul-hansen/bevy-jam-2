mod components;
mod style;
mod systems;

pub use components::*;

use crate::AppState;
use bevy::prelude::*;
use systems::*;

pub struct UiAppPlugin;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub enum UiState {
    #[default]
    Title,
    CustomGameMenu,
    PauseMenu,
    SettingsMenu,
    Hidden,
}

impl Plugin for UiAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiEvent>()
            .add_state::<UiState>()
            .add_startup_system(set_ui_theme)
            .add_startup_system(lock_mouse);
        // Settings does not need to lock/unlock mouse since it will be opened from another menu
        app.add_system(draw_settings.in_set(OnUpdate(UiState::SettingsMenu)));
        app.add_system(draw_pause_menu.in_set(OnUpdate(UiState::PauseMenu)));
        app.add_system(unlock_mouse.in_schedule(OnEnter(UiState::PauseMenu)));
        app.add_system(lock_mouse.in_schedule(OnEnter(UiState::Hidden)));
        app.add_system(draw_game_over.in_set(OnUpdate(AppState::GameOver)));
        app.add_system(unlock_mouse.in_schedule(OnEnter(AppState::GameOver)));
        app.add_system(lock_mouse.in_schedule(OnExit(AppState::GameOver)));
        app.add_system(draw_title.in_set(OnUpdate(AppState::Title)));
        app.add_system(unlock_mouse.in_schedule(OnEnter(AppState::Title)));
        app.add_system(lock_mouse.in_schedule(OnExit(AppState::Title)));
        app.add_system(on_title_enter.in_schedule(OnEnter(AppState::Title)));
        app.add_system(on_title_exit.in_schedule(OnExit(AppState::Title)));
        app.add_system(draw_round_settings.in_set(OnUpdate(UiState::CustomGameMenu)));
        app.add_system(unlock_mouse.in_schedule(OnEnter(UiState::CustomGameMenu)));
        app.add_system(lock_mouse.in_schedule(OnExit(UiState::CustomGameMenu)));
        app.add_system(toggle_pause_hotkey);
        app.add_system(on_focused);
        app.add_system(on_click);
        app.add_system(toggle_fullscreen);
        app.add_system(handle_ui_events.in_base_set(CoreSet::PostUpdate));
        app.add_system(hide_ui.in_schedule(OnEnter(AppState::Playing)));
        app.add_system(show_pause_menu.in_schedule(OnEnter(AppState::Paused)));
        app.insert_resource(UiData::default());
    }
}
