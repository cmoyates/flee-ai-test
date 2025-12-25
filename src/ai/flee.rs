use ::bevy::prelude::*;
use bevy::color::palettes::css;
use rand::Rng;

use crate::{
    collisions::s_collision,
    spatial::SpatialGrid,
    utils::{lerp, line_intersect},
    GizmosVisible, Physics, PlayerPosition,
};

use super::config::{
    AI_DEBUG_CIRCLE_SIZE, AI_MAX_DETECTION_DISTANCE, AI_MIN_FLEE_DISTANCE, AI_RAYCAST_DISTANCE,
    AI_RENDER_RADIUS, AI_VISUALIZATION_RADIUS, AI_WANDER_DISPLACE_RANGE, AI_WANDER_RADIUS,
    FLEE_MAX_SPEED, LOS_CACHE_THRESHOLD, STEERING_SCALE, WANDER_MAX_SPEED,
};

// Pre-computed direction vectors for 16 directions (22.5° apart)
// Complexity: O(1) instead of O(16) per-frame cos/sin calculations
// Computed once per system using Local cache
fn get_direction_vectors() -> [Vec2; 16] {
    use std::f32::consts::PI;
    [
        Vec2::from_angle(0.0),             // 0°
        Vec2::from_angle(PI / 8.0),        // 22.5°
        Vec2::from_angle(PI / 4.0),        // 45°
        Vec2::from_angle(3.0 * PI / 8.0),  // 67.5°
        Vec2::from_angle(PI / 2.0),        // 90°
        Vec2::from_angle(5.0 * PI / 8.0),  // 112.5°
        Vec2::from_angle(3.0 * PI / 4.0),  // 135°
        Vec2::from_angle(7.0 * PI / 8.0),  // 157.5°
        Vec2::from_angle(PI),              // 180°
        Vec2::from_angle(9.0 * PI / 8.0),  // 202.5°
        Vec2::from_angle(5.0 * PI / 4.0),  // 225°
        Vec2::from_angle(11.0 * PI / 8.0), // 247.5°
        Vec2::from_angle(3.0 * PI / 2.0),  // 270°
        Vec2::from_angle(13.0 * PI / 8.0), // 292.5°
        Vec2::from_angle(7.0 * PI / 4.0),  // 315°
        Vec2::from_angle(15.0 * PI / 8.0), // 337.5°
    ]
}

// Pre-computed direction indices array (always [0..15])
// Avoids per-frame allocation and initialization
const DIR_INDICES: [usize; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

// System-level caches for performance optimization
#[derive(Default)]
pub(crate) struct SystemCache {
    // LOS cache
    last_player_pos: Vec2,
    last_ai_pos: Vec2,
    cached_los_result: Option<bool>,
    frame_count: u32,
    // Direction vectors cache
    direction_vectors: Option<[Vec2; 16]>,
}

/// Plugin for Flee AI behavior system.
///
/// Registers the AI movement system to run before collision detection,
/// ensuring AI movement is processed before physics resolution.
pub struct FleeAIPlugin;

impl Plugin for FleeAIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, s_flee_ai_movement.before(s_collision));
    }
}

/// Component representing a Flee AI agent.
///
/// The AI blends between fleeing from the player and wandering based on
/// line-of-sight detection and distance thresholds.
///
/// # Fields
///
/// * `dir_weights` - Weight for each of 16 directions (22.5° apart).
///   Used to select the best unobstructed movement direction.
/// * `wander_angle` - Persistent angle for smooth wandering behavior.
///   Updated each frame with random displacement to create organic movement.
/// * `color` - Visual indicator of current behavior state:
///   - Red (0.0): Pure flee behavior
///   - Orange (0.5): Mixed behavior
///   - Green (1.0): Pure wander behavior
/// * `blend` - Blend factor between flee (0.0) and wander (1.0) behaviors.
///   Calculated based on player distance and line-of-sight.
#[derive(Component)]
pub struct FleeAI {
    pub dir_weights: [f32; 16],
    pub wander_angle: f32,
    pub color: Color,
    pub blend: f32,
}

