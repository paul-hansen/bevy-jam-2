use bevy::prelude::*;
use bevy_egui::{egui, EguiPlugin};
use bevy_inspector_egui::bevy_inspector::ui_for_world;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugin(DefaultInspectorConfigPlugin);
        }
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugin(EguiPlugin);
        }
        app.add_system(hotkey);
    }
}

fn hotkey(world: &mut World, mut show_inspector: Local<bool>) {
    if let Some(keys) = world.get_resource::<Input<KeyCode>>() {
        if keys.just_pressed(KeyCode::F12) {
            *show_inspector = !*show_inspector;
        }
    }
    let egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();
    let old_style = egui_context.style();
    let inspector_style = egui::style::Style {
        visuals: egui::Visuals::light(),
        ..default()
    };
    egui_context.set_style(inspector_style);
    if *show_inspector {
        egui::Window::new("Inspector")
            .default_size([90.0, 400.0])
            .show(&egui_context, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui_for_world(world, ui);
                    ui.allocate_space(ui.available_size());
                })
            });
    }
    egui_context.set_style(old_style);
}
