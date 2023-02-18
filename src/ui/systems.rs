use crate::round::PlayerSettings;
use crate::ui::style::get_style;
use crate::ui::Logo;
use crate::{
    AppState, BoidColor, Bot, GlobalActions, MultiplayerMode, PlayerType, RoundSettings, Winner,
};
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, WindowFocused, WindowMode};
use bevy_egui::egui::{Align, Align2, InnerResponse, Response, Ui};
use bevy_egui::{egui, EguiContext};
use bevy_egui_kbgp::KbgpEguiResponseExt;
use egui::vec2;
use leafwing_input_manager::prelude::*;
use std::fmt::Debug;

#[derive(Debug, Reflect, Resource)]
#[reflect(Resource)]
pub struct UiData {
    pub round_settings: RoundSettings,
    #[reflect(ignore)]
    pub window_mode: WindowMode,
    pub window_width: f32,
    pub window_height: f32,
}

#[derive(Debug)]
pub enum UiEvent {
    SettingsSaved,
}

impl Default for UiData {
    fn default() -> Self {
        Self {
            round_settings: Default::default(),
            window_mode: WindowMode::Windowed,
            window_width: 1280.0,
            window_height: 800.0,
        }
    }
}

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
                    if let Err(e) = app_state.set(AppState::LoadRound) {
                        error!("Error when restarting game: {e}");
                    };
                }

                if ui.button("Settings").kbgp_navigation().clicked() {
                    if let Err(e) = app_state.push(AppState::SettingsMenu) {
                        error!("Error when opening settings menu from pause menu: {e}");
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
    #[cfg(not(target_arch = "wasm32"))] mut exit: EventWriter<bevy::app::AppExit>,
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
                    .button("Quick Play")
                    .kbgp_navigation()
                    .kbgp_initial_focus()
                    .clicked()
                {
                    if let Err(e) = app_state.set(AppState::LoadRound) {
                        error!("Error when starting game: {e}");
                    };
                }

                if ui
                    .button("Custom Game")
                    .kbgp_navigation()
                    .kbgp_initial_focus()
                    .clicked()
                {
                    if let Err(e) = app_state.set(AppState::CustomGameMenu) {
                        error!("Error when transitioning to round settings: {e}");
                    };
                }
                ui.small("^ Play custom with friends! ^");

                if ui
                    .button("Settings")
                    .kbgp_navigation()
                    .kbgp_initial_focus()
                    .clicked()
                {
                    if let Err(e) = app_state.push(AppState::SettingsMenu) {
                        error!("Error when transitioning to round settings: {e}");
                    };
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    ui.separator();

                    if ui.button("Exit Game").kbgp_navigation().clicked() {
                        exit.send(bevy::app::AppExit);
                    };
                }
            });
        });
}

pub fn on_title_enter(mut query: Query<&mut Visibility, With<Logo>>) {
    query.single_mut().is_visible = true;
}

pub fn on_title_exit(mut query: Query<&mut Visibility, With<Logo>>) {
    query.single_mut().is_visible = false;
}

