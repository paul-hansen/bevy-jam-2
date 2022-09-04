use crate::{
    AppState, PlayerActions, RoundSettings, Winner, ARENA_PADDING, BOID_SCALE, LEADER_SCALE,
};
use bevy::ecs::schedule::StateError;
use bevy::prelude::*;
use bevy_inspector_egui::egui::Ui;
use bevy_inspector_egui::{Context, Inspectable};
use bevy_prototype_debug_lines::DebugLines;
use itertools::Itertools;
use leafwing_input_manager::action_state::ActionData;
use leafwing_input_manager::axislike::DualAxisData;
use leafwing_input_manager::orientation::{Orientation, Rotation};
use leafwing_input_manager::prelude::*;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;
use std::mem;

#[derive(Inspectable, Debug)]
pub struct BoidSettings {
    pub cohesion_enabled: bool,
    pub separation_enabled: bool,
    pub alignment_enabled: bool,
    /// The maximum speed the boid is allowed to go in units per second
    #[inspectable(min = 0.0, max = 9999.0)]
    pub max_speed: f32,
    /// The minimum speed the boid is allowed to go in units per second
    #[inspectable(min = 0.0, max = 9999.0)]
    pub min_speed: f32,
    /// The amount the boid's speed changes by in units per second
    #[inspectable(min = 0.0, max = 9999.0)]
    pub acceleration: f32,
    /// The deceleration applied to the boid every frame in units per second
    #[inspectable(min = 0.0, max = 9999.0)]
    pub drag: f32,
    #[inspectable(min = 0.0, max = 3600.0)]
    pub max_turn_rate_per_second: f32,
    #[inspectable(min = 0.0, max = 1000.0)]
    pub separation_distance: f32,
    #[inspectable(min = 0.0, max = 1000.0)]
    pub capture_range: f32,
    #[inspectable(min = 0.0, max = 1000.0)]
    pub vision_range: f32,
    pub debug_lines: bool,
}

impl Default for BoidSettings {
    fn default() -> Self {
        Self {
            cohesion_enabled: true,
            separation_enabled: true,
            alignment_enabled: true,
            max_speed: 120.0,
            min_speed: 60.0,
            acceleration: 300.0,
            drag: 100.0,
            max_turn_rate_per_second: 520.0,
            separation_distance: 15.0,
            capture_range: 20.0,
            vision_range: 500.0,
            debug_lines: false,
        }
    }
}

#[derive(Component, Default)]
pub struct Boid {}

#[derive(Component, Default, Debug, Inspectable)]
pub struct Velocity {
    pub forward: f32,
}

#[derive(Component, Default)]
pub struct BoidNeighborsCaptureRange {
    entities: Vec<Entity>,
}

impl Inspectable for BoidNeighborsCaptureRange {
    type Attributes = ();

    fn ui(&mut self, ui: &mut Ui, _: Self::Attributes, _: &mut Context) -> bool {
        ui.label(format!("{}", self.entities.len()));
        false
    }
}

#[derive(Component, Default, Reflect)]
pub struct BoidNeighborsSeparation {
    pub entities: Vec<Entity>,
}

impl Inspectable for BoidNeighborsSeparation {
    type Attributes = ();

    fn ui(&mut self, ui: &mut Ui, _: Self::Attributes, _: &mut Context) -> bool {
        ui.label(format!("{}", self.entities.len()));
        false
    }
}

// Collects inputs every frame and averages them.
// Gives equal weight to the different factors pulling on the boids.
// Makes them jiggle back and forth less than adding all the inputs.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct BoidAveragedInputs {
    turn_average: f32,
    turn_count: u32,
    speed_average: f32,
    speed_count: u32,
}

impl BoidAveragedInputs {
    pub fn add_turn(&mut self, direction: f32) {
        if direction.is_nan() {
            error!("Tried to add nan to inputs");
        } else {
            self.turn_count += 1;
            self.turn_average = ((self.turn_average * (self.turn_count - 1) as f32) + direction)
                / self.turn_count as f32;
        }
    }

    pub fn turn_average(&self) -> f32 {
        debug_assert!(!self.turn_average.is_nan());
        self.turn_average
    }

