# src/ai/AGENTS.md - AI Behavior Module

## Module Overview

This module implements the **Flee AI** behavior system, which creates autonomous agents that flee from the player while blending with wandering behavior based on line-of-sight and distance.

**Key Files**:
- `flee.rs` - Complete AI behavior implementation
- `config.rs` - Centralized configuration constants

**Behavior Inspiration**: Rain World creature behavior (fleeing from player, wandering when safe)

---

## Core Concepts

### Behavior Blending

The AI blends between two behaviors:
1. **Flee**: Move away from player (when player is close and visible)
2. **Wander**: Random walk behavior (when player is far or not visible)

**Blend Factor** (`blend: f32`):
- `0.0` = Pure flee (red color)
- `1.0` = Pure wander (green color)
- `0.5` = 50/50 mix (orange color)

### Line-of-Sight (LOS) Detection

The AI checks if it can see the player using spatial partitioning for efficient raycasting:

```rust
let edges = spatial_grid.edges_along_ray(ai_pos, player_pos.position);
let mut can_see = true;
for edge in edges {
    if line_intersect(edge.start, edge.end, ai_pos, player_pos.position).is_some() {
        can_see = false;
        break;
    }
}
```

**Optimization**: Uses `SpatialGrid` to only test edges along the ray path, reducing complexity from O(all_edges) to O(nearby_edges).

**Caching**: LOS results are cached and invalidated when player or AI moves more than `LOS_CACHE_THRESHOLD` (5.0 pixels).

**If LOS blocked**: Blend gradually increases toward wander (`blend += delta_time`, clamped to handle lag spikes)
**If LOS clear**: Blend based on distance (closer = more flee)

### Directional Weighting

The AI tests 16 directions (22.5° apart) and weights each by:
1. **Dot product** with desired direction (flee or wander)
2. **Obstruction check** via raycasting

The highest-weighted unobstructed direction is selected.

---

## Component Structure

### FleeAI Component

```rust
#[derive(Component)]
pub struct FleeAI {
    pub dir_weights: [f32; 16],  // Weight for each of 16 directions
    pub wander_angle: f32,        // Persistent angle for smooth wandering
    pub color: Color,             // Visual indicator (red=flee, green=wander)
    pub blend: f32,               // 0.0=flee, 1.0=wander
}
```

**Initialization** (from `main.rs`):
```rust
FleeAI {
    dir_weights: [0.0; 16],
    wander_angle: PI / 2.0,
    color: Color::Srgba(css::GREEN),
    blend: 1.0,  // Start wandering
}
```

---

## System: s_flee_ai_movement

**Schedule**: `Update`, runs `.before(s_collision)`

**System Signature**:
```rust
pub fn s_flee_ai_movement(
    mut flee_ai_query: Query<(&mut Transform, &mut Physics, &mut FleeAI)>,
    player_pos: Res<PlayerPosition>,
    spatial_grid: Res<SpatialGrid>,
    mut gizmos: Gizmos,
    gizmos_visible: Res<GizmosVisible>,
    time: Res<Time>,
    mut cache: Local<SystemCache>,
)
```

**Note**: Uses `SpatialGrid` for efficient raycasting and `SystemCache` for LOS caching.

### Execution Flow

1. **LOS Check**: Raycast from AI to player using spatial grid (cached for performance)
2. **Blend Calculation** (with edge case handling):
   - No LOS: `blend = (blend + delta_time.min(0.1)).min(1.0)` (gradual wander, clamped for lag spikes)
   - LOS: `blend = ((distance - min_dist) / (max_dist - min_dist)).clamp(0.0, 1.0)`
   - Handles zero distance (player at exact AI position) and beyond detection range
3. **Color Update**: `color = Color::srgb(1.0 - blend, blend, 0.0)` (red→orange→green)
4. **Direction Calculation** (with edge case handling):
   - Flee direction: `-(player_pos - ai_pos).normalize()` (fallback if zero distance)
   - Wander direction: `get_wander_dir(...)` (handles zero velocity)
   - Blended direction: `flee_dir.lerp(wander_dir, blend)`
