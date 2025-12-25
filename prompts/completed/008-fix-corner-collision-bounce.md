<objective>
Fix the bug where both AI and player entities bounce away when running directly into outward-facing (convex) corners in the level geometry.
</objective>

<context>
This is a Bevy 0.16.1 2D game with circular entities (player and AI) that collide with polygon-based level geometry. The collision system uses line projection to detect collisions with polygon edges.

The bug occurs specifically at **outward-facing corners** (convex vertices where two edges meet). When an entity moves directly toward such a corner, instead of sliding along the wall or stopping, the entity bounces away unexpectedly.

Key files:
- `./src/collisions.rs` - Contains the collision detection and resolution system
- `./src/level.rs` - Level polygon generation (for understanding polygon structure)
</context>

<root_cause>
The bug is in the `find_projection` function in `collisions.rs`. When the projection point falls outside the line segment (which happens near corners), the function artificially inflates the returned distance by adding `radius * 2.0`:

```rust
// Line 115-117 - When projection is past start
return (point_vec.length_squared() + radius * 2.0, start);

// Line 119-121 - When projection is past end  
return ((point - end).length_squared() + radius * 2.0, end);
```

This artificial inflation prevents proper corner collision detection because:
1. The inflated distance makes corners seem farther away than they are
2. Two adjacent edges both return inflated distances at the shared corner vertex
3. The collision system can't properly resolve collisions at corners

Additionally, the collision resolution in `s_collision` takes the adjustment with the largest absolute component (lines 79-84), which may not correctly handle corner cases where both X and Y adjustments are needed simultaneously.
</root_cause>

<requirements>
1. **Remove artificial distance inflation**: The `find_projection` function should return the actual distance to corner vertices (start/end points) without adding `radius * 2.0`

2. **Proper corner collision detection**: Entities should properly detect when they're colliding with a corner vertex, not just edge lines

3. **Smooth corner resolution**: When hitting a corner, entities should either:
   - Slide along the more aligned wall
   - Stop cleanly without bouncing backward
   - NOT bounce away from the corner

4. **Preserve existing behavior**: Edge collisions (not at corners) should continue working as they do now

5. **Test with both player and AI**: Both use the same Physics component and collision system, so the fix should work for both
</requirements>

<implementation>
Thoroughly analyze the collision system and consider multiple approaches:

**Approach 1 - Fix find_projection only:**
- Remove the `+ radius * 2.0` additions when returning corner distances
- This allows the collision system to properly detect corner proximity

**Approach 2 - Improve collision resolution:**
- Accumulate adjustments differently (vector addition vs component-wise max)
- Handle corner cases where multiple edges contribute to the collision

**Approach 3 - Hybrid:**
- Fix find_projection for accurate distances
- Also improve how multiple edge collisions are combined in s_collision

Consider:
- The `new_normal` calculation (lines 67-72) which accumulates normals from touching lines
- The velocity adjustment using accumulated normals (lines 96-97)
- Whether the current "take max component" approach (lines 79-84) is appropriate for corners
</implementation>

<verification>
Before completing, verify:
1. Run `cargo build` - must compile without errors
2. Run `cargo clippy --all-targets --all-features -D warnings` - must pass
3. Run the game with `cargo run` and test:
   - Move player directly into outward-facing corners - should NOT bounce away
   - Move player along walls - should still slide smoothly
   - Move player into inward-facing corners - should stop cleanly
   - Observe AI behavior at corners - should behave the same as player
</verification>

<success_criteria>
- Entities hitting outward-facing corners no longer bounce backward
- Entities slide along walls or stop cleanly at corners
- All existing edge collision behavior is preserved
- Code passes clippy with -D warnings
- No performance regression in collision detection
</success_criteria>

