use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::window::{PrimaryWindow, WindowResized};

/// A component that will update the attached camera's viewport to be sized relative to the screen
/// ```
/// // Create a viewport that takes up the top left section of the screen.
/// let vp = ViewportRelative::new(0.0, 0.0, 0.5, 0.5)
/// ```
#[derive(Component, Debug, Reflect, Copy, Clone)]
#[reflect(Component)]
pub struct ViewportRelative {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub border: f32,
}

impl Default for ViewportRelative {
    fn default() -> Self {
        Self::fullscreen()
    }
}

impl ViewportRelative {
    pub fn new(x: f32, y: f32, width: f32, height: f32, border: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            border,
        }
    }

    pub fn split_vertical(&self, sections: usize) -> Vec<ViewportRelative> {
        let new_width = self.width / sections as f32;
        (0..sections)
            .map(|i| {
                ViewportRelative::new(
                    self.x + (new_width * i as f32),
                    self.y,
                    new_width,
                    self.height,
                    self.border,
                )
            })
            .collect()
    }

    pub fn split_horizontal(&self, sections: usize) -> Vec<ViewportRelative> {
        let new_height = self.height / sections as f32;
        (0..sections)
            .map(|i| {
                ViewportRelative::new(
                    self.x,
                    self.y + (new_height * i as f32),
                    self.width,
                    new_height,
                    self.border,
                )
            })
            .collect()
    }

    pub fn top() -> Self {
        Self::new(0.0, 0.0, 1.0, 0.5, 0.0)
    }

    pub fn bottom() -> Self {
        Self::new(0.0, 0.5, 1.0, 0.5, 0.0)
    }

    pub fn left() -> Self {
        Self::new(0.0, 0.0, 0.5, 1.0, 0.0)
    }

    pub fn right() -> Self {
        Self::new(0.5, 0.0, 0.5, 1.0, 0.0)
    }

    pub fn top_left() -> Self {
        Self::new(0.0, 0.0, 0.5, 0.5, 0.0)
    }

    pub fn top_right() -> Self {
        Self::new(0.5, 0.0, 0.5, 0.5, 0.0)
    }

    pub fn bottom_left() -> Self {
        Self::new(0.0, 0.5, 0.5, 0.5, 0.0)
    }

    pub fn bottom_right() -> Self {
        Self::new(0.5, 0.5, 0.5, 0.5, 0.0)
    }

    pub fn fullscreen() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0, 0.0)
    }

    pub fn with_border(mut self, border: f32) -> Self {
        self.border = border;
        self
    }
}

pub fn set_camera_viewports(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut resize_events: EventReader<WindowResized>,

    mut query: Query<(&mut Camera, &ViewportRelative)>,
    added_query: Query<Added<ViewportRelative>>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.

    let window = windows.single();
    if resize_events.iter().count() != 0 || !added_query.is_empty() {
        for (mut camera, relative_viewport) in query.iter_mut() {
            camera.viewport = Some(Viewport {
                physical_position: UVec2::new(
                    ((window.physical_width() as f32 * relative_viewport.x)
                        + relative_viewport.border) as u32,
                    ((window.physical_height() as f32 * relative_viewport.y)
                        + relative_viewport.border) as u32,
                ),
                physical_size: UVec2::new(
                    ((window.physical_width() as f32 * relative_viewport.width)
                        - (relative_viewport.border * 2.0)) as u32,
                    ((window.physical_height() as f32 * relative_viewport.height)
                        - (relative_viewport.border * 2.0)) as u32,
                ),
                ..default()
            });
        }
    }
}

pub struct PlayerViewports {
    viewports: Vec<ViewportRelative>,
    border_thickness: f32,
}

pub enum ViewportLayoutPreference {
    Horizontal,
    Vertical,
}

impl PlayerViewports {
    pub fn new(
        player_count: u8,
        layout_preference: ViewportLayoutPreference,
        border_thickness: f32,
    ) -> Self {
        let viewports = match player_count {
            1 => vec![ViewportRelative::fullscreen()],
            2 => match layout_preference {
                ViewportLayoutPreference::Horizontal => {
                    vec![ViewportRelative::top(), ViewportRelative::bottom()]
                }
                ViewportLayoutPreference::Vertical => {
                    vec![ViewportRelative::left(), ViewportRelative::right()]
                }
            },
            3 => match layout_preference {
                ViewportLayoutPreference::Horizontal => {
                    vec![
                        ViewportRelative::top(),
                        ViewportRelative::bottom_left(),
                        ViewportRelative::bottom_right(),
                    ]
                }
                ViewportLayoutPreference::Vertical => {
                    vec![
                        ViewportRelative::left(),
                        ViewportRelative::top_right(),
                        ViewportRelative::bottom_right(),
                    ]
                }
            },
            4 => vec![
                ViewportRelative::top_left(),
                ViewportRelative::top_right(),
                ViewportRelative::bottom_left(),
                ViewportRelative::bottom_right(),
            ],
            x if x <= 6 => match layout_preference {
                ViewportLayoutPreference::Horizontal => ViewportRelative::top()
                    .split_vertical((x - 3) as usize)
                    .into_iter()
                    .chain(ViewportRelative::bottom().split_vertical(3))
                    .collect(),
                ViewportLayoutPreference::Vertical => {
                    ViewportRelative::fullscreen().split_vertical(x as usize)
                }
            },
            x if x <= 8 => match layout_preference {
                ViewportLayoutPreference::Horizontal => ViewportRelative::top()
                    .split_vertical((x - 4) as usize)
                    .into_iter()
                    .chain(ViewportRelative::bottom().split_vertical(4))
                    .collect(),
                ViewportLayoutPreference::Vertical => ViewportRelative::left()
                    .split_horizontal((x - 4) as usize)
                    .into_iter()
                    .chain(ViewportRelative::right().split_horizontal(4))
                    .collect(),
            },
            _ => unimplemented!(),
        };
        debug_assert_eq!(viewports.len() as u8, player_count);
        Self {
            viewports,
            border_thickness,
        }
    }

    pub fn get(&self, id: usize) -> ViewportRelative {
        self.viewports[id].with_border(self.border_thickness)
    }
}