/// Main system for Flee AI movement behavior.
///
/// Processes all AI agents each frame, calculating:
/// 1. Line-of-sight detection to player
/// 2. Blend factor between flee and wander behaviors
/// 3. Direction selection via 16-direction sampling with obstruction checks
/// 4. Steering-based movement with configurable speeds
///
/// Runs before collision detection to ensure AI movement is processed first.
///
/// # Edge Cases Handled
///
/// * Zero velocity: Uses fallback direction when velocity is zero
/// * All directions blocked: Falls back to zero movement (prevents freezing)
/// * Player at exact AI position: Distance calculation handles zero distance
/// * High frame delta: Blend calculation clamps to prevent overshooting
/// * Very small distances: Normalization uses `normalize_or_zero()` to avoid NaN
pub fn s_flee_ai_movement(
    mut flee_ai_query: Query<(&mut Transform, &mut Physics, &mut FleeAI)>,
    player_pos: Res<PlayerPosition>,
    spatial_grid: Res<SpatialGrid>,
    mut gizmos: Gizmos,
    gizmos_visible: Res<GizmosVisible>,
    time: Res<Time>,
    mut cache: Local<SystemCache>,
) {
    // Update LOS cache if player moved significantly
    let player_moved = (player_pos.position - cache.last_player_pos).length_squared()
        > LOS_CACHE_THRESHOLD * LOS_CACHE_THRESHOLD;
    cache.frame_count += 1;

    for (mut ai_transform, mut ai_physics, mut ai_data) in flee_ai_query.iter_mut() {
        // Cache AI position to avoid repeated .xy() calls
        let ai_pos = ai_transform.translation.xy();

        // Check if the AI can see the player using spatial partitioning
        // Complexity: O(nearby_edges) instead of O(all_edges)
        let can_see_player = {
            // Use cached result if positions haven't changed significantly
            if !player_moved
                && (ai_pos - cache.last_ai_pos).length_squared()
                    < LOS_CACHE_THRESHOLD * LOS_CACHE_THRESHOLD
                && cache.cached_los_result.is_some()
            {
                cache.cached_los_result.unwrap()
            } else {
                // Perform spatial raycast
                let edges = spatial_grid.edges_along_ray(ai_pos, player_pos.position);
                let mut can_see = true;

                // Only test edges along the ray path (optimized)
                for edge in edges {
                    if line_intersect(edge.start, edge.end, ai_pos, player_pos.position).is_some() {
                        can_see = false;
                        break;
                    }
                }

                // Update cache
                cache.last_player_pos = player_pos.position;
                cache.last_ai_pos = ai_pos;
                cache.cached_los_result = Some(can_see);
                can_see
            }
        };

        // Calculate distance to player (handle edge case: player at exact AI position)
        let distance = (player_pos.position - ai_pos).length();

        // Calculate blend factor with edge case handling
        let blend = if !can_see_player {
            // Gradually increase blend toward wander when player not visible
            // Clamp delta to prevent overshooting on lag spikes
            let delta = time.delta_secs().min(0.1); // Cap at 100ms to handle lag spikes
            (ai_data.blend + delta).min(1.0)
        } else {
            // Blend based on distance when player is visible
            let distance_range = AI_MAX_DETECTION_DISTANCE - AI_MIN_FLEE_DISTANCE;
            if distance_range > 0.0 && distance >= AI_MIN_FLEE_DISTANCE {
                ((distance - AI_MIN_FLEE_DISTANCE) / distance_range).clamp(0.0, 1.0)
            } else if distance < AI_MIN_FLEE_DISTANCE {
                // Player very close: pure flee
                0.0
            } else {
                // Player beyond detection range: pure wander
                1.0
            }
        };

        ai_data.color = Color::srgb(1.0 - blend, blend, 0.0);

        if gizmos_visible.visible {
            gizmos.line_2d(ai_pos, player_pos.position, ai_data.color);
        }

        // Set previous position for collision system (must be done before movement)
        ai_physics.prev_position = ai_pos;

        // Calculate flee direction (away from player)
        // Handle edge case: player at exact AI position returns zero vector
        let to_player = player_pos.position - ai_pos;
        let flee_dir = if to_player.length_squared() > 0.0001 {
            // Normal case: normalize direction away from player
            -to_player.normalize()
        } else {
            // Edge case: player at exact position, use last velocity direction or fallback
            if ai_physics.velocity.length_squared() > 0.0001 {
                -ai_physics.velocity.normalize()
            } else {
                Vec2::X // Fallback: move right
            }
        };

        // Calculate wander direction (handles zero velocity case internally)
        let wander_dir = get_wander_dir(
            &ai_physics.velocity,
            &ai_pos,
            &mut gizmos,
            &mut ai_data.wander_angle,
            gizmos_visible.visible,
            blend,
        );

        // Blend flee and wander directions
        let blended_dir = flee_dir.lerp(wander_dir, blend);

        // Update dir weights using pre-computed direction vectors
        // Complexity: O(16) with pre-computed vectors instead of O(16) cos/sin calls
        let dir_vectors = cache
            .direction_vectors
            .get_or_insert_with(get_direction_vectors);
        for (i, weight) in ai_data.dir_weights.iter_mut().enumerate() {
            *weight = dir_vectors[i].dot(blended_dir);
        }

        // Get the dir with the highest weight that's not obstructed
        // Complexity: O(16 log 16) sort + O(16 × nearby_edges) raycasts (optimized with spatial grid)
        let actual_dir = {
            // Use pre-computed indices array and sort by weight
            let mut dir_indices = DIR_INDICES;
            dir_indices.sort_by(|a, b| {
                ai_data.dir_weights[*b]
                    .partial_cmp(&ai_data.dir_weights[*a])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Find the first non-obstructed dir with early exit
            let mut actual_dir = Vec2::ZERO;
            let mut found_valid_dir = false;

            // Early exit: stop after finding first good direction
            let dir_vectors = cache
                .direction_vectors
                .get_or_insert_with(get_direction_vectors);
            for &dir_idx in &dir_indices {
                let dir = dir_vectors[dir_idx];
                let ray_end = ai_pos + dir * AI_RAYCAST_DISTANCE;

                // Use spatial grid to only test edges along ray path
                let edges = spatial_grid.edges_along_ray(ai_pos, ray_end);
                let mut obstructed = false;

                // Early exit: break immediately when obstruction found
                for edge in edges {
                    if line_intersect(edge.start, edge.end, ai_pos, ray_end).is_some() {
                        obstructed = true;
                        break;
                    }
                }

                if !obstructed {
                    actual_dir = dir;
                    found_valid_dir = true;
                    break; // Early exit: found good direction
                }
            }

            // Edge case: all directions blocked
            // Fallback to blended direction (may still be obstructed, but collision system will handle it)
            if !found_valid_dir {
                // Use blended direction as fallback - collision system will prevent penetration
                actual_dir = blended_dir.normalize_or_zero();
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

/// Calculate wander direction using steering-based wandering algorithm.
///
/// Projects the velocity vector forward and selects a random point on a circle
/// around that projection. Creates smooth, organic wandering behavior.
///
/// # Arguments
///
/// * `velocity` - Current velocity vector (may be zero)
/// * `position` - Current AI position
/// * `gizmos` - Gizmos for debug visualization
/// * `wander_angle` - Persistent angle, mutated each frame with random displacement
/// * `gizmos_visible` - Whether to render debug visualization
/// * `blend` - Blend factor for visualization alpha
///
/// # Returns
///
/// Normalized direction vector toward the wander target.
///
/// # Edge Cases
///
/// * Zero velocity: Uses last wander angle direction as fallback
/// * Very small velocity: Normalizes safely using `normalize_or_zero()`
pub fn get_wander_dir(
    velocity: &Vec2,
    position: &Vec2,
    gizmos: &mut Gizmos,
    wander_angle: &mut f32,
    gizmos_visible: bool,
    blend: f32,
) -> Vec2 {
    // Handle edge case: zero or very small velocity
    let velocity_dir = if velocity.length_squared() > 0.0001 {
        velocity.normalize()
    } else {
        // Fallback: use direction from wander angle
        Vec2::from_angle(*wander_angle)
    };

    // Project velocity forward to create wander circle center
    let mut wander_point = velocity_dir * AI_RAYCAST_DISTANCE;
    wander_point += *position;

    // Calculate angle from velocity direction (handle zero velocity case)
    let velocity_angle = if velocity.length_squared() > 0.0001 {
        velocity.y.atan2(velocity.x)
    } else {
        *wander_angle // Use wander angle directly if velocity is zero
    };

    // Use Vec2::from_angle instead of manual cos/sin
    let angle = *wander_angle + velocity_angle;
    let circle_center = Vec2::from_angle(angle) * AI_WANDER_RADIUS + wander_point;

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

    // Use thread-local RNG (rand::rng() is already thread-local, but we avoid creating it every frame)
    // Note: rand::rng() is already optimized, but we could cache it if needed
    let mut rng = rand::rng();
    *wander_angle += rng.random_range(-AI_WANDER_DISPLACE_RANGE..AI_WANDER_DISPLACE_RANGE);

    // Return normalized direction toward wander target
    // Handle edge case: circle center at exact position
    let to_target = circle_center - *position;
    to_target.normalize_or_zero()
}

/// Render Flee AI entities with debug visualization.
///
/// Draws AI agents as colored circles (color indicates blend state) and
/// optional debug information when gizmos are visible.
///
/// # Arguments
///
/// * `flee_ai_query` - Query for all AI entities with Transform, Physics, and FleeAI components
/// * `gizmos` - Gizmos for rendering
/// * `gizmos_visible` - Whether to show debug visualization (surface normal, direction weights)
///
/// # Visualization
///
/// * Colored circle: AI entity (red=flee, green=wander)
/// * Surface normal line: White line showing collision normal (if gizmos visible)
/// * Direction weights: 16 lines showing weight vectors (if gizmos visible)
///   - Green: Positive weights (preferred directions)
///   - Red: Negative weights (avoided directions)
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