    pub fn add_speed(&mut self, input: f32) {
        if input.is_nan() {
            error!("Tried to add nan to inputs");
        } else {
            self.speed_count += 1;
            self.speed_average = ((self.speed_average * (self.speed_count - 1) as f32) + input)
                / self.speed_count as f32;
        }
    }

    pub fn speed_average(&self) -> f32 {
        debug_assert!(!self.speed_average.is_nan());
        self.speed_average
    }

    pub fn reset(&mut self) {
        self.turn_average = 0.0;
        self.speed_average = 0.0;
        self.turn_count = 0;
        self.speed_count = 0;
    }
}

#[derive(Component, Debug)]
pub struct Leader;

#[allow(clippy::type_complexity)]
pub fn update_boid_neighbors(
    mut query: Query<
        (
            Entity,
            &Transform,
            &mut BoidNeighborsCaptureRange,
            &mut BoidNeighborsSeparation,
        ),
        With<Boid>,
    >,
    boid_settings: Res<BoidSettings>,
) {
    let positions: Vec<(Entity, Vec3)> = query
        .iter()
        .map(|(entity, transform, _, _)| (entity, transform.translation))
        .collect();
    let separation_distance_squared = boid_settings.separation_distance.powf(2.0);
    let capture_range_squared = boid_settings.capture_range.powf(2.0);
    for (entity, transform, mut capture_neighbors, mut separation_neighbors) in query.iter_mut() {
        let mut c = Vec::new();
        let mut s = Vec::new();
        for (target, position) in positions.iter().filter(|(t, _)| t.id() != entity.id()) {
            let distance_squared = transform
                .translation
                .truncate()
                .distance_squared(position.truncate());
            if distance_squared < separation_distance_squared {
                s.push(*target)
            }
            if distance_squared < capture_range_squared {
                c.push(*target);
            }
        }
        capture_neighbors.entities = c;
        separation_neighbors.entities = s;
    }
}

#[derive(Component, Eq, PartialEq, Copy, Clone, Debug, Hash, Inspectable)]
pub enum BoidColor {
    Red,
    Green,
    Blue,
    Yellow,
    Purple,
    Orange,
    Pink,
    Cyan,
}

impl BoidColor {
    pub fn from_index(i: usize) -> Option<Self> {
        match i {
            0 => Some(Self::Red),
            1 => Some(Self::Green),
            2 => Some(Self::Blue),
            3 => Some(Self::Yellow),
            4 => Some(Self::Purple),
            5 => Some(Self::Orange),
            6 => Some(Self::Pink),
            7 => Some(Self::Cyan),
            _ => None,
        }
    }

    pub const ALL: [Self; 8] = [
        Self::Red,
        Self::Green,
        Self::Blue,
        Self::Yellow,
        Self::Purple,
        Self::Orange,
        Self::Pink,
        Self::Cyan,
    ];

    pub fn color(&self) -> Color {
        match self {
            BoidColor::Red => Color::RED,
            BoidColor::Green => Color::GREEN,
            BoidColor::Blue => Color::BLUE,
            BoidColor::Yellow => Color::YELLOW,
            BoidColor::Purple => Color::PURPLE,
            BoidColor::Orange => Color::ORANGE,
            BoidColor::Pink => Color::PINK,
            BoidColor::Cyan => Color::CYAN,
        }
    }
}

