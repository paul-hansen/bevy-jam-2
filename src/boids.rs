use crate::{how_much_right_or_left, Average, ARENA_PADDING, ARENA_RADIUS};
use bevy::prelude::*;
use bevy_inspector_egui::egui::Ui;
use bevy_inspector_egui::{Context, Inspectable};
use bevy_prototype_debug_lines::DebugLines;
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
    cohesion_distance: f32,
    #[inspectable(min = 0.0, max = 1000.0)]
    alignment_distance: f32,
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
            separation_distance: 10.0,
            cohesion_distance: 120.0,
            alignment_distance: 25.0,
            debug_lines: false,
        }
    }
}

#[derive(Component, Default)]
pub struct Boid {}

#[derive(Component, Default)]
pub struct BoidNeighborsCohesion {
    entities: Vec<Entity>,
}

impl Inspectable for BoidNeighborsCohesion {
    type Attributes = ();

    fn ui(&mut self, ui: &mut Ui, _: Self::Attributes, _: &mut Context) -> bool {
        ui.label(format!("{}", self.entities.len()));
        false
    }
}

#[derive(Component, Default, Reflect)]
pub struct BoidNeighborsAlignment {
    entities: Vec<Entity>,
}

impl Inspectable for BoidNeighborsAlignment {
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
            if self.count == 0 {
                self.average = direction;
            } else {
                self.average = self.average + ((direction - self.average) / self.count as f32);
            }
            self.count += 1;
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

pub fn update_boid_neighbors(
    mut query: Query<
        (
            Entity,
            &Transform,
            &mut BoidNeighborsAlignment,
            &mut BoidNeighborsCohesion,
            &mut BoidNeighborsSeparation,
        ),
        With<Boid>,
    >,
    boid_settings: Res<BoidSettings>,
) {
    let positions: Vec<(Entity, Vec3)> = query
        .iter()
        .map(|(entity, transform, _, _, _)| (entity, transform.translation))
        .collect();
    for (
        entity,
        transform,
        mut alignment_neighbors,
        mut cohesion_neighbors,
        mut separation_neighbors,
    ) in query.iter_mut()
    {
        let mut a = Vec::new();
        let mut c = Vec::new();
        let mut s = Vec::new();
        for (target, position) in positions.iter().filter(|(t, _)| t.id() != entity.id()) {
            let distance = transform
                .translation
                .truncate()
                .distance(position.truncate());
            if distance < boid_settings.alignment_distance {
                a.push(*target);
            }
            if distance < boid_settings.separation_distance {
                s.push(*target)
            } else if distance < boid_settings.cohesion_distance {
                c.push(*target);
            }
        }
        alignment_neighbors.entities = a;
        cohesion_neighbors.entities = c;
        separation_neighbors.entities = s;
    }
}

#[derive(Component, Default, Eq, PartialEq, Copy, Clone, Debug)]
pub enum BoidColor {
    #[default]
    None,
    Red,
    Green,
    Blue,
    Yellow,
}

impl BoidColor {
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Red,
            1 => Self::Green,
            2 => Self::Blue,
            3 => Self::Yellow,
            _ => Self::None,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            BoidColor::None => Color::WHITE,
            BoidColor::Red => Color::RED,
            BoidColor::Green => Color::GREEN,
            BoidColor::Blue => Color::BLUE,
            BoidColor::Yellow => Color::YELLOW,
        }
    }
}

pub fn update_boid_transforms(
    mut boid_query: Query<(&mut Transform, &mut BoidTurnDirectionInputs), With<Boid>>,
    time: Res<Time>,
    mut lines: ResMut<DebugLines>,
    boid_settings: Res<BoidSettings>,
) {
    for (mut transform, mut inputs) in boid_query.iter_mut() {
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
        inputs.add(r);

        transform.rotate_z(
            inputs.average() * boid_settings.max_turn_rate_per_second * time.delta_seconds(),
        );
        transform.translation += forward * time.delta_seconds() * boid_settings.speed;
        inputs.reset();
    }
}

pub fn calculate_cohesion_inputs(
    mut query: Query<
        (
            &Transform,
            &BoidNeighborsCohesion,
            &mut BoidTurnDirectionInputs,
        ),
        With<Boid>,
    >,
    transforms: Query<&Transform>,
    mut lines: ResMut<DebugLines>,
    boid_settings: Res<BoidSettings>,
) {
    if !boid_settings.cohesion_enabled {
        return;
    }
    // Rotate towards the average position of other boids within cohesion range.
    for (transform, neighbors, mut inputs) in query.iter_mut() {
        let average_position_of_near_boids: Vec2 = transforms
            .iter_many(&neighbors.entities)
            .map(|t| t.translation.truncate())
            .avg();
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

pub fn calculate_separation_inputs(
    mut query: Query<
        (
            &Transform,
            &BoidNeighborsSeparation,
            &mut BoidTurnDirectionInputs,
        ),
        With<Boid>,
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

pub fn calculate_alignment_inputs(
    mut query: Query<
        (
            &Transform,
            &BoidNeighborsAlignment,
            &mut BoidTurnDirectionInputs,
        ),
        With<Boid>,
    >,
    transforms: Query<&Transform>,
    mut lines: ResMut<DebugLines>,
    boid_settings: Res<BoidSettings>,
) {
    if !boid_settings.alignment_enabled {
        return;
    }
    for (transform, neighbors, mut inputs) in query.iter_mut() {
        let average: Vec2 = transforms
            .iter_many(&neighbors.entities)
            .map(|t| t.up().truncate())
            .avg()
            .normalize();
        if !average.is_nan() {
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

pub fn propagate_boid_color(
    mut query: Query<(Entity, &BoidNeighborsAlignment), With<BoidColor>>,
    mut boid_colors: Query<&mut BoidColor>,
) {
    for (entity, neighbors) in query.iter_mut() {
        if let Ok(color) = boid_colors.get(entity).cloned() {
            if color != BoidColor::None {
                // println!("{}", neighbors.entities.len());
                let mut iter = boid_colors.iter_many_mut(&neighbors.entities);
                while let Some(mut boid_color) = iter.fetch_next() {
                    if *boid_color != color {
                        let _ = mem::replace(&mut *boid_color, color);
                    }
                }
            }
        }
    }
}
