use bevy_egui::egui;
use bevy_egui::egui::epaint::Shadow;
use bevy_egui::egui::style::{Margin, WidgetVisuals, Widgets};
use bevy_egui::egui::FontFamily::Proportional;
use bevy_egui::egui::{FontId, Stroke, TextStyle, Vec2};
use bevy_egui_kbgp::egui::{Color32, Rounding};

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
            item_spacing: Vec2::new(8.0, 8.0),
            button_padding: Vec2::new(16.0, 8.0),
            window_margin: Margin::same(12.0),
            ..Default::default()
        },
        visuals: egui::Visuals {
            widgets: Widgets {
                inactive: WidgetVisuals {
                    rounding: Rounding::same(15.0),
                    ..egui::Visuals::light().widgets.inactive
                },
                hovered: WidgetVisuals {
                    rounding: Rounding::same(15.0),
                    bg_fill: Color32::from_rgb(255, 80, 80),
                    fg_stroke: Stroke::new(1.0, Color32::WHITE),
                    ..egui::Visuals::light().widgets.hovered
                },
                active: WidgetVisuals {
                    rounding: Rounding::same(15.0),
                    bg_fill: Color32::from_rgb(230, 50, 50),
                    fg_stroke: Stroke::new(1.0, Color32::WHITE),
                    ..egui::Visuals::light().widgets.hovered
                },
                ..egui::Visuals::light().widgets
            },
            window_rounding: Rounding::same(2.5),
            window_shadow: Shadow {
                extrusion: 0.0,
                color: Default::default(),
            },
            ..egui::Visuals::light()
        },
        ..Default::default()
    }
}
