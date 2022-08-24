use crate::{BoidSettings, GlobalActions};
use bevy::prelude::*;
use bevy_inspector_egui::plugin::InspectorWindows;
use bevy_inspector_egui::WorldInspectorParams;
use leafwing_input_manager::prelude::*;

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
