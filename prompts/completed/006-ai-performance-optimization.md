<objective>
Optimize the AI system performance to efficiently handle many AI agents simultaneously.

This phase focuses on reducing computational complexity and eliminating unnecessary work in the existing flee/wander AI implementation.
</objective>

<context>
This is a Bevy 0.16.1 game with AI agents that flee from the player and wander when safe.

The current implementation in `src/ai/flee.rs` has these performance characteristics:
- **LOS check**: Iterates all polygon edges to check if AI can see player
- **Direction sampling**: Tests 16 directions, each requiring raycast through all polygon edges
- **Per-frame complexity**: O(AI_count × polygon_count × edges_per_polygon × 17) raycasts

Read these files before making changes:
- `src/ai/flee.rs` - AI system to optimize (275 lines)
- `src/level.rs` - Level structure (for spatial partitioning)
- `src/utils.rs` - line_intersect function used for raycasts
- `src/ai/AGENTS.md` - Current documentation

Reference the CLAUDE.md and AGENTS.md files for project conventions.
</context>

<current_bottlenecks>
Based on the existing code:

1. **LOS Check (lines 54-76)**: Loops through ALL polygon edges for every AI every frame
2. **Direction Obstruction Check (lines 145-162)**: For each of 16 directions, loops through ALL polygon edges
3. **Per-frame allocations**: `rand::rng()` called every frame in `get_wander_dir`
4. **Sorting overhead**: `dir_indices.sort_by()` called every frame per AI
5. **No early termination**: Direction check continues even after finding obstruction in outer polygon loop
</current_bottlenecks>

<requirements>
Implement the following performance optimizations:

### 1. Spatial Partitioning for Raycasts
- Implement a **grid-based spatial hash** for polygon edges:
  - Store which edges intersect each grid cell
  - Only test edges in cells along the raycast path
  - Compute once at startup (level is static)
- Add the spatial structure as a Bevy Resource
- Target: Reduce raycast complexity from O(all_edges) to O(nearby_edges)

### 2. Reduce Redundant Calculations
- **Cache the LOS result** for a few frames if player hasn't moved significantly
- **Early-exit** direction testing: stop after finding first good direction in sorted order
- **Pre-compute direction vectors**: The 16 directions are constant, compute once
- Move `rand::rng()` creation outside of per-frame function or use thread-local

### 3. Optimize Direction Sampling
- **Skip opposite directions**: If direction N is blocked close, direction N+8 is likely clear
- **Adaptive early exit**: After finding 2-3 good directions, stop testing
- Pre-compute the `dir_indices` array (it's always [0..15])

### 4. Minor Optimizations
- Break out of inner edge loop immediately when obstruction found (currently breaks to next polygon, not next direction)
- Consider caching `ai_transform.translation.xy()` instead of calling multiple times
- Use `Vec2::from_angle()` instead of manual cos/sin for direction vectors
</requirements>

<implementation>
Create/modify files following these guidelines:

**New file `src/spatial.rs`** (recommended):
```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct SpatialGrid {
    cell_size: f32,
    grid: HashMap<(i32, i32), Vec<EdgeRef>>,
}

impl SpatialGrid {
    pub fn new(polygons: &[Polygon], cell_size: f32) -> Self { ... }
    pub fn edges_along_ray(&self, start: Vec2, end: Vec2) -> impl Iterator<Item = &EdgeRef> { ... }
}
```

**Modify `src/ai/flee.rs`**:
- Add `SpatialGrid` as system parameter
- Replace polygon iteration with spatial queries
- Add static direction vectors
- Optimize the hot loops

**Modify `src/main.rs`**:
- Create SpatialGrid resource after level generation
- Register in app

Follow Bevy patterns:
- New data structures should be Resources
- Consider using `Local<T>` for per-system caches
- Use system ordering to ensure spatial data is ready
</implementation>

<constraints>
- **Preserve existing behavior exactly** - this is pure optimization
- Maintain exact same visual output
- Keep debug visualization working (gizmos)
- Don't add external dependencies (use stdlib/Bevy only)
- Spatial structure should be generic enough for collision system to potentially use later
</constraints>

<output>
Create/modify files:
- `./src/spatial.rs` - Spatial partitioning (new file)
- `./src/ai/flee.rs` - Optimized AI system
- `./src/main.rs` - Register SpatialGrid resource, add module declaration
</output>

<verification>
Before completing:
- [ ] Run `cargo build --release` - no compilation errors
- [ ] Run `cargo clippy --all-targets --all-features -D warnings` - no warnings
- [ ] Run `cargo fmt` - code is formatted
- [ ] Visual behavior is identical to before optimization
- [ ] Test with debug gizmos (press G) - same visualization
- [ ] AI still flees correctly when player approaches
- [ ] AI still wanders when player is far/hidden
</verification>

<success_criteria>
- Raycast operations use spatial partitioning
- Direction vectors are pre-computed constants
- No unnecessary per-frame allocations
- Code compiles without warnings
- AI behavior unchanged from original
- Theoretical complexity reduction documented in code comments
</success_criteria>
