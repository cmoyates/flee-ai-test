mod ai;
mod collisions;
mod level;

use std::f32::consts::PI;

use ::bevy::prelude::*;
use ai::flee::wander;
use bevy::{app::AppExit, window::PresentMode};
use collisions::{line_intersect, s_collision, CollisionPlugin};
use level::{generate_level_polygons, Polygon};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(InputDir { dir: Vec2::ZERO })
        .insert_resource(PlayerPosition {
            position: Vec2::ZERO,
        })
        .insert_resource(GizmosVisible { visible: false })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Flee AI Test".to_string(),
                present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CollisionPlugin)
        // Startup systems
        .add_systems(Startup, s_init)
        // Update systems
        .add_systems(Update, s_input)
        .add_systems(
            Update,
            (
                s_player_movement.before(s_collision),
                s_flee_ai_movement.before(s_collision),
            ),
        )
        .add_systems(Update, s_render.after(s_collision))
        .run();
}

#[derive(Resource)]
pub struct Level {
    pub polygons: Vec<Polygon>,
    pub grid_size: f32,
    pub size: Vec2,
    pub half_size: Vec2,
}

#[derive(Resource)]
pub struct InputDir {
    pub dir: Vec2,
}

#[derive(Resource)]
pub struct PlayerPosition {
    pub position: Vec2,
}

#[derive(Resource)]
pub struct GizmosVisible {
    pub visible: bool,
}

pub const PLAYER_MAX_SPEED: f32 = 5.0;
pub const PLAYER_STEERING_SCALE: f32 = 0.1;

pub const FLEE_AI_MAX_SPEED: f32 = 3.0;

#[derive(Component)]
pub struct Physics {
    pub prev_position: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub radius: f32,
    pub normal: Vec2,
}

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct FleeAI {
    pub dir_weights: [f32; 16],
    pub wander_angle: f32,
    pub color: Color,
    pub blend: f32,
}

pub fn s_init(mut commands: Commands) {
    let grid_size = 32.0;

    let (level_polygons, size, half_size) = generate_level_polygons(grid_size);

    commands.insert_resource(Level {
        polygons: level_polygons,
        grid_size,
        size,
        half_size,
    });

    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Physics {
            prev_position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            radius: 12.0,
            normal: Vec2::ZERO,
        },
        Player {},
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(100.0, 100.0, 0.0)),
        Physics {
            prev_position: Vec2::ZERO,
            velocity: Vec2::X,
            acceleration: Vec2::ZERO,
            radius: 8.0,
            normal: Vec2::ZERO,
        },
        FleeAI {
            dir_weights: [0.0; 16],
            wander_angle: PI / 2.0,
            color: Color::GREEN,
            blend: 1.0,
        },
    ));
}

pub fn s_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut input_dir: ResMut<InputDir>,
    mut gizmos_visible: ResMut<GizmosVisible>,
) {
    let mut direction = Vec2::ZERO;

    // Escape to exit
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }

    // Toggle gizmos
    if keyboard_input.just_pressed(KeyCode::G) {
        gizmos_visible.visible = !gizmos_visible.visible;
    }

    // Arrow keys to move
    if keyboard_input.pressed(KeyCode::Up) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        direction.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::Left) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        direction.x += 1.0;
    }

    // Normalize direction
    direction = direction.normalize_or_zero();

    // Set direction resource
    input_dir.dir = direction;
}