pub fn draw_round_settings(
    mut egui_context: ResMut<EguiContext>,
    mut app_state: ResMut<State<AppState>>,
    mut ui_data: ResMut<UiData>,
    mut round_settings: ResMut<RoundSettings>,
) {
    egui::Window::new("Round Settings")
        .title_bar(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.set_width(200.0);

            egui::Grid::new("players")
                .min_row_height(40.0)
                .num_columns(4)
                .show(ui, |ui| {
                    ui.label("Player");
                    ui.label("Type");
                    ui.end_row();
                    let mut remove_indexes = Vec::new();
                    for (i, player_setting) in ui_data.round_settings.players.iter_mut().enumerate()
                    {
                        ui.label(format!("Player {}", i + 1));
                        egui::ComboBox::from_id_source(format!("player_settings_type{i}"))
                            .selected_text(player_setting.player_type.human_bot_label())
                            .show_ui(ui, |ui| {
                                ui.set_width(200.0);
                                ui.selectable_value(
                                    &mut player_setting.player_type,
                                    PlayerType::AnyDevice,
                                    "Human",
                                )
                                .kbgp_navigation();
                                ui.selectable_value(
                                    &mut player_setting.player_type,
                                    PlayerType::Bot(Bot::BoneHead),
                                    "Bot",
                                )
                                .kbgp_navigation();
                            })
                            .response
                            .kbgp_navigation();
                        egui::ComboBox::from_id_source(format!("player_settings_extra_{i}"))
                            .selected_text(player_setting.player_type.to_string())
                            .show_ui(ui, |ui| {
                                ui.set_width(200.0);
                                if player_setting.player_type.is_local() {
                                    for option in PlayerType::human_options() {
                                        ui.selectable_value(
                                            &mut player_setting.player_type,
                                            option,
                                            option.to_string(),
                                        )
                                        .kbgp_navigation();
                                    }
                                } else {
                                    for option in PlayerType::bot_options() {
                                        ui.selectable_value(
                                            &mut player_setting.player_type,
                                            option,
                                            option.to_string(),
                                        )
                                        .kbgp_navigation();
                                    }
                                }
                            })
                            .response
                            .kbgp_navigation();
                        if ui.button("X").kbgp_navigation().clicked() {
                            remove_indexes.push(i);
                        }
                        ui.end_row();
                    }

                    for index in remove_indexes {
                        ui_data.round_settings.players.remove(index);
                    }
                    let new_id = ui_data.round_settings.players.len();
                    if let Some(new_color) = BoidColor::from_index(new_id) {
                        if ui.button("Add Player").kbgp_navigation().clicked() {
                            ui_data.round_settings.players.push(PlayerSettings {
                                player_type: default(),
                                color: new_color,
                            });
                        }
                        ui.end_row();
                    }
                });

            ui.vertical_centered_justified(|ui| {
                if ui_data.round_settings.local_player_count() > 1 {
                    ui.label("Local Multiplayer Mode: ");
                    egui::ComboBox::from_id_source("local_screen_type")
                        .width(ui.available_width())
                        .selected_text(ui_data.round_settings.multiplayer_mode.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut ui_data.round_settings.multiplayer_mode,
                                MultiplayerMode::SharedScreen,
                                MultiplayerMode::SharedScreen.to_string(),
                            )
                            .kbgp_navigation();

                            ui.selectable_value(
                                &mut ui_data.round_settings.multiplayer_mode,
                                MultiplayerMode::SplitScreenVertical,
                                MultiplayerMode::SplitScreenVertical.to_string(),
                            )
                            .kbgp_navigation();

                            ui.selectable_value(
                                &mut ui_data.round_settings.multiplayer_mode,
                                MultiplayerMode::SplitScreenHorizontal,
                                MultiplayerMode::SplitScreenHorizontal.to_string(),
                            )
                            .kbgp_navigation();
                        })
                        .response
                        .kbgp_navigation();
                }
                horizontal_right_to_left_top(ui, |ui| {
                    if ui
                        .button("Start Game")
                        .kbgp_navigation()
                        .kbgp_initial_focus()
                        .clicked()
                    {
                        *round_settings = ui_data.round_settings.clone();
                        if let Err(e) = app_state.set(AppState::LoadRound) {
                            error!("Error when starting custom game: {e}");
                        };
                    }
                    if ui.button("Back").kbgp_navigation().clicked() {
                        *round_settings = ui_data.round_settings.clone();
                        if let Err(e) = app_state.set(AppState::Title) {
                            error!("Error when backing out of custom game: {e}");
                        };
                    }
                });
            });
        });
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
                    if let Err(e) = app_state.set(AppState::LoadRound) {
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

pub fn draw_settings(
    mut egui_context: ResMut<EguiContext>,
    mut app_state: ResMut<State<AppState>>,
    mut ui_data: ResMut<UiData>,
    mut ui_event_writer: EventWriter<UiEvent>,
) {
    egui::Window::new("Settings")
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.set_width(240.0);
            ui.vertical_centered(|ui| ui.heading("Settings"));
            ui.separator();
            ui.vertical_centered_justified(|ui| {
                ui_data.window_mode.draw_as_combo_box(ui, 210.0);
                if ui_data.window_mode == WindowMode::SizedFullscreen {
                    ui.add(
                        egui::DragValue::new(&mut ui_data.window_width)
                            .speed(1.0)
                            .clamp_range(100.0..=50000.0)
                            .prefix("W: "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut ui_data.window_height)
                            .speed(1.0)
                            .clamp_range(100.0..=50000.0)
                            .prefix("H: "),
                    );
                }
                horizontal_right_to_left_top(ui, |ui| {
                    if ui
                        .button("Save")
                        .kbgp_navigation()
                        .kbgp_initial_focus()
                        .clicked()
                    {
                        if let Err(e) = app_state.pop() {
                            error!("Error closing settings: {e}");
                        };
                        ui_event_writer.send(UiEvent::SettingsSaved);
                    }

                    if ui.button("Back").kbgp_navigation().clicked() {
                        if let Err(e) = app_state.pop() {
                            error!("Error closing settings: {e}");
                        };
                    }
                });
            });
        });
}

