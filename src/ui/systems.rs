use crate::{AppState, BoidSettings, GlobalActions};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::egui::{Align, Align2, Layout};
use bevy_egui::{egui, EguiContext};
use bevy_egui_kbgp::KbgpEguiResponseExt;
use bevy_inspector_egui::plugin::InspectorWindows;
use bevy_inspector_egui::WorldInspectorParams;
use egui::vec2;
use leafwing_input_manager::prelude::*;

pub fn draw_pause_menu(
    mut egui_context: ResMut<EguiContext>,
    mut exit: EventWriter<AppExit>,
    mut app_state: ResMut<State<AppState>>,
) {
    egui::Window::new("Game Paused")
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 120.0))
        .resizable(false)
        .collapsible(false)
        .min_width(200.0)
        .show(egui_context.ctx_mut(), |ui| {
            ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                if app_state.inactives().contains(&AppState::Playing)
                    && ui
                        .button("Resume")
                        .kbgp_navigation()
                        .kbgp_initial_focus()
                        .clicked()
                {
                    if let Err(e) = app_state.pop() {
                        error!("Error resuming game: {e}");
                    };
                }

                if ui.button("Restart").kbgp_navigation().clicked() {
                    if let Err(e) = app_state.set(AppState::Setup) {
                        error!("Error when restarting game: {e}");
                    };
                }
                if ui.button("Exit Game").kbgp_navigation().clicked() {
                    exit.send(AppExit);
                };
            });
        });
}

/// Handles toggling the Menu app state when the toggle menu button is pressed
pub fn toggle_pause_menu(
    action_state: Query<&ActionState<GlobalActions>>,
    mut app_state: ResMut<State<AppState>>,
) {
    let action_state = action_state.single();
    if action_state.just_pressed(GlobalActions::ToggleMenu) {
        match app_state.current() {
            AppState::PauseMenu => {
                if let Err(e) = app_state.set(AppState::Playing) {
                    error!("Error while trying to close the menu: {e}");
                } else {
                    info!("Transitioning to {:?}", AppState::Playing)
                }
            }
            AppState::Playing => {
                if let Err(e) = app_state.push(AppState::PauseMenu) {
                    error!("Error while trying to open the menu: {e}");
                } else {
                    info!("Transitioning to {:?}", AppState::PauseMenu)
                }
            }
            _ => {}
        }
    }
}

pub fn toggle_boid_settings(
    mut inspector_windows: ResMut<InspectorWindows>,
    action_state: Query<&ActionState<GlobalActions>>,
) {
    let action_state = action_state.single();
    if action_state.just_released(GlobalActions::ToggleBoidSettings) {
        let inspector_window_data = inspector_windows.window_data_mut::<BoidSettings>();
        inspector_window_data.visible = !inspector_window_data.visible;
    }
}

pub fn toggle_world_inspector(
    action_state: Query<&ActionState<GlobalActions>>,
    mut world_inspector_params: ResMut<WorldInspectorParams>,
) {
    let action_state = action_state.single();
    if action_state.just_released(GlobalActions::ToggleWorldInspector) {
        world_inspector_params.enabled = !world_inspector_params.enabled;
    }
}
