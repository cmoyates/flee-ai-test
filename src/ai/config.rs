/// Centralized configuration for AI behavior parameters.
///
/// This module contains all tunable constants for the Flee AI system.
/// Parameters are organized by category for easy understanding and adjustment.
/// Maximum speed when wandering (pixels per frame).
///
/// Lower values create slower, more cautious wandering behavior.
/// Higher values make AI move faster when not fleeing.
pub const WANDER_MAX_SPEED: f32 = 3.0;

/// Maximum speed when fleeing from player (pixels per frame).
///
/// Should be higher than `WANDER_MAX_SPEED` to create urgency when fleeing.
/// Too high values may cause AI to overshoot corners or feel jittery.
pub const FLEE_MAX_SPEED: f32 = 5.0;

/// Steering force scale factor for smooth acceleration/deceleration.
///
/// Controls how quickly the AI changes direction:
/// - Lower values (0.05-0.1): Smooth, gradual turns
/// - Higher values (0.2+): Sharp, immediate direction changes
///
/// This creates realistic steering behavior instead of instant direction changes.
pub const STEERING_SCALE: f32 = 0.1;

// ============================================================================
// Detection and Flee Behavior Parameters
// ============================================================================

/// Maximum distance at which AI can detect the player (pixels).
///
/// Beyond this distance, AI will always wander regardless of line-of-sight.
/// Larger values create more reactive AI that responds from farther away.
pub const AI_MAX_DETECTION_DISTANCE: f32 = 400.0;

/// Minimum distance threshold for pure flee behavior (pixels).
///
/// When player is closer than this distance and visible, AI will flee (blend = 0.0).
/// Should be less than `AI_MAX_DETECTION_DISTANCE`.
/// Smaller values make AI less skittish, larger values make AI flee sooner.
pub const AI_MIN_FLEE_DISTANCE: f32 = 200.0;

// ============================================================================
// Obstruction Detection Parameters
// ============================================================================

/// Raycast distance for checking direction obstructions (pixels).
///
/// Used when testing 16 directions for valid movement paths.
/// Longer distances detect obstacles earlier but are more expensive.
/// Should be long enough to detect nearby walls but not so long as to be wasteful.
pub const AI_RAYCAST_DISTANCE: f32 = 100.0;

// ============================================================================
// Wandering Behavior Parameters
// ============================================================================

/// Radius of the wander circle (pixels).
///
/// The AI projects its velocity forward and creates a circle around that point.
/// Larger values create wider, more exploratory wandering paths.
/// Smaller values create tighter, more focused movement.
pub const AI_WANDER_RADIUS: f32 = 50.0;

/// Maximum angle displacement per frame for wander angle (radians).
///
/// Controls how quickly the wander target changes:
/// - Lower values (0.1-0.2): Smooth, gradual wander changes
/// - Higher values (0.4+): Erratic, unpredictable movement
///
/// Applied as: `wander_angle += random_range(-AI_WANDER_DISPLACE_RANGE..AI_WANDER_DISPLACE_RANGE)`
pub const AI_WANDER_DISPLACE_RANGE: f32 = 0.3;

// ============================================================================
// Visualization and Debug Parameters
// ============================================================================

/// Radius of the visualization circle showing direction weights (pixels).
///
/// Used when rendering debug visualization (press G to toggle).
/// Larger values make the direction weight visualization more visible.
pub const AI_VISUALIZATION_RADIUS: f32 = 30.0;

/// Render radius for AI entity circle (pixels).
///
/// The actual size of the AI agent as drawn on screen.
/// Should match the physics radius for accurate collision representation.
pub const AI_RENDER_RADIUS: f32 = 8.0;

/// Size of debug visualization circles (pixels).
///
/// Used for wander target visualization (red/green dots).
/// Small enough to not obscure the main visualization.
pub const AI_DEBUG_CIRCLE_SIZE: f32 = 5.0;

// ============================================================================
// Performance Optimization Parameters
// ============================================================================

/// Distance threshold for LOS cache invalidation (pixels).
///
/// If player or AI moves more than this distance, the line-of-sight cache
/// is invalidated and recalculated. Larger values reduce cache misses but
/// may cause stale LOS results.
pub const LOS_CACHE_THRESHOLD: f32 = 5.0;
