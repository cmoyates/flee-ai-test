# src/AGENTS.md - Core Game Systems

## Module Overview

This directory contains the core game systems, ECS architecture, and main application setup. The project follows a **plugin-first** architecture where functionality is organized into Bevy plugins.

**Key Files:**
- `main.rs`: Application entry point, plugin registration, player systems, rendering
- `collisions.rs`: Physics-based collision detection plugin
- `level.rs`: Level data structures and generation from JSON
- `utils.rs`: Shared math utilities
- `ai/`: AI behavior module → [see ai/AGENTS.md](ai/AGENTS.md)

---

## ECS Architecture Patterns

### System Execution Order

The game uses explicit system ordering via `.before()` and `.after()`:

```rust
// From main.rs - Update schedule order:
1. s_input (no dependencies)
2. s_flee_ai_movement.before(s_collision)  // AI moves first
3. s_player_movement.before(s_collision)   // Player moves second
4. s_collision                              // Collision resolution
5. s_render.after(s_collision)              // Render after physics
```

**Critical**: Movement systems MUST run before collision detection. Rendering MUST run after collision to show final positions.

### Plugin Structure

All major systems are organized into plugins:

```rust
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, my_system.before(other_system));
    }
}
```

**Plugins in this project:**
- `CollisionPlugin` (`collisions.rs`) - Handles all entity collisions
- `FleeAIPlugin` (`ai/flee.rs`) - AI movement and behavior

### Resources

Resources are global state accessible to all systems:

```rust
#[derive(Resource)]
pub struct MyResource {
    pub data: Type,
}

// In systems:
fn my_system(resource: Res<MyResource>) {  // Read-only
    // Use resource.data
}

fn my_mut_system(mut resource: ResMut<MyResource>) {  // Mutable
    resource.data = new_value;
}
```

**Current Resources:**
- `Level`: Level geometry (polygons, grid size, dimensions)
- `InputDir`: Current player input direction (updated each frame)
- `PlayerPosition`: Player's current world position (updated by movement system)
- `GizmosVisible`: Debug visualization toggle

### Components

Components are data attached to entities:

```rust
#[derive(Component)]
pub struct MyComponent {
    pub field: Type,
}
```

**Current Components:**
- `Physics`: Movement data (velocity, acceleration, radius, normal, prev_position)
- `Player`: Marker component for player entity
- `FleeAI`: AI behavior data (see `ai/AGENTS.md`)

**Physics Component Pattern:**
All moving entities use the `Physics` component:
- `prev_position`: Used for collision resolution (where entity was last frame)
- `velocity`: Current movement velocity
- `acceleration`: Steering-based acceleration (applied each frame)
- `radius`: Circular collision radius
- `normal`: Surface normal when touching walls (prevents penetration)

---

## System Patterns

### Movement System Pattern

Movement systems follow this pattern:
1. Read input/desired direction
2. Calculate desired velocity
3. Apply steering (desired - current) * scale
4. Update velocity: `velocity += acceleration`
5. Update position: `transform.translation += velocity`

```rust
pub fn s_player_movement(
    input_dir: Res<InputDir>,
    mut player_query: Query<(&mut Transform, &mut Physics), With<Player>>,
    mut player_pos: ResMut<PlayerPosition>,
) {
    if let Ok((mut transform, mut physics)) = player_query.single_mut() {
        physics.prev_position = transform.translation.xy();
        
        let desired_velocity = input_dir.dir * MAX_SPEED;
        let steering = (desired_velocity - physics.velocity) * STEERING_SCALE;
        
        physics.acceleration = steering;
        physics.velocity += physics.acceleration;
        
        transform.translation += physics.velocity.extend(0.0);
        player_pos.position = transform.translation.xy();
    }
}
```

**Critical**: Always update `prev_position` BEFORE modifying `translation` - collision system depends on this!

### Input System Pattern

Input systems read `ButtonInput<KeyCode>` and update resources:

```rust
pub fn s_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut input_dir: ResMut<InputDir>,
    // ... other resources
) {
    let mut direction = Vec2::ZERO;
    
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    // ... other keys
    
    input_dir.dir = direction.normalize_or_zero();
}
```

### Rendering System Pattern

Rendering uses Bevy's `Gizmos` for 2D debug visualization:

