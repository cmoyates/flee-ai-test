<objective>
Polish the AI system, tune parameters, handle edge cases, and ensure robust integration with other game systems.

This is the final phase that makes the existing AI production-ready.
</objective>

<context>
This is a Bevy 0.16.1 game with AI agents that flee from the player and wander when safe.

The current implementation features:
- 16-direction sampling with obstruction raycasting
- Line-of-sight detection for player visibility
- Blend factor between flee (red) and wander (green) behaviors
- Steering-based movement with configurable speeds

Read these files:
- `src/ai/flee.rs` - AI system to polish
- `src/ai/AGENTS.md` - Documentation to update
- `src/collisions.rs` - Ensure proper integration
- `src/main.rs` - Spawning and initialization

Reference the CLAUDE.md and AGENTS.md files for project conventions.
</context>

<requirements>
### 1. Parameter Organization
Create a centralized configuration for AI parameters:
- Extract the existing constants into a configuration struct:
  ```rust
  // Current constants in flee.rs:
  WANDER_MAX_SPEED, FLEE_MAX_SPEED, STEERING_SCALE,
  AI_MAX_DETECTION_DISTANCE, AI_MIN_FLEE_DISTANCE, AI_RAYCAST_DISTANCE,
  AI_WANDER_RADIUS, AI_WANDER_DISPLACE_RANGE, AI_VISUALIZATION_RADIUS,
  AI_RENDER_RADIUS, AI_DEBUG_CIRCLE_SIZE
  ```
- Consider making key parameters adjustable at runtime (Resource)
- Document what each parameter affects
- Group related parameters logically

### 2. Edge Case Hardening
Test and fix these scenarios:
- **All directions blocked**: Currently returns `Vec2::ZERO` - ensure this doesn't cause freezing
- **Zero velocity**: `get_wander_dir` normalizes velocity - handle zero case
- **Player at exact AI position**: Distance calculation edge case
- **Very high frame delta** (lag spike): Blend calculation uses `time.delta_secs()`
- **AI spawned at polygon edge**: Ensure initial state is valid

### 3. Integration Verification
Ensure proper interaction with:
- **Collision system**: AI's `prev_position` is set correctly (line 96)
- **Player movement**: Smooth flee dynamics when chased
- **Level geometry**: All polygon types handled in raycasts
- **Debug visualization**: Gizmos are accurate and helpful

### 4. Code Quality
- Add doc comments to public functions (`s_flee_ai_movement`, `get_wander_dir`, `render_flee_ai`)
- Add doc comment to `FleeAI` component explaining each field
- Ensure consistent naming conventions
- Remove any dead code or unused imports
- Consider splitting the large `s_flee_ai_movement` function if beneficial

### 5. Documentation Update
Update `src/ai/AGENTS.md` to ensure it accurately reflects:
- Current behavior descriptions
- All constants and their effects
- How the blend factor works
- Debug visualization guide
- Common tuning tips
</requirements>

<implementation>
**Option A: Config as constants (simpler)**
Keep constants but organize better with doc comments:
```rust
/// Maximum speed when fleeing from player (pixels/frame)
const FLEE_MAX_SPEED: f32 = 5.0;
```

**Option B: Config as Resource (more flexible)**
Create `src/ai/config.rs`:
```rust
#[derive(Resource)]
pub struct AIConfig {
    pub flee_max_speed: f32,
    pub wander_max_speed: f32,
    pub detection_distance: f32,
    // ... etc
}

impl Default for AIConfig {
    fn default() -> Self { ... }
}
```

Choose based on whether runtime adjustment is needed.

**Update `src/ai/flee.rs`**:
- Add safety checks for edge cases (zero velocity, all blocked, etc.)
- Add doc comments to all public items
- Ensure `actual_dir` fallback is sensible when all directions blocked

**Update `src/ai/AGENTS.md`**:
- Comprehensive documentation of the system
</implementation>

<constraints>
- Don't introduce new dependencies
- Preserve existing behavior (this is polish, not redesign)
- Keep any performance optimizations from phase 3 (if applied)
- Maintain backward compatibility
</constraints>

<output>
Create/modify files:
- `./src/ai/config.rs` - Centralized configuration (optional, if using Resource approach)
- `./src/ai/flee.rs` - Polished implementation with doc comments
- `./src/ai/mod.rs` - Updated exports (if adding config.rs)
- `./src/ai/AGENTS.md` - Updated documentation
- `./src/main.rs` - Config resource initialization (if using Resource approach)
</output>

<verification>
Complete verification checklist:
- [ ] `cargo build` - no errors
- [ ] `cargo build --release` - no errors  
- [ ] `cargo clippy --all-targets --all-features -D warnings` - no warnings
- [ ] `cargo fmt --check` - properly formatted
- [ ] `cargo test --all` - all tests pass (if any)
- [ ] Manual test: Normal gameplay feels good
- [ ] Manual test: AI flees when player approaches
- [ ] Manual test: AI wanders when player is hidden/far
- [ ] Manual test: Debug visualization works (press G)
- [ ] Manual test: AI behavior at level boundaries
- [ ] Documentation is complete and accurate
</verification>

<success_criteria>
- All AI parameters are documented
- Edge cases don't cause crashes or weird behavior
- Code has doc comments on public items
- Documentation reflects actual implementation
- No clippy warnings, code is formatted
- AI feels polished and production-ready
</success_criteria>

<final_checklist>
Before declaring this phase complete:
1. Play the game for several minutes - does AI feel good?
2. Try to break the AI - does it handle edge cases?
3. Read through all AI code - is it maintainable?
4. Read AGENTS.md - would a new developer understand the system?
5. Run full CI checks - everything passes?
</final_checklist>
