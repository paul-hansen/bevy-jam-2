use bevy_egui::egui;
use bevy_egui::egui::epaint::Shadow;
use bevy_egui::egui::style::Margin;
use bevy_egui::egui::FontFamily::Proportional;
use bevy_egui::egui::{FontId, TextStyle, Vec2};
use bevy_egui_kbgp::egui::Rounding;

pub fn get_style() -> egui::Style {
    egui::Style {
        text_styles: [
            (TextStyle::Heading, FontId::new(32.0, Proportional)),
            (TextStyle::Body, FontId::new(18.0, Proportional)),
            (TextStyle::Monospace, FontId::new(18.0, Proportional)),
            (TextStyle::Button, FontId::new(18.0, Proportional)),
            (TextStyle::Small, FontId::new(12.0, Proportional)),
        ]
        .into(),
        spacing: egui::style::Spacing {
            button_padding: Vec2::new(16.0, 8.0),
            window_margin: Margin::same(10.0),
            ..Default::default()
        },
        visuals: egui::Visuals {
            window_rounding: Rounding::same(1.5),
            window_shadow: Shadow {
                extrusion: 0.0,
                color: Default::default(),
            },
            ..egui::Visuals::light()
        },
        ..Default::default()
    }
}