/// Helper for laying out buttons side by side right aligned.
/// Add contents in reverse order.
pub fn horizontal_right_to_left_top<R>(
    ui: &mut Ui,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<InnerResponse<R>> {
    ui.horizontal_top(|ui| {
        ui.with_layout(
            egui::Layout::right_to_left(Align::Center).with_cross_align(Align::TOP),
            add_contents,
        )
    })
}

pub fn handle_ui_events(
    mut events: EventReader<UiEvent>,
    mut windows: ResMut<Windows>,
    ui_data: Res<UiData>,
) {
    for event in events.iter() {
        info!("{event:?}");
        match event {
            UiEvent::SettingsSaved => {
                let window = windows.primary_mut();
                if window.mode() != ui_data.window_mode {
                    window.set_mode(ui_data.window_mode);
                }
                if window.mode() == WindowMode::SizedFullscreen {
                    window.set_resolution(ui_data.window_width, ui_data.window_height);
                }
            }
        }
    }
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
    window.set_cursor_grab_mode(CursorGrabMode::Locked);
    window.set_cursor_visibility(false);
}

pub fn on_focused(
    mut events: EventReader<WindowFocused>,
    mut windows: ResMut<Windows>,
    app_state: Res<State<AppState>>,
) {
    for event in events.iter() {
        if app_state.current().eq(&AppState::Playing) {
            if let Some(window) = windows.get_mut(event.id) {
                window.set_cursor_grab_mode(CursorGrabMode::Locked);
                window.set_cursor_visibility(false);
            }
        }
    }
}

pub fn on_click(
    mut events: EventReader<MouseButtonInput>,
    mut windows: ResMut<Windows>,
    app_state: Res<State<AppState>>,
) {
    for _ in events.iter() {
        if app_state.current().eq(&AppState::Playing) {
            if let Some(window) = windows.get_primary_mut() {
                window.set_cursor_grab_mode(CursorGrabMode::Locked);
                window.set_cursor_visibility(false);
            }
        }
    }
}

pub fn unlock_mouse(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_cursor_grab_mode(CursorGrabMode::None);
    window.set_cursor_visibility(true);
}

pub fn toggle_fullscreen(
    action_state: Query<&ActionState<GlobalActions>>,
    mut windows: ResMut<Windows>,
) {
    let action_state = action_state.single();
    if action_state.just_released(GlobalActions::ToggleFullScreen) {
        for window in windows.iter_mut() {
            if window.is_focused() {
                match window.mode() {
                    WindowMode::Windowed => window.set_mode(WindowMode::BorderlessFullscreen),
                    _ => window.set_mode(WindowMode::Windowed),
                }
            }
        }
    }
}

pub trait ComboBoxEnum {
    fn combo_box_label() -> &'static str;

    fn values(&self) -> Box<dyn Iterator<Item = Self>>;

    fn value_label(&self) -> String;

    fn draw_as_combo_box(
        &mut self,
        ui: &mut Ui,
        width: f32,
    ) -> InnerResponse<Option<Option<Response>>>
    where
        Self: Eq + Copy,
    {
        let mut inner_response = egui::ComboBox::from_id_source(Self::combo_box_label())
            .selected_text(self.value_label())
            .width(width)
            .show_ui(ui, |ui| {
                self.values()
                    .map(|value| {
                        ui.selectable_value(self, value, value.value_label())
                            .kbgp_navigation()
                    })
                    .fold(None, |a, b| Some(a.map_or(b.clone(), |a| a | b.clone())))
            });

        inner_response.response = inner_response.response.kbgp_navigation();
        inner_response
    }
}

impl ComboBoxEnum for WindowMode {
    fn combo_box_label() -> &'static str {
        "Window Mode"
    }

    fn values(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            [
                Self::Windowed,
                Self::BorderlessFullscreen,
                #[cfg(not(target_arch = "wasm32"))]
                Self::Fullscreen,
            ]
            .iter()
            .copied(),
        )
    }

    fn value_label(&self) -> String {
        match self {
            WindowMode::Windowed => "Windowed",
            #[cfg(not(target_arch = "wasm32"))]
            WindowMode::BorderlessFullscreen => "Borderless Fullscreen",
            #[cfg(target_arch = "wasm32")]
            WindowMode::BorderlessFullscreen => "Fullscreen",
            WindowMode::SizedFullscreen => "Fullscreen Custom",
            WindowMode::Fullscreen => "Fullscreen",
        }
        .to_string()
    }
}
