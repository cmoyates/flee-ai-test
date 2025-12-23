# src/ai/AGENTS.md - AI Behavior Module

## Module Overview

This module implements the **Flee AI** behavior system, which creates autonomous agents that flee from the player while blending with wandering behavior based on line-of-sight and distance.

**Key File**: `flee.rs` - Complete AI behavior implementation

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

The AI checks if it can see the player by raycasting through all level polygons:

```rust
let can_see_player = {
    let mut can_see = true;
    'polygon: for polygon in &level.polygons {
        for line in polygon.edges() {
            if line_intersect(line, ai_pos, player_pos).is_some() {
                can_see = false;
                break 'polygon;
            }
        }
    }
    can_see
};
```

**If LOS blocked**: Blend gradually increases toward wander (`blend += delta_time`)
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
    level: Res<Level>,
    mut gizmos: Gizmos,
    gizmos_visible: Res<GizmosVisible>,
    time: Res<Time>,
)
```

### Execution Flow

1. **LOS Check**: Raycast from AI to player through all polygons
2. **Blend Calculation**:
   - No LOS: `blend = (blend + delta_time).min(1.0)` (gradual wander)
   - LOS: `blend = ((distance - min_dist) / (max_dist - min_dist)).max(0.0)`
3. **Color Update**: `color = Color::srgb(1.0 - blend, blend, 0.0)` (red→orange→green)
4. **Direction Calculation**:
   - Flee direction: `-(player_pos - ai_pos).normalize()`
   - Wander direction: `get_wander_dir(...)`
   - Blended direction: `flee_dir.lerp(wander_dir, blend)`
5. **Directional Weighting**: Calculate 16-direction weights via dot product
6. **Obstruction Check**: Test each direction (sorted by weight) for collisions
7. **Steering**: Apply steering behavior with blended max speed
8. **Movement**: Update velocity and position

### Constants

```rust
const WANDER_MAX_SPEED: f32 = 3.0;
const FLEE_MAX_SPEED: f32 = 5.0;
pub const STEERING_SCALE: f32 = 0.1;

// Distance thresholds (in s_flee_ai_movement):
let max_distance = 400.0;  // Beyond this, always wander
let min_distance = 200.0;  // Closer than this, always flee
```

---

## Wandering Behavior

### Function: get_wander_dir

**Purpose**: Generate smooth random walk direction

**Algorithm**:
1. Project velocity vector forward (100 units)
2. Create circle around projected point (radius 50)
3. Select random point on circle (displaced from current wander angle)
4. Return direction toward that point

**Wander Angle Persistence**:
- `wander_angle` persists across frames
- Displaced by random amount each frame: `wander_angle += rng.gen_range(-0.3..0.3)`
- Creates smooth, organic movement

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

### Raycasting Optimization

The LOS check and obstruction checks perform many raycasts:
- LOS: 1 raycast per polygon edge
- Obstruction: 16 directions × N polygons × M edges per polygon

**Current Approach**: Brute force (acceptable for small levels)
**Future Optimization**: Spatial partitioning (grid/quadtree) for large levels

### Direction Weight Calculation

16 directions tested each frame. Consider caching if performance becomes an issue.

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

Edit constants in `s_flee_ai_movement`:
```rust
let max_distance = 400.0;  // Adjust for larger/smaller detection range
let min_distance = 200.0;  // Adjust for flee distance
```

### Changing Direction Count

Modify `dir_weights` array size and angle increment:
```rust
pub dir_weights: [f32; 16],  // Change 16 to desired count
// ...
angle += PI / 8.0;  // Change 8.0 to (count / 2.0)
```

---

## Anti-Patterns to Avoid

- ❌ **Forgetting to update `prev_position`**: Must be set before movement (collision system depends on it)
- ❌ **Hardcoding raycast distances**: Use configurable constants
- ❌ **Not normalizing directions**: Always `.normalize_or_zero()` before use
- ❌ **Ignoring blend factor**: Always use blended max speed, not fixed speed
- ❌ **Testing all directions without sorting**: Sort by weight first, then test (performance)

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

