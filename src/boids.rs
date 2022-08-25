use crate::{how_much_right_or_left, Actions, ARENA_PADDING, ARENA_RADIUS, BOID_SCALE};
use bevy::prelude::*;
use bevy_inspector_egui::egui::Ui;
use bevy_inspector_egui::{Context, Inspectable};
use bevy_prototype_debug_lines::DebugLines;
use leafwing_input_manager::action_state::ActionData;
use leafwing_input_manager::axislike::DualAxisData;
use leafwing_input_manager::orientation::{Orientation, Rotation};
use leafwing_input_manager::prelude::*;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;
use std::mem;

#[derive(Inspectable, Debug)]
pub struct BoidSettings {
    cohesion_enabled: bool,
    separation_enabled: bool,
    alignment_enabled: bool,
    /// The maximum speed the boid is allowed to go in units per second
    #[inspectable(min = 0.0, max = 9999.0)]
    max_speed: f32,
    /// The minimum speed the boid is allowed to go in units per second
    #[inspectable(min = 0.0, max = 9999.0)]
    min_speed: f32,
    /// The amount the boid's speed changes by in units per second
    #[inspectable(min = 0.0, max = 9999.0)]
    acceleration: f32,
    /// The deceleration applied to the boid every frame in units per second
    #[inspectable(min = 0.0, max = 9999.0)]
    drag: f32,
    #[inspectable(min = 0.0, max = 3600.0)]
    max_turn_rate_per_second: f32,
    #[inspectable(min = 0.0, max = 1000.0)]
    separation_distance: f32,
    #[inspectable(min = 0.0, max = 1000.0)]
    capture_range: f32,
    debug_lines: bool,
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
            debug_lines: false,
        }
    }
}

#[derive(Component, Default)]
pub struct Boid {}

#[derive(Component, Default, Debug)]
pub struct Velocity {
    forward: f32,
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
    entities: Vec<Entity>,
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
    for (entity, transform, mut capture_neighbors, mut separation_neighbors) in query.iter_mut() {
        let mut c = Vec::new();
        let mut s = Vec::new();
        for (target, position) in positions.iter().filter(|(t, _)| t.id() != entity.id()) {
            let distance = transform
                .translation
                .truncate()
                .distance(position.truncate());
            if distance < boid_settings.separation_distance {
                s.push(*target)
            }
            if distance < boid_settings.capture_range {
                c.push(*target);
            }
        }
        capture_neighbors.entities = c;
        separation_neighbors.entities = s;
    }
}

#[derive(Component, Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub enum BoidColor {
    Red,
    Green,
    Blue,
    Yellow,
}

impl BoidColor {
    pub fn from_index(i: usize) -> Option<Self> {
        match i {
            0 => Some(Self::Red),
            1 => Some(Self::Green),
            2 => Some(Self::Blue),
            3 => Some(Self::Yellow),
            _ => None,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            BoidColor::Red => Color::RED,
            BoidColor::Green => Color::GREEN,
            BoidColor::Blue => Color::BLUE,
            BoidColor::Yellow => Color::YELLOW,
        }
    }
}

