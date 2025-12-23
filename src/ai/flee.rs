use std::f32::consts::PI;

use ::bevy::prelude::*;
use bevy::color::palettes::css;
use rand::Rng;

use crate::{
    collisions::s_collision,
    utils::{lerp, line_intersect},
    GizmosVisible, Level, Physics, PlayerPosition,
};

const WANDER_MAX_SPEED: f32 = 3.0;
const FLEE_MAX_SPEED: f32 = 5.0;

pub const STEERING_SCALE: f32 = 0.1;

// AI behavior parameters
const AI_MAX_DETECTION_DISTANCE: f32 = 400.0;
const AI_MIN_FLEE_DISTANCE: f32 = 200.0;
const AI_RAYCAST_DISTANCE: f32 = 100.0;
const AI_WANDER_RADIUS: f32 = 50.0;
const AI_WANDER_DISPLACE_RANGE: f32 = 0.3;
const AI_VISUALIZATION_RADIUS: f32 = 30.0;
const AI_RENDER_RADIUS: f32 = 8.0;
const AI_DEBUG_CIRCLE_SIZE: f32 = 5.0;

pub struct FleeAIPlugin;

impl Plugin for FleeAIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, s_flee_ai_movement.before(s_collision));
    }
}

#[derive(Component)]
pub struct FleeAI {
    pub dir_weights: [f32; 16],
    pub wander_angle: f32,
    pub color: Color,
    pub blend: f32,
}

