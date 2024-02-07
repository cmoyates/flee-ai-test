mod ai;
mod collisions;
mod level;
mod utils;

use std::f32::consts::PI;

use ::bevy::prelude::*;
use ai::flee::{render_flee_ai, FleeAI, FleeAIPlugin};
use bevy::{app::AppExit, window::PresentMode};
use collisions::{s_collision, CollisionPlugin};
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
        .add_plugins(FleeAIPlugin)
        // Startup systems
        .add_systems(Startup, s_init)
        // Update systems
        .add_systems(Update, s_input)
        .add_systems(Update, (s_player_movement.before(s_collision),))
        .add_systems(Update, s_render.after(s_collision))
        .run();
}

pub const PLAYER_MAX_SPEED: f32 = 5.0;
pub const PLAYER_STEERING_SCALE: f32 = 0.1;

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

    // Escape to exit (if not WASM)
    #[cfg(not(target_arch = "wasm32"))]
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

pub fn s_render(
    mut gizmos: Gizmos,
    level: Res<Level>,
    mut player_query: Query<(&Transform, &Physics), With<Player>>,
    flee_ai_query: Query<(&Transform, &Physics, &FleeAI)>,
    gizmos_visible: Res<GizmosVisible>,
) {
    // Draw the level polygons
    for polygon in &level.polygons {
        gizmos.linestrip_2d(
            polygon.points.iter().cloned().collect::<Vec<Vec2>>(),
            polygon.color,
        );
    }

    // Draw the flee AI
    render_flee_ai(flee_ai_query, &mut gizmos, gizmos_visible.visible);

    // Draw the player
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
