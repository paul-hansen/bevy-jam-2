use crate::ui::style::get_style;
use crate::ui::Logo;
use crate::{AppState, BoidSettings, GlobalActions, Winner};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::egui::Align2;
use bevy_egui::{egui, EguiContext};
use bevy_egui_kbgp::KbgpEguiResponseExt;
use bevy_inspector_egui::plugin::InspectorWindows;
use bevy_inspector_egui::WorldInspectorParams;
use egui::vec2;
use leafwing_input_manager::prelude::*;

pub fn set_ui_theme(mut ctx: ResMut<EguiContext>) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "sans".to_owned(),
        egui::FontData::from_static(include_bytes!("../../fonts/JosefinSans-Medium.ttf")),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "sans".to_owned());
    ctx.ctx_mut().set_fonts(fonts);
    ctx.ctx_mut().set_style(get_style());
}

pub fn draw_pause_menu(
    mut egui_context: ResMut<EguiContext>,
    mut app_state: ResMut<State<AppState>>,
) {
    egui::Window::new("Game Paused")
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 120.0))
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| ui.heading("Game Paused"));
            ui.separator();
            ui.set_width(220.0);
            ui.vertical_centered_justified(|ui| {
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
                if ui.button("Return to Title").kbgp_navigation().clicked() {
                    if let Err(e) = app_state.set(AppState::Title) {
                        error!("Error when returning to title: {e}");
                    };
                }
            });
        });
}

pub fn draw_title(
    mut egui_context: ResMut<EguiContext>,
    mut exit: EventWriter<AppExit>,
    mut app_state: ResMut<State<AppState>>,
) {
    egui::Window::new("Flock Fusion")
        .title_bar(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 120.0))
        .resizable(false)
        .collapsible(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.set_width(200.0);
            ui.vertical_centered_justified(|ui| {
                if ui
                    .button("Start game")
                    .kbgp_navigation()
                    .kbgp_initial_focus()
                    .clicked()
                {
                    if let Err(e) = app_state.set(AppState::Setup) {
                        error!("Error when restarting game: {e}");
                    };
                }
                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Exit Game").kbgp_navigation().clicked() {
                    exit.send(AppExit);
                };
            });
        });
}

pub fn on_title_enter(mut query: Query<&mut Visibility, With<Logo>>) {
    query.single_mut().is_visible = true;
}

pub fn on_title_exit(mut query: Query<&mut Visibility, With<Logo>>) {
    query.single_mut().is_visible = false;
}

pub fn draw_game_over(
    mut egui_context: ResMut<EguiContext>,
    mut app_state: ResMut<State<AppState>>,
    winner: Option<Res<Winner>>,
) {
    let title = match winner {
        None => "Tie!".to_string(),
        Some(winner) => format!("{:?} Won!", winner.color),
    };
    egui::Window::new("Winner")
        .title_bar(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 120.0))
        .resizable(false)
        .collapsible(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| ui.heading(title));
            ui.separator();
            ui.set_width(220.0);
            ui.set_width(200.0);
            ui.vertical_centered_justified(|ui| {
                if ui
                    .button("Restart")
                    .kbgp_navigation()
                    .kbgp_initial_focus()
                    .clicked()
                {
                    if let Err(e) = app_state.set(AppState::Setup) {
                        error!("Error when restarting game: {e}");
                    };
                }

                if ui.button("Return to Title").kbgp_navigation().clicked() {
                    if let Err(e) = app_state.set(AppState::Title) {
                        error!("Error when returning to title: {e}");
                    };
                }
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

pub fn lock_mouse(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_cursor_lock_mode(true);
    window.set_cursor_visibility(false);
}

pub fn unlock_mouse(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_cursor_lock_mode(false);
    window.set_cursor_visibility(true);
}

pub fn toggle_boid_settings(
    mut inspector_windows: ResMut<InspectorWindows>,
    action_state: Query<&ActionState<GlobalActions>>,
    mut windows: ResMut<Windows>,
) {
    let action_state = action_state.single();
    if action_state.just_released(GlobalActions::ToggleBoidSettings) {
        let inspector_window_data = inspector_windows.window_data_mut::<BoidSettings>();
        inspector_window_data.visible = !inspector_window_data.visible;
        let window = windows.get_primary_mut().unwrap();
        window.set_cursor_lock_mode(!inspector_window_data.visible);
        window.set_cursor_visibility(inspector_window_data.visible);
    }
}

pub fn toggle_world_inspector(
    action_state: Query<&ActionState<GlobalActions>>,
    mut world_inspector_params: ResMut<WorldInspectorParams>,
    mut windows: ResMut<Windows>,
) {
    let action_state = action_state.single();
    if action_state.just_released(GlobalActions::ToggleWorldInspector) {
        world_inspector_params.enabled = !world_inspector_params.enabled;
        let window = windows.get_primary_mut().unwrap();
        window.set_cursor_lock_mode(!world_inspector_params.enabled);
        window.set_cursor_visibility(world_inspector_params.enabled);
    }
}
