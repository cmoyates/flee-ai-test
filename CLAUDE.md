# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Bevy game engine project demonstrating AI flee behavior inspired by Rain World. The AI controls a creature that flees from the player, blending between fleeing and wandering behaviors based on line-of-sight and distance. The project can be compiled for both native and WASM targets.

## Build and Run Commands

### Native Development
```bash
# Build the project
cargo build

# Run the project
cargo run

# Build for release
cargo build --release
```

### WASM Target
The project is configured for WASM with wasm-server-runner in `.cargo/config.toml`:
```bash
# Build and run WASM version
cargo run --target wasm32-unknown-unknown
```

## Architecture

### Core Systems Flow
The application uses Bevy's ECS plugin architecture with systems running in a specific order:
1. Input handling (`s_input`)
2. AI movement (`s_flee_ai_movement`) → before collision
3. Player movement (`s_player_movement`) → before collision
4. Collision detection (`s_collision`)
5. Rendering (`s_render`) → after collision

### Module Structure
- `main.rs`: Core game loop, player controller, input handling, and rendering
- `ai/flee.rs`: Flee AI behavior implementation with directional weighting and wandering
- `collisions.rs`: Physics-based collision system using line projection and polygon containment
- `level.rs`: Level generation from JSON tile data with polygon optimization
- `utils.rs`: Math utilities (lerp, line intersection, cross product)

### Key Components

**Physics Component** (`main.rs:69-76`): All moving entities (player and AI) have:
- `prev_position`: Used for collision resolution to track where entity was before collision
- `velocity`, `acceleration`: For steering behavior implementation
- `radius`: For circular collision detection
- `normal`: Surface normal when touching walls, used to prevent penetration

**FleeAI Component** (`ai/flee.rs:26-31`):
- `dir_weights`: 16-direction weights for obstacle avoidance
- `wander_angle`: Persistent angle for smooth wandering behavior
- `blend`: Controls flee vs wander mix (0=flee, 1=wander)
- `color`: Visual indicator of current behavior state (red=flee, green=wander)

### AI Behavior System

The flee AI (`ai/flee.rs:33-176`) implements a hybrid behavior:

1. **Line-of-Sight Check**: Raycasts through level polygons to determine if AI can see player
2. **Blend Calculation**:
   - If no LOS: gradually blend toward wandering over time
   - If LOS: blend based on distance (flee when close, wander when far)
3. **Directional Weighting**:
   - Tests 16 directions around AI (22.5° apart)
   - Weights each by dot product with desired direction (flee or wander)
   - Sorts by weight and selects first unobstructed direction via raycasting
4. **Steering Behavior**: Uses desired velocity → steering → acceleration flow
5. **Wandering**: Circle-based random walk projected ahead of velocity vector

### Collision System

The collision system (`collisions.rs:18-97`) handles:
- **Line Projection**: Projects entity position onto polygon edges to find closest point
- **Penetration Resolution**: Adjusts position to maintain radius distance from walls
- **Polygon Containment**: Uses ray-polygon intersection counting (odd=inside) to detect when entity clips through
- **Normal Calculation**: Accumulates collision normals to remove velocity in penetration direction

### Level Generation

Level data (`level.rs:14-584`):
- Loads from embedded `assets/level.json` (2D tile array)
- Converts tiles to edge lines (squares=1, right triangles=2-5, commented isosceles=6-9)
- Optimizes by merging collinear adjacent lines
- Assembles lines into closed polygons
- Determines collision side using winding order and containment test
- Container polygon (outer boundary) has inverted collision side

## Platform Differences

The codebase handles platform-specific behavior:
- Escape key to exit is disabled on WASM (`main.rs:133-136`)
- WASM uses wasm-server-runner for live development

## Controls

- Arrow keys: Move player
- G: Toggle gizmos/debug visualization
- Escape: Exit (native only)

## Debug Visualization

When gizmos are enabled:
- White circle: Player with collision radius
- Green/Red/Orange circle: AI (color indicates blend state)
- White circle around AI: Shows 16 directional weight vectors
- Green/Red lines from AI center: Directional weights (green=positive, red=negative)
- Line to player: Shows LOS state (colored by blend)
- Wander circle and target point visualization
- Surface normals on entities