pub fn update_boid_transforms(
    mut boid_query: Query<
        (
            &mut Transform,
            &mut ActionState<Actions>,
            &BoidAveragedInputs,
            &mut Velocity,
        ),
        With<Boid>,
    >,
    time: Res<Time>,
    mut lines: ResMut<DebugLines>,
    boid_settings: Res<BoidSettings>,
) {
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
        if direction.length_squared() > (ARENA_RADIUS - ARENA_PADDING).powf(2.) {
            let angle = direction.y.atan2(direction.x) - FRAC_PI_2;

            transform.rotation.rotate_towards(
                Quat::from_axis_angle(Vec3::Z, angle),
                Some(Rotation::from_radians(FRAC_PI_2 * time.delta_seconds())),
            );
        } else {
            add_axis_input(
                &mut action_state,
                Actions::Rotate,
                DualAxisData::new(-inputs.turn_average(), 0.0),
            );
            add_axis_input(
                &mut action_state,
                Actions::Throttle,
                DualAxisData::new(0.0, inputs.speed_average()),
            );

            if let Some(axis_data) = action_state.clamped_axis_pair(Actions::Rotate) {
                transform.rotate_z(
                    -axis_data.x()
                        * boid_settings.max_turn_rate_per_second.to_radians()
                        * time.delta_seconds(),
                );
            }

            if let Some(axis_data) = action_state.clamped_axis_pair(Actions::Throttle) {
                if axis_data.length_squared() > 0.01 {
                    acceleration += boid_settings.acceleration * axis_data.y();
                }
            }

            if let Some(axis_data) = action_state.clamped_axis_pair(Actions::Direction) {
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

        if action_state.pressed(Actions::Boost) {
            velocity.forward += boid_settings.acceleration;
        }

        velocity.forward += (acceleration - boid_settings.drag) * time.delta_seconds();
        velocity.forward = velocity
            .forward
            .clamp(boid_settings.min_speed, boid_settings.max_speed);
        transform.translation += forward * time.delta_seconds() * velocity.forward;
    }
}

pub fn clear_inputs(mut query: Query<(&mut BoidAveragedInputs, &mut ActionState<Actions>)>) {
    for (mut inputs, mut action_state) in query.iter_mut() {
        inputs.reset();
        action_state.set_action_data(Actions::Rotate, ActionData::default());
    }
}

#[allow(clippy::type_complexity)]
pub fn calculate_cohesion_inputs(
    mut query: Query<
        (&Transform, &mut BoidAveragedInputs, &BoidColor, &Velocity),
        (With<Boid>, Without<Leader>),
    >,
    leader_query: Query<(&Transform, &BoidColor, &Velocity), With<Leader>>,
    mut lines: ResMut<DebugLines>,
    boid_settings: Res<BoidSettings>,
) {
    if !boid_settings.cohesion_enabled {
        return;
    }
    // Turn and move towards the leader's position if they have one.
    for (transform, mut inputs, color, velocity) in query.iter_mut() {
        if let Some((leader_transform, _, leader_velocity)) =
            leader_query.iter().find(|(_, c, _)| *c == color)
        {
            let leader_position = leader_transform.translation.truncate();

            let direction_to_target =
                (leader_position - transform.translation.truncate()).normalize();

            let turn_towards_leader_direction = -how_much_right_or_left(
                transform,
                &(transform.translation.truncate() + direction_to_target),
            );
            let speed_up_down = match leader_velocity.forward - velocity.forward {
                x if x > 120.0 => 1.0,
                x if x > 0.0 => 0.5,
                x if x < -3.0 => -1.0,
                _ => 0.0,
            };
            if boid_settings.debug_lines {
                lines.line_gradient(
                    transform.translation,
                    (direction_to_target.extend(0.0) * 20.0) + transform.translation,
                    0.0,
                    Color::rgba(0.2, 1.0, 0.2, 1.0),
                    Color::rgba(0.2, 1.0, 0.2, 0.0),
                );
            }

            inputs.add_turn(turn_towards_leader_direction);
            inputs.add_speed(speed_up_down);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn calculate_separation_inputs(
    mut query: Query<
        (
            &Transform,
            &BoidNeighborsSeparation,
            &mut BoidAveragedInputs,
        ),
        (With<Boid>, Without<InputMap<Actions>>),
    >,
    transforms: Query<&Transform>,
    mut lines: ResMut<DebugLines>,
    boid_settings: Res<BoidSettings>,
) {
    if !boid_settings.separation_enabled {
        return;
    }
    for (transform, neighbors, mut inputs) in query.iter_mut() {
        transforms
            .iter_many(&neighbors.entities)
            .for_each(|target| {
                let direction = how_much_right_or_left(transform, &target.translation.truncate());
                if boid_settings.debug_lines {
                    lines.line_gradient(
                        transform.translation,
                        ((target.translation - transform.translation) * 0.5)
                            + transform.translation,
                        0.0,
                        Color::rgba(1.0, 0.0, 0.0, (direction + 1.0) / 2.0),
                        Color::rgba(1.0, 0.0, 0.0, 0.2),
                    );
                }
                inputs.add_turn(direction);
            });
    }
}

#[allow(clippy::type_complexity)]
pub fn calculate_alignment_inputs(
    mut query: Query<
        (&Transform, &mut BoidAveragedInputs, &BoidColor),
        (With<Boid>, Without<InputMap<Actions>>),
    >,
    leader_query: Query<(&Transform, &BoidColor), With<Leader>>,
    mut lines: ResMut<DebugLines>,
    boid_settings: Res<BoidSettings>,
) {
    if !boid_settings.alignment_enabled {
        return;
    }
    for (transform, mut inputs, color) in query.iter_mut() {
        if let Some((leader_transform, _)) = leader_query.iter().find(|(_, c)| *c == color) {
            let average = leader_transform.up().truncate();
            if boid_settings.debug_lines {
                lines.line_colored(
                    transform.translation,
                    transform.translation + (average.extend(0.0) * 20.0),
                    0.0,
                    Color::VIOLET,
                );
            }
            inputs.add_turn(-how_much_right_or_left(
                &Transform::from_rotation(transform.rotation),
                &average,
            ));
        }
    }
}

pub fn update_boid_color(mut query: Query<(&mut Sprite, &BoidColor), Changed<BoidColor>>) {
    for (mut sprite, color) in query.iter_mut() {
        sprite.color = color.color();
    }
}

pub enum GameEvent {
    LeaderCaptured(BoidColor),
}

pub fn propagate_boid_color(
    mut commands: Commands,
    mut query: Query<(Entity, &BoidNeighborsCaptureRange), With<Boid>>,
    mut boid_colors: Query<&mut BoidColor>,
    leader_query: Query<&Leader>,
    mut event_writer: EventWriter<GameEvent>,
) {
    for (entity, neighbors) in query.iter_mut() {
        let mut neighbor_color_counts: HashMap<BoidColor, usize> = HashMap::new();
        for other_color in boid_colors.iter_many(&neighbors.entities) {
            let count = neighbor_color_counts.entry(*other_color).or_insert(0);
            *count += 1;
        }
        let dominate_color = neighbor_color_counts.into_iter().max_by_key(|(_, c)| *c);
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
}

pub fn leader_removed(removals: RemovedComponents<Leader>, mut query: Query<&mut Transform>) {
    for entity in removals.iter() {
        if let Ok(mut transform) = query.get_mut(entity) {
            transform.scale = BOID_SCALE;
        }
    }
}

pub fn leader_defeated(
    mut commands: Commands,
    mut event_reader: EventReader<GameEvent>,
    mut query: Query<(Entity, &BoidColor, &mut Sprite)>,
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
                            .remove::<InputMap<Actions>>()
                            .remove::<BoidColor>();
                    }
                }
            }
        }
    }
}

fn add_axis_input(
    action_state: &mut ActionState<Actions>,
    action: Actions,
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