pub fn update_boid_transforms(
    mut boid_query: Query<
        (
            &mut Transform,
            &mut ActionState<PlayerActions>,
            &BoidAveragedInputs,
            &mut Velocity,
        ),
        With<Boid>,
    >,
    time: Res<Time>,
    mut lines: ResMut<DebugLines>,
    boid_settings: Res<BoidSettings>,
    round_settings: Res<RoundSettings>,
) {
    let active_arena_radius_squared = (round_settings.arena_radius - ARENA_PADDING).powf(2.);
    for (mut transform, mut action_state, inputs, mut velocity) in boid_query.iter_mut() {
        if boid_settings.debug_lines {
            lines.line_colored(
                transform.translation,
                transform.translation + (transform.up() * 20.0),
                0.0,
                Color::MIDNIGHT_BLUE,
            );
        }

        let forward = transform.up();
        let mut acceleration = 0.0;

        // if headed out of bounds, rotate towards the center
        let direction = -transform.translation.truncate();
        if direction.length_squared() > active_arena_radius_squared {
            let angle = direction.y.atan2(direction.x) - FRAC_PI_2;

            transform.rotation.rotate_towards(
                Quat::from_axis_angle(Vec3::Z, angle),
                Some(Rotation::from_radians(FRAC_PI_2 * time.delta_seconds())),
            );
        } else {
            add_axis_input(
                &mut action_state,
                PlayerActions::Rotate,
                DualAxisData::new(inputs.turn_average(), 0.0),
            );
            add_axis_input(
                &mut action_state,
                PlayerActions::Throttle,
                DualAxisData::new(0.0, inputs.speed_average()),
            );

            if let Some(axis_data) = action_state.clamped_axis_pair(PlayerActions::Rotate) {
                transform.rotate_z(
                    -axis_data.x()
                        * boid_settings.max_turn_rate_per_second.to_radians()
                        * time.delta_seconds(),
                );
            }

            if let Some(axis_data) = action_state.clamped_axis_pair(PlayerActions::Throttle) {
                if axis_data.length_squared() > 0.01 {
                    acceleration += boid_settings.acceleration * axis_data.y();
                }
            }

            if let Some(axis_data) = action_state.clamped_axis_pair(PlayerActions::Direction) {
                if axis_data.length_squared() > 0.01 {
                    transform.rotation.rotate_towards(
                        Quat::from_rotation_z((-axis_data.x()).atan2(axis_data.y())),
                        Some(Rotation::from_degrees(
                            boid_settings.max_turn_rate_per_second * time.delta_seconds(),
                        )),
                    );
                }
            }
        }

        if action_state.pressed(PlayerActions::Boost) {
            velocity.forward += boid_settings.acceleration;
        }

        velocity.forward += (acceleration - boid_settings.drag) * time.delta_seconds();
        velocity.forward = velocity.forward.clamp(
            // clamp requires that min <= to max, adding the extra min here so it
            // doesn't panic if max_speed is set to lower than min_speed via the inspector.
            boid_settings.min_speed.min(boid_settings.max_speed),
            boid_settings.max_speed,
        );
        transform.translation += forward * time.delta_seconds() * velocity.forward;
    }
}

pub fn clear_inputs(mut query: Query<(&mut BoidAveragedInputs, &mut ActionState<PlayerActions>)>) {
    for (mut inputs, mut action_state) in query.iter_mut() {
        inputs.reset();
        action_state.set_action_data(PlayerActions::Rotate, ActionData::default());
        action_state.set_action_data(PlayerActions::Boost, ActionData::default());
    }
}

pub fn update_boid_color(mut query: Query<(&mut Sprite, &BoidColor), Changed<BoidColor>>) {
    for (mut sprite, color) in query.iter_mut() {
        sprite.color = color.color();
    }
}

pub enum GameEvent {
    LeaderCaptured(BoidColor),
    GameOver(Winner),
}

pub fn propagate_boid_color(
    mut commands: Commands,
    query: Query<(Entity, &BoidNeighborsCaptureRange)>,
    mut boid_colors: Query<&mut BoidColor>,
    leader_query: Query<&Leader>,
    mut event_writer: EventWriter<GameEvent>,
) {
    for (entity, neighbors) in query.iter() {
        let mut neighbor_color_counts: HashMap<BoidColor, usize> = HashMap::new();

        // Build a list of all the colors with our color last if we have one.
        // Use this later to skip checking neighbors of our color if there aren't other colors.
        let our_color = boid_colors.get(entity);
        let mut all_colors = BoidColor::ALL.to_vec();
        if let Ok(our_color) = our_color {
            all_colors = all_colors
                .iter()
                .filter(|c| *c != our_color)
                .cloned()
                .chain([*our_color])
                .collect();
        }

        for color in all_colors {
            if Ok(&color) == our_color && neighbor_color_counts.keys().len() == 0 {
                continue;
            }
            let mut results = Vec::new();
            get_neighbors_of_color_recursive(
                entity,
                neighbors,
                color,
                &query,
                &boid_colors,
                &mut results,
                10,
            );
            if !results.is_empty() {
                neighbor_color_counts.insert(color, results.len());
            }
        }

        let dominate_color = neighbor_color_counts
            .into_iter()
            .filter(|(_, v)| *v != 0)
            .max_by_key(|(_, c)| *c);
        if let Some((dominate_color, count)) = dominate_color {
            if let Ok(mut our_color) = boid_colors.get_mut(entity) {
                // Decide if we should convert it
                if *our_color != dominate_color && count > 1 {
                    // Apply the conversion
                    if leader_query.contains(entity) {
                        // We converted a leader!
                        event_writer.send(GameEvent::LeaderCaptured(*our_color))
                        // We don't want to change the color yet as it will be handled in the
                        // leader captured system.
                    } else {
                        let _ = mem::replace(&mut *our_color, dominate_color);
                    }
                }
            } else {
                // Boids without a color always get converted.
                commands.entity(entity).insert(dominate_color);
            }
        }
    }

    // Check if there is only one color left
    let mut remaining_colors = boid_colors.iter().unique();
    if let Some(first_color) = remaining_colors.next() {
        if remaining_colors.count() == 0 {
            event_writer.send(GameEvent::GameOver(Winner {
                color: *first_color,
            }));
        }
    }
}