5. **Directional Weighting**: Calculate 16-direction weights via dot product
6. **Obstruction Check**: Test each direction (sorted by weight) using spatial grid
7. **Fallback Handling**: If all directions blocked, use blended direction (collision system handles penetration)
8. **Steering**: Apply steering behavior with blended max speed
9. **Movement**: Update velocity and position
10. **Collision Prep**: Set `prev_position` for collision system

### Configuration Constants

All AI parameters are centralized in `config.rs`:

**Speed Parameters**:
- `WANDER_MAX_SPEED: f32 = 3.0` - Maximum speed when wandering
- `FLEE_MAX_SPEED: f32 = 5.0` - Maximum speed when fleeing
- `STEERING_SCALE: f32 = 0.1` - Steering force scale for smooth acceleration

**Detection Parameters**:
- `AI_MAX_DETECTION_DISTANCE: f32 = 400.0` - Maximum detection range
- `AI_MIN_FLEE_DISTANCE: f32 = 200.0` - Minimum distance for pure flee behavior

**Obstruction Detection**:
- `AI_RAYCAST_DISTANCE: f32 = 100.0` - Raycast distance for direction checks

**Wandering Behavior**:
- `AI_WANDER_RADIUS: f32 = 50.0` - Radius of wander circle
- `AI_WANDER_DISPLACE_RANGE: f32 = 0.3` - Maximum angle displacement per frame

**Visualization**:
- `AI_VISUALIZATION_RADIUS: f32 = 30.0` - Debug visualization circle radius
- `AI_RENDER_RADIUS: f32 = 8.0` - AI entity render size
- `AI_DEBUG_CIRCLE_SIZE: f32 = 5.0` - Debug circle size

**Performance**:
- `LOS_CACHE_THRESHOLD: f32 = 5.0` - Distance threshold for cache invalidation

See `config.rs` for detailed documentation of each parameter.

---

## Wandering Behavior

### Function: get_wander_dir

**Purpose**: Generate smooth random walk direction

**Algorithm**:
1. Handle zero velocity edge case (use wander angle as fallback)
2. Project velocity vector forward (`AI_RAYCAST_DISTANCE` units)
3. Create circle around projected point (`AI_WANDER_RADIUS` radius)
4. Select random point on circle (displaced from current wander angle)
5. Return normalized direction toward that point (handles zero distance)

**Wander Angle Persistence**:
- `wander_angle` persists across frames
- Displaced by random amount each frame: `wander_angle += rng.random_range(-AI_WANDER_DISPLACE_RANGE..AI_WANDER_DISPLACE_RANGE)`
- Creates smooth, organic movement

**Edge Cases Handled**:
- Zero velocity: Uses wander angle direction as fallback
- Very small velocity: Normalizes safely using `normalize_or_zero()`
- Circle center at exact position: Returns zero vector (handled by normalization)

**Visualization** (when gizmos visible):
- Red dot: Projected velocity point
- White circle: Wander radius
- Green dot: Selected wander target

---

## Rendering: render_flee_ai

**Called from**: `main.rs` `s_render` system

**Purpose**: Draw AI visualization and debug info

**Rendering**:
- Colored circle (color indicates blend state)
- Surface normal line (if gizmos visible)
- 16-direction weight visualization (if gizmos visible)
  - Green lines: Positive weights
  - Red lines: Negative weights
  - Length proportional to weight magnitude

---

## Steering Behavior Pattern

The AI uses standard steering behavior:

```rust
let desired_velocity = direction * max_speed;
let steering = (desired_velocity - current_velocity) * STEERING_SCALE;
let acceleration = steering;
let new_velocity = current_velocity + acceleration;
```

**Steering Scale**: `0.1` creates smooth acceleration/deceleration (not instant direction changes)

