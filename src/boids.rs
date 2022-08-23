use crate::{how_much_right_or_left, Actions, ARENA_PADDING, ARENA_RADIUS, BOID_SCALE};
use bevy::prelude::*;
use bevy_inspector_egui::egui::Ui;
use bevy_inspector_egui::{Context, Inspectable};
use bevy_prototype_debug_lines::DebugLines;
use leafwing_input_manager::action_state::ActionData;
use leafwing_input_manager::axislike::DualAxisData;
use leafwing_input_manager::prelude::*;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::mem;

#[derive(Inspectable, Debug)]
pub struct BoidSettings {
    cohesion_enabled: bool,
    separation_enabled: bool,
    alignment_enabled: bool,
    #[inspectable(min = 0.0, max = 9999.0)]
    speed: f32,
    #[inspectable(min = 0.0, max = PI * 180.0)]
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
            speed: 80.0,
            max_turn_rate_per_second: PI * 2.8,
            separation_distance: 15.0,
            capture_range: 20.0,
            debug_lines: false,
        }
    }
}

#[derive(Component, Default)]
pub struct Boid {}

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

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct BoidTurnDirectionInputs {
    average: f32,
    count: u32,
}

impl BoidTurnDirectionInputs {
    pub fn add(&mut self, direction: f32) {
        if direction.is_nan() {
            error!("Tried to add nan to inputs");
        } else {
            self.count += 1;
            self.average =
                ((self.average * (self.count - 1) as f32) + direction) / self.count as f32;
        }
    }

    pub fn average(&self) -> f32 {
        debug_assert!(!self.average.is_nan());
        self.average
    }

    pub fn reset(&mut self) {
        self.average = 0.0;
        self.count = 0;
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
            &mut BoidTurnDirectionInputs,
        ),
        With<Boid>,
    >,
    time: Res<Time>,
    mut lines: ResMut<DebugLines>,
    boid_settings: Res<BoidSettings>,
) {
    for (mut transform, mut action_state, mut inputs) in boid_query.iter_mut() {
        if boid_settings.debug_lines {
            lines.line_colored(
                transform.translation,
                transform.translation + (transform.up() * 20.0),
                0.0,
                Color::MIDNIGHT_BLUE,
            );
        }

        let forward = transform.up();

        // The more into the arena padding the more it turns, making it turn around.
        let r = ((transform.translation.length() - (ARENA_RADIUS - ARENA_PADDING)).max(0.0)
            / ARENA_PADDING)
            * 2.0;
        if r.abs() > 0.0 {
            inputs.add(r);
        }
        add_axis_input(
            &mut action_state,
            Actions::Move,
            DualAxisData::new(-inputs.average(), 0.0),
        );
        inputs.reset();
        let mut speed_multiplier = 1.0;
        if let Some(axis_data) = action_state.clamped_axis_pair(Actions::Move) {
            transform.rotate_z(
                -axis_data.x() * boid_settings.max_turn_rate_per_second * time.delta_seconds(),
            );
            speed_multiplier += axis_data.y();
        }

        transform.translation +=
            forward * time.delta_seconds() * boid_settings.speed * speed_multiplier;
        action_state.set_action_data(Actions::Move, ActionData::default());
    }
}

#[allow(clippy::type_complexity)]
pub fn calculate_cohesion_inputs(
    mut query: Query<
        (&Transform, &mut BoidTurnDirectionInputs, &BoidColor),
        (With<Boid>, Without<Leader>),
    >,
    leader_query: Query<(&Transform, &BoidColor), With<Leader>>,
    mut lines: ResMut<DebugLines>,
    boid_settings: Res<BoidSettings>,
) {
    if !boid_settings.cohesion_enabled {
        return;
    }
    // Rotate towards the average position of other boids within cohesion range.
    for (transform, mut inputs, color) in query.iter_mut() {
        if let Some((leader_transform, _)) = leader_query.iter().find(|(_, c)| *c == color) {
            let average_position_of_near_boids = leader_transform.translation.truncate();
            if average_position_of_near_boids.is_nan() {
                // There were no neighbors so it divided by zero when averaging.
                // We don't need to add any inputs if it has no neighbors so we move on..
                continue;
            }
            let direction_to_target =
                (average_position_of_near_boids - transform.translation.truncate()).normalize();

            let turn_direction_to_center_of_near = -how_much_right_or_left(
                transform,
                &(transform.translation.truncate() + direction_to_target),
            );
            if boid_settings.debug_lines {
                lines.line_gradient(
                    transform.translation,
                    (direction_to_target.extend(0.0) * 20.0) + transform.translation,
                    0.0,
                    Color::rgba(0.2, 1.0, 0.2, 1.0),
                    Color::rgba(0.2, 1.0, 0.2, 0.0),
                );
            }
            inputs.add(turn_direction_to_center_of_near);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn calculate_separation_inputs(
    mut query: Query<
        (
            &Transform,
            &BoidNeighborsSeparation,
            &mut BoidTurnDirectionInputs,
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
                inputs.add(direction);
            });
    }
}

#[allow(clippy::type_complexity)]
pub fn calculate_alignment_inputs(
    mut query: Query<
        (&Transform, &mut BoidTurnDirectionInputs, &BoidColor),
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
            inputs.add(-how_much_right_or_left(
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
