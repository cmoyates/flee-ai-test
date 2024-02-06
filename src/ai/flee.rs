use bevy::{gizmos::gizmos::Gizmos, math::Vec2, render::color::Color};
use rand::Rng;

pub fn wander(
    velocity: &Vec2,
    position: &Vec2,
    gizmos: &mut Gizmos,
    wander_angle: &mut f32,
    gizmos_visible: bool,
    blend: f32,
) -> Vec2 {
    let mut wander_point = velocity.clone();
    wander_point = wander_point.normalize_or_zero();
    wander_point *= 100.0;
    wander_point += position.clone();

    let wander_radius = 50.0;

    let velocity_angle = velocity.y.atan2(velocity.x);

    let x = wander_radius * (*wander_angle + velocity_angle).cos();
    let y = wander_radius * (*wander_angle + velocity_angle).sin();

    let circle_center = Vec2::new(x, y) + wander_point;

    if gizmos_visible {
        gizmos.circle_2d(wander_point, 5.0, Color::RED.with_a(blend));
        gizmos.circle_2d(wander_point, wander_radius, Color::WHITE.with_a(blend));
        gizmos.circle_2d(circle_center, 5.0, Color::GREEN.with_a(blend));
        gizmos.circle_2d(circle_center, 15.0, Color::WHITE.with_a(blend));
    }

    let mut rng = rand::thread_rng();

    let diplace_range: f32 = 0.3;

    *wander_angle += rng.gen_range(-diplace_range..diplace_range);

    return (circle_center - *position).normalize();
}
