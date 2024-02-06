use bevy::{
    app::{App, Plugin, Update},
    ecs::system::{Query, Res},
    math::{Vec2, Vec3Swizzles},
    transform::components::Transform,
};

use crate::{Level, Physics};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, s_collision);
    }
}

pub fn s_collision(mut entity_query: Query<(&mut Transform, &mut Physics)>, level: Res<Level>) {
    for (mut transform, mut physics) in entity_query.iter_mut() {
        let mut adjustment = Vec2::ZERO;
        let mut new_normal = Vec2::ZERO;

        for polygon_index in 0..level.polygons.len() {
            let polygon = level.polygons.get(polygon_index).unwrap();

            let mut intersect_counter = 0;
            let mut colliding_with_polygon = false;

            for line_index in 1..polygon.points.len() {
                let start = polygon.points[line_index - 1];
                let end = polygon.points[line_index];

                // Intersection detection
                {
                    let intersection = line_intersect(
                        start,
                        end,
                        transform.translation.xy(),
                        transform.translation.xy() + Vec2::new(2.0, 1.0) * 10000.0,
                    );

                    if intersection.is_some() {
                        intersect_counter += 1;
                    }
                }

                let previous_side_of_line =
                    side_of_line_detection(start, end, physics.prev_position);

                if previous_side_of_line != polygon.collision_side {
                    continue;
                }

                let (distance_sq, projection) =
                    find_projection(start, end, transform.translation.xy(), physics.radius);

                let colliding_with_line = distance_sq <= physics.radius.powi(2);
                colliding_with_polygon = colliding_with_polygon || colliding_with_line;

                let touching_line = distance_sq <= (physics.radius + 0.5).powi(2);

                if touching_line {
                    let normal_dir = (transform.translation.xy() - projection).normalize_or_zero();

                    // Add the normal dir to the players new normal
                    new_normal -= normal_dir;
                }

                if colliding_with_line {
                    let mut delta = (transform.translation.xy() - projection).normalize_or_zero();

                    delta *= physics.radius - distance_sq.sqrt();

                    if delta.x.abs() > adjustment.x.abs() {
                        adjustment.x = delta.x;
                    }
                    if delta.y.abs() > adjustment.y.abs() {
                        adjustment.y = delta.y;
                    }
                }
            }
            if colliding_with_polygon && intersect_counter % 2 == 1 {
                transform.translation = physics.prev_position.extend(0.0);
            }
        }

        // Update the normal
        physics.normal = new_normal.normalize_or_zero();

        // Remove the players velocity in the direction of the normal
        let velocity_adjustment = physics.velocity.dot(new_normal) * new_normal;
        physics.velocity -= velocity_adjustment;

        // Update the players position
        transform.translation += adjustment.extend(0.0);
    }
}

pub fn find_projection(start: Vec2, end: Vec2, point: Vec2, radius: f32) -> (f32, Vec2) {
    let point_vec = point - start;
    let line_vec = end - start;

    let line_vec_normalized = line_vec.normalize();

    let dot = point_vec.dot(line_vec_normalized);

    let projection_point = line_vec_normalized * dot + start;

    // If the projection point is outside the line past start
    if (end - projection_point).length_squared() > (end - start).length_squared() {
        return (point_vec.length_squared() + radius * 2.0, start);
    }
    // If the projection point is outside the line past end
    if (projection_point - start).length_squared() > (end - start).length_squared() {
        return ((point - end).length_squared() + radius * 2.0, end);
    }

    let dist = (point - projection_point).length_squared();

    return (dist, projection_point);
}

pub fn side_of_line_detection(line_start: Vec2, line_end: Vec2, point: Vec2) -> f32 {
    let determinant = (line_end.x - line_start.x) * (point.y - line_start.y)
        - (line_end.y - line_start.y) * (point.x - line_start.x);

    return determinant.signum();
}

pub fn line_intersect(
    line_1_start: Vec2,
    line_1_end: Vec2,
    line_2_start: Vec2,
    line_2_end: Vec2,
) -> Option<Vec2> {
    let line_1 = line_1_end - line_1_start;
    let line_2 = line_2_end - line_2_start;
    let r_cross_s = cross_product(line_1, line_2);
    let a_to_c = line_2_start - line_1_start;
    let t = cross_product(a_to_c, line_2) / r_cross_s;
    let u = cross_product(a_to_c, line_1) / r_cross_s;

    if t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0 {
        Some(Vec2::new(
            line_1_start.x + t * line_1.x,
            line_1_start.y + t * line_1.y,
        ))
    } else {
        None
    }
}

pub fn cross_product(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}