---

## Plugin Structure

```rust
pub struct FleeAIPlugin;

impl Plugin for FleeAIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, s_flee_ai_movement.before(s_collision));
    }
}
```

**Registration**: Added in `main.rs` via `.add_plugins(FleeAIPlugin)`

---

## Performance Considerations

### Spatial Partitioning

The system uses `SpatialGrid` for efficient raycasting:
- **LOS check**: Only tests edges along ray path (O(nearby_edges) instead of O(all_edges))
- **Obstruction check**: 16 directions × O(nearby_edges_per_direction)
- **Grid-based**: Edges partitioned into cells matching level grid size

### Caching Optimizations

- **LOS Cache**: Results cached and invalidated when positions change significantly
- **Direction Vectors**: Pre-computed 16-direction vectors cached per system
- **Direction Indices**: Pre-computed array avoids per-frame allocation

### Direction Weight Calculation

16 directions tested each frame with pre-computed vectors (no per-frame cos/sin calculations).

---

## Debug Visualization

**Toggle**: Press `G` key (via `GizmosVisible` resource)

**Visual Elements**:
- AI circle: Color indicates blend (red=flee, green=wander)
- LOS line: Colored line to player (color matches blend)
- Direction weights: 16 lines showing weight vectors
- Wander circle: White circle showing wander target area
- Surface normal: White line showing collision normal

---

## Common Patterns

### Adding New AI Behavior

1. Add fields to `FleeAI` component
2. Modify `s_flee_ai_movement` to calculate new behavior
3. Blend with existing behaviors via `lerp()`
4. Update color/visualization if needed

### Modifying Flee/Wander Thresholds

Edit constants in `config.rs`:
```rust
pub const AI_MAX_DETECTION_DISTANCE: f32 = 400.0;  // Adjust for larger/smaller detection range
pub const AI_MIN_FLEE_DISTANCE: f32 = 200.0;  // Adjust for flee distance
```

All parameters are documented in `config.rs` with guidance on tuning.

### Changing Direction Count

Modify `dir_weights` array size and angle increment:
```rust
pub dir_weights: [f32; 16],  // Change 16 to desired count
// ...
angle += PI / 8.0;  // Change 8.0 to (count / 2.0)
```

---

## Edge Cases Handled

The implementation includes robust edge case handling:

- **Zero velocity**: Uses wander angle direction as fallback in `get_wander_dir`
- **All directions blocked**: Falls back to blended direction (collision system prevents penetration)
- **Player at exact AI position**: Distance calculation handles zero distance gracefully
- **Very high frame delta (lag spikes)**: Blend calculation clamps delta to 0.1 seconds
- **Very small distances**: All normalizations use `normalize_or_zero()` to avoid NaN
- **Zero distance in wander**: Returns zero vector safely

## Anti-Patterns to Avoid

- ❌ **Forgetting to update `prev_position`**: Must be set before movement (collision system depends on it)
- ❌ **Hardcoding parameters**: Use constants from `config.rs`
- ❌ **Not normalizing directions**: Always `.normalize_or_zero()` before use
- ❌ **Ignoring blend factor**: Always use blended max speed, not fixed speed
- ❌ **Testing all directions without sorting**: Sort by weight first, then test (performance)
- ❌ **Not handling zero velocity**: Always check `length_squared() > epsilon` before normalizing
- ❌ **Ignoring lag spikes**: Clamp delta time in blend calculations

---

## Quick Reference

### Find AI Systems
```bash
rg -n "s_flee_ai" src/ai/
```

### Find Steering Constants
```bash
rg -n "MAX_SPEED|STEERING_SCALE" src/ai/
```

### Find LOS Logic
```bash
rg -n "can_see_player|line_intersect" src/ai/
```

---

## Next Steps

- **Core systems**: See [../AGENTS.md](../AGENTS.md)
- **Root guide**: See [../../AGENTS.md](../../AGENTS.md)