pub fn s_player_movement(
    input_dir: Res<InputDir>,
    mut player_query: Query<(&mut Transform, &mut Physics), With<Player>>,
    mut player_pos: ResMut<PlayerPosition>,
) {
    if let Ok((mut player_transform, mut player_physics)) = player_query.get_single_mut() {
        player_physics.prev_position = player_transform.translation.xy();

        let desired_velocity = input_dir.dir * PLAYER_MAX_SPEED;
        let steering = (desired_velocity - player_physics.velocity) * PLAYER_STEERING_SCALE;

        player_physics.acceleration = steering;
        let new_velocity = player_physics.velocity + player_physics.acceleration;
        player_physics.velocity = new_velocity;

        player_transform.translation.x += player_physics.velocity.x;
        player_transform.translation.y += player_physics.velocity.y;

        player_pos.position = player_transform.translation.xy();
    }
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
            'polygon: for i in 0..level.polygons.len() {
                let polygon = level.polygons.get(i).unwrap();

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
        let max_distance = 400.0;
        let min_distance = 200.0;
        let blend = if !can_see_player {
            (ai_data.blend + time.delta_seconds()).min(1.0)
        } else {
            ((distance - min_distance) / (max_distance - min_distance)).max(0.0)
        };

        ai_data.color = Color::rgb(1.0 - blend, blend, 0.0);

        if gizmos_visible.visible {
            gizmos.line_2d(
                ai_transform.translation.xy(),
                player_pos.position,
                ai_data.color,
            );
        }

        ai_physics.prev_position = ai_transform.translation.xy();

        let flee_dir = -(player_pos.position - ai_physics.prev_position).normalize_or_zero();

        let wander_dir = wander(
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
            let mut dir_indices: Vec<usize> = (0..16).collect();

            // Sort the indices by the weight (descending)
            dir_indices.sort_by(|a, b| {
                ai_data.dir_weights[*b]
                    .partial_cmp(&ai_data.dir_weights[*a])
                    .unwrap()
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
                            ai_transform.translation.xy() + dir * 100.0,
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

        let desired_velocity = actual_dir * lerp(PLAYER_MAX_SPEED, FLEE_AI_MAX_SPEED, blend);
        let steering = (desired_velocity - ai_physics.velocity) * PLAYER_STEERING_SCALE;

        ai_physics.acceleration = steering;
        let new_velocity = ai_physics.velocity + ai_physics.acceleration;
        ai_physics.velocity = new_velocity;

        ai_transform.translation.x += ai_physics.velocity.x;
        ai_transform.translation.y += ai_physics.velocity.y;

        ai_data.blend = blend;
    }
}

pub fn s_render(
    mut gizmos: Gizmos,
    level: Res<Level>,
    mut player_query: Query<(&Transform, &Physics), With<Player>>,
    mut flee_ai_query: Query<(&Transform, &Physics, &FleeAI)>,
    gizmos_visible: Res<GizmosVisible>,
) {
    // Draw the level polygons
    for polygon in &level.polygons {
        gizmos.linestrip_2d(
            polygon.points.iter().cloned().collect::<Vec<Vec2>>(),
            polygon.color,
        );
    }

    for (flee_ai_transform, flee_ai_physics, flee_ai_data) in flee_ai_query.iter_mut() {
        let flee_ai_pos = flee_ai_transform.translation.xy();

        gizmos.circle_2d(flee_ai_pos, 8.0, flee_ai_data.color);

        // Draw the normal
        if gizmos_visible.visible {
            gizmos.line_2d(
                flee_ai_pos,
                flee_ai_pos + flee_ai_physics.normal * flee_ai_physics.radius,
                Color::WHITE,
            );
        }

        // Draw the dir weights
        if gizmos_visible.visible {
            gizmos.circle_2d(flee_ai_pos, 30.0, Color::WHITE.with_a(0.2));

            let mut angle: f32 = 0.0;

            // Get the max weight
            let max_weight: f32 = flee_ai_data
                .dir_weights
                .iter()
                .fold(0.0, |acc, &x| acc.max(x));

            for weight in flee_ai_data.dir_weights.iter() {
                let color = if *weight < 0.0 {
                    Color::RED
                } else {
                    Color::GREEN
                };

                let x = angle.cos() * 30.0 * weight.abs() / max_weight;
                let y = angle.sin() * 30.0 * weight.abs() / max_weight;
                gizmos.line_2d(flee_ai_pos, flee_ai_pos + Vec2::new(x, y), color);

                angle += std::f32::consts::PI / 8.0;
            }
        }
    }

    for (player_transform, player_physics) in player_query.iter_mut() {
        gizmos.circle_2d(
            player_transform.translation.xy(),
            player_physics.radius,
            Color::WHITE,
        );

        // Draw the normal
        if gizmos_visible.visible {
            gizmos.line_2d(
                player_transform.translation.xy(),
                player_transform.translation.xy() + player_physics.normal * player_physics.radius,
                Color::WHITE,
            );
        }
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