pub fn s_flee_ai_movement(
    mut flee_ai_query: Query<(&mut Transform, &mut Physics, &mut FleeAI)>,
    player_pos: Res<PlayerPosition>,
    level: Res<Level>,
    mut gizmos: Gizmos,
    gizmos_visible: Res<GizmosVisible>,
    time: Res<Time>,
) {
    for (mut ai_transform, mut ai_physics, mut ai_data) in flee_ai_query.iter_mut() {
        // Check if the AI can see the player
        let can_see_player = {
            let mut can_see_player = true;
            'polygon: for polygon in &level.polygons {
                for line_index in 1..polygon.points.len() {
                    let start = polygon.points[line_index - 1];
                    let end = polygon.points[line_index];

                    let res = line_intersect(
                        start,
                        end,
                        ai_transform.translation.xy(),
                        player_pos.position,
                    );

                    if res.is_some() {
                        can_see_player = false;
                        break 'polygon;
                    }
                }
            }

            can_see_player
        };

        let distance = (player_pos.position - ai_transform.translation.xy()).length();
        let blend = if !can_see_player {
            (ai_data.blend + time.delta_secs()).min(1.0)
        } else {
            ((distance - AI_MIN_FLEE_DISTANCE) / (AI_MAX_DETECTION_DISTANCE - AI_MIN_FLEE_DISTANCE))
                .max(0.0)
        };

        ai_data.color = Color::srgb(1.0 - blend, blend, 0.0);

        if gizmos_visible.visible {
            gizmos.line_2d(
                ai_transform.translation.xy(),
                player_pos.position,
                ai_data.color,
            );
        }

        ai_physics.prev_position = ai_transform.translation.xy();

        let flee_dir = -(player_pos.position - ai_transform.translation.xy()).normalize_or_zero();

        let wander_dir = get_wander_dir(
            &ai_physics.velocity,
            &ai_transform.translation.xy(),
            &mut gizmos,
            &mut ai_data.wander_angle,
            gizmos_visible.visible,
            blend,
        );

        let blended_dir = flee_dir.lerp(wander_dir, blend);

        // Update dir weights
        {
            let mut angle: f32 = 0.0;

            for i in 0..16 {
                let dir = Vec2::new(angle.cos(), angle.sin());
                let weight = dir.dot(blended_dir);
                ai_data.dir_weights[i] = weight;
                angle += PI / 8.0;
            }
        }

        // Get the dir with the highest weight that's not obstructed
        let actual_dir = {
            // Get an array of ints 0 - 15
            let mut dir_indices: [usize; 16] =
                [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

            // Sort the indices by the weight (descending)
            dir_indices.sort_by(|a, b| {
                ai_data.dir_weights[*b]
                    .partial_cmp(&ai_data.dir_weights[*a])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Find the first non-obstructed dir
            let mut actual_dir = Vec2::ZERO;

            for i in dir_indices {
                let angle = i as f32 * PI / 8.0;
                let dir = Vec2::new(angle.cos(), angle.sin());

                let mut obstructed = false;

                for polygon in &level.polygons {
                    for line_index in 1..polygon.points.len() {
                        let start = polygon.points[line_index - 1];
                        let end = polygon.points[line_index];

                        let res = line_intersect(
                            start,
                            end,
                            ai_transform.translation.xy(),
                            ai_transform.translation.xy() + dir * AI_RAYCAST_DISTANCE,
                        );

                        if res.is_some() {
                            obstructed = true;
                            break;
                        }
                    }
                }

                if !obstructed {
                    actual_dir = dir;
                    break;
                }
            }

            actual_dir
        };

        let desired_velocity = actual_dir * lerp(FLEE_MAX_SPEED, WANDER_MAX_SPEED, blend);
        let steering = (desired_velocity - ai_physics.velocity) * STEERING_SCALE;

        ai_physics.acceleration = steering;
        let new_velocity = ai_physics.velocity + ai_physics.acceleration;
        ai_physics.velocity = new_velocity;

        ai_transform.translation.x += ai_physics.velocity.x;
        ai_transform.translation.y += ai_physics.velocity.y;

        ai_data.blend = blend;
    }
}

pub fn get_wander_dir(
    velocity: &Vec2,
    position: &Vec2,
    gizmos: &mut Gizmos,
    wander_angle: &mut f32,
    gizmos_visible: bool,
    blend: f32,
) -> Vec2 {
    let mut wander_point = *velocity;
    wander_point = wander_point.normalize_or_zero();
    wander_point *= AI_RAYCAST_DISTANCE;
    wander_point += *position;

    let velocity_angle = velocity.y.atan2(velocity.x);

    let x = AI_WANDER_RADIUS * (*wander_angle + velocity_angle).cos();
    let y = AI_WANDER_RADIUS * (*wander_angle + velocity_angle).sin();

    let circle_center = Vec2::new(x, y) + wander_point;

    if gizmos_visible {
        gizmos.circle_2d(
            wander_point,
            AI_DEBUG_CIRCLE_SIZE,
            css::RED.with_alpha(blend),
        );
        gizmos.circle_2d(wander_point, AI_WANDER_RADIUS, css::WHITE.with_alpha(blend));
        gizmos.circle_2d(
            circle_center,
            AI_DEBUG_CIRCLE_SIZE,
            css::GREEN.with_alpha(blend),
        );
    }

    let mut rng = rand::rng();

    *wander_angle += rng.random_range(-AI_WANDER_DISPLACE_RANGE..AI_WANDER_DISPLACE_RANGE);

    (circle_center - *position).normalize()
}

pub fn render_flee_ai(
    flee_ai_query: Query<(&Transform, &Physics, &FleeAI)>,
    gizmos: &mut Gizmos,
    gizmos_visible: bool,
) {
    for (flee_ai_transform, flee_ai_physics, flee_ai_data) in flee_ai_query.iter() {
        let flee_ai_pos = flee_ai_transform.translation.xy();

        gizmos.circle_2d(flee_ai_pos, AI_RENDER_RADIUS, flee_ai_data.color);

        // Draw the normal
        if gizmos_visible {
            gizmos.line_2d(
                flee_ai_pos,
                flee_ai_pos + flee_ai_physics.normal * flee_ai_physics.radius,
                css::WHITE,
            );
        }

        // Draw the dir weights
        if gizmos_visible {
            gizmos.circle_2d(
                flee_ai_pos,
                AI_VISUALIZATION_RADIUS,
                css::WHITE.with_alpha(0.2),
            );

            let mut angle: f32 = 0.0;

            // Get the max weight
            let max_weight: f32 = flee_ai_data
                .dir_weights
                .iter()
                .fold(0.0, |acc, &x| acc.max(x));

            for weight in flee_ai_data.dir_weights.iter() {
                let color = if *weight < 0.0 { css::RED } else { css::GREEN };

                let x = angle.cos() * AI_VISUALIZATION_RADIUS * weight.abs() / max_weight;
                let y = angle.sin() * AI_VISUALIZATION_RADIUS * weight.abs() / max_weight;
                gizmos.line_2d(flee_ai_pos, flee_ai_pos + Vec2::new(x, y), color);

                angle += std::f32::consts::PI / 8.0;
            }
        }
    }
}