/// Get all the neighbors in capture range and their neighbors and their neighbors etc.
/// Does not include itself.
///
/// Pass a vector of all the previously visited entities to prevent duplicates
pub fn get_neighbors_of_color_recursive(
    entity: Entity,
    neighbors: &BoidNeighborsCaptureRange,
    color: BoidColor,
    query: &Query<(Entity, &BoidNeighborsCaptureRange)>,
    colors: &Query<&mut BoidColor>,
    results: &mut Vec<Entity>,
    depth: usize,
) {
    if depth == 0 {
        return;
    }
    neighbors
        .entities
        .iter()
        .flat_map(|neighbor| query.get(*neighbor))
        // Remove colorless
        .flat_map(|(e, n)| colors.get(e).map(|c| (e, n, *c)))
        .filter(|(e, _, c)| *c == color && *e != entity)
        .for_each(|(e, neighbors, color)| {
            if !results.contains(&e) {
                results.push(e);
                get_neighbors_of_color_recursive(
                    e,
                    neighbors,
                    color,
                    query,
                    colors,
                    results,
                    depth - 1,
                );
            }
        });
}

pub fn leader_removed(removals: RemovedComponents<Leader>, mut query: Query<&mut Transform>) {
    for entity in removals.iter() {
        if let Ok(mut transform) = query.get_mut(entity) {
            transform.scale = BOID_SCALE;
        }
    }
}

pub fn leader_added(mut query: Query<&mut Transform, Added<Leader>>) {
    for mut transform in query.iter_mut() {
        transform.scale = LEADER_SCALE;
    }
}

pub fn leader_defeated(
    mut commands: Commands,
    mut event_reader: EventReader<GameEvent>,
    mut query: Query<(Entity, &BoidColor, &mut Sprite)>,
    mut app_state: ResMut<State<AppState>>,
) {
    for event in event_reader.iter() {
        match event {
            GameEvent::LeaderCaptured(captured_color) => {
                info!("{:?} Leader Defeated", captured_color);
                for (entity, color, mut sprite) in query.iter_mut() {
                    if color == captured_color {
                        sprite.color = Color::WHITE;
                        commands
                            .entity(entity)
                            .remove::<Leader>()
                            .remove::<InputMap<PlayerActions>>()
                            .remove::<BoidColor>();
                    }
                }
            }
            GameEvent::GameOver(winner) => {
                commands.insert_resource(winner.clone());
                if let Err(e) = app_state.push(AppState::GameOver) {
                    match e {
                        StateError::AlreadyInState => {}
                        StateError::StateAlreadyQueued => {}
                        StateError::StackEmpty => {
                            error!("Failed to change app state to game over: {e}")
                        }
                    }
                };
            }
        }
    }
}

fn add_axis_input(
    action_state: &mut ActionState<PlayerActions>,
    action: PlayerActions,
    axis_data: DualAxisData,
) {
    let mut data = action_state.action_data(action);
    data.value += axis_data.x();
    data.axis_pair = Some(
        data.axis_pair
            .map_or(axis_data, |u| u.merged_with(axis_data)),
    );
    action_state.set_action_data(action, data);
}