```rust
pub fn s_render(
    mut gizmos: Gizmos,
    level: Res<Level>,
    gizmos_visible: Res<GizmosVisible>,
    // ... queries
) {
    // Draw level polygons
    for polygon in &level.polygons {
        gizmos.linestrip_2d(polygon.points.to_vec(), polygon.color);
    }
    
    // Draw entities (only if gizmos visible)
    if gizmos_visible.visible {
        // Debug visualization
    }
}
```

**Gizmo Toggle**: Use `GizmosVisible` resource to control debug rendering. Press `G` key to toggle.

---

## Module-Specific Guidance

### main.rs

**Responsibilities:**
- App initialization and plugin registration
- Resource initialization
- Entity spawning (player, AI, camera)
- Input handling
- Rendering orchestration

**Key Constants:**
- `PLAYER_MAX_SPEED: f32 = 5.0`
- `PLAYER_STEERING_SCALE: f32 = 0.1`

**Startup System (`s_init`):**
- Generates level polygons from JSON
- Spawns camera
- Spawns player entity with `Physics` + `Player` components
- Spawns AI entity with `Physics` + `FleeAI` components

**Platform-Specific Code:**
- Escape key exit disabled on WASM: `#[cfg(not(target_arch = "wasm32"))]`

### collisions.rs

**See**: `collisions.rs` for collision detection patterns. Key concepts:
- Line projection onto polygon edges
- Penetration resolution (maintain radius distance)
- Polygon containment detection (ray-casting method)
- Normal accumulation for velocity damping

**System**: `s_collision` runs after all movement systems, resolves penetrations, updates normals.

### level.rs

**Responsibilities:**
- Loading level data from embedded JSON (`include_bytes!()`)
- Converting tile grid to polygon geometry
- Optimizing collinear line segments
- Determining collision sides (winding order)

**Key Function**: `generate_level_polygons(grid_size: f32) -> (Vec<Polygon>, Vec2, Vec2)`

**Tile Types:**
- `0`: Empty space
- `1`: Full square
- `2-5`: Right triangles (4 orientations)
- `6-9`: Isosceles triangles (currently commented out)

**Polygon Structure:**
```rust
pub struct Polygon {
    pub points: Vec<Vec2>,        // Closed polygon vertices
    pub collision_side: f32,      // +1 or -1 (winding order)
    pub color: Color,              // Random color for visualization
}
```

### utils.rs

**Math Utilities:**
- `lerp(a: f32, b: f32, t: f32) -> f32`: Linear interpolation
- `line_intersect(...) -> Option<Vec2>`: Line-line intersection
- `cross_product(a: Vec2, b: Vec2) -> f32`: 2D cross product

**Usage**: These are pure functions, no side effects. Used by collision and AI systems.

---

## Common Query Patterns

### Single Entity Query
```rust
if let Ok((transform, physics)) = query.single_mut() {
    // Exactly one entity expected
}
```

### Iterate Over Entities
```rust
for (transform, physics) in query.iter_mut() {
    // Multiple entities
}
```

### Filtered Queries
```rust
Query<&Transform, (With<Player>, Without<FleeAI>)>
```

---

## Anti-Patterns to Avoid

- ❌ **Modifying `prev_position` after movement**: Always update it BEFORE changing `translation`
- ❌ **Running collision before movement**: Movement systems must run before `s_collision`
- ❌ **Using `ResMut` when `Res` suffices**: Prefer read-only access
- ❌ **Spawn entities in Update systems**: Use `Startup` or `Commands` in Update
- ❌ **Hardcoding system order**: Use `.before()`/`.after()` explicitly
- ❌ **Large queries without filters**: Use `With<T>`/`Without<T>` to narrow queries

---

## Quick Reference

### Find Systems
```bash
rg -n "pub fn s_" src/
```

### Find Resources
```bash
rg -n "derive\(Resource\)" src/
```

### Find Components
```bash
rg -n "derive\(Component\)" src/
```

### Find System Dependencies
```bash
rg -n "\.(before|after)\(" src/
```

---

## Next Steps

- **AI behavior**: See [ai/AGENTS.md](ai/AGENTS.md)
- **Asset format**: See [../assets/AGENTS.md](../assets/AGENTS.md)
- **Root guide**: See [../AGENTS.md](../AGENTS.md)

