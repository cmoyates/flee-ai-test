# AI System Analysis

**Date**: 2024  
**System**: Flee AI Behavior (`src/ai/flee.rs`)  
**Bevy Version**: 0.16.1

---

## Executive Summary

This analysis identifies critical performance bottlenecks and logic issues in the AI fleeing/wandering system. The top 5 issues to address are:

1. **O(n²) Raycast Complexity**: Each AI performs 1 LOS raycast + 16 direction raycasts per frame, each checking all polygon edges. With N AIs and M polygons with E edges: **O(N × M × E × 17)** complexity.
2. **Per-Frame Array Allocation**: Direction indices array `[0..15]` is created and sorted every frame in hot path (lines 126-134).
3. **Redundant LOS Calculation**: LOS check iterates all polygons even after finding first intersection (early break exists but still checks all edges).
4. **Cornered AI Edge Case**: When all 16 directions are obstructed, AI velocity becomes `Vec2::ZERO`, causing AI to stop completely instead of backing up or finding alternative path.
5. **Monolithic System Function**: `s_flee_ai_movement` is 140 lines handling LOS, blending, weighting, obstruction checks, and steering - should be split into smaller systems.

---

## 1. Performance Bottlenecks

### 1.1 Raycast Complexity (HIGH IMPACT)

**Location**: `src/ai/flee.rs:54-76` (LOS check), `src/ai/flee.rs:145-162` (obstruction checks)

**Current Complexity**:
- **LOS Check**: O(M × E) per AI per frame
  - M = number of polygons
  - E = average edges per polygon
  - Early break on first intersection, but worst case checks all edges
  
- **Obstruction Checks**: O(16 × M × E) per AI per frame
  - 16 directions tested
  - Each direction checks all polygon edges
  - Sorted by weight, but worst case tests all 16 directions

**Total Per AI Per Frame**: Up to **17 × M × E** raycasts (1 LOS + 16 obstruction)

**Example**: With 10 polygons averaging 8 edges each:
- Per AI: 17 × 10 × 8 = **1,360 raycasts per frame**
- At 60 FPS: **81,600 raycasts per second per AI**
- With 5 AIs: **408,000 raycasts per second**

**Impact**: This is the primary performance bottleneck. Scales poorly with level complexity and AI count.

**Recommendations**:
1. **Spatial Partitioning**: Implement grid-based or quadtree spatial partitioning to reduce polygon checks
2. **Early Exit Optimization**: Cache recently checked directions/polygons
3. **Reduced Direction Sampling**: Consider 8 directions instead of 16 (22.5° → 45° increments)
4. **LOS Caching**: Cache LOS results for a few frames (player moves slowly relative to AI update rate)

**Priority**: **CRITICAL** - Address in Phase 2 (Performance Optimization)

---

### 1.2 Per-Frame Array Allocation (MEDIUM IMPACT)

**Location**: `src/ai/flee.rs:126-134`

**Issue**: 
```rust
let mut dir_indices: [usize; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
dir_indices.sort_by(|a, b| {
    ai_data.dir_weights[*b]
        .partial_cmp(&ai_data.dir_weights[*a])
        .unwrap_or(std::cmp::Ordering::Equal)
});
```

**Analysis**:
- Array is stack-allocated (good), but sorting happens every frame
- Sorting 16 elements is O(16 log 16) = O(64) operations per frame per AI
- `partial_cmp().unwrap_or()` adds overhead for NaN handling (unlikely but adds branch)

**Impact**: 
- Small per-AI cost, but accumulates with many AIs
- Unnecessary work if weights haven't changed significantly

**Recommendations**:
1. **Pre-compute sorted indices**: Only sort when weights change significantly (delta threshold)
2. **Use iterator-based approach**: `(0..16).max_by_key(|&i| ai_data.dir_weights[i])` for single best direction
3. **Partial sort**: Use `select_nth_unstable()` to get top-K directions instead of full sort

**Priority**: **MEDIUM** - Address in Phase 2

---

### 1.3 Redundant Calculations (LOW-MEDIUM IMPACT)

**Location**: `src/ai/flee.rs:78, 98, 109, 173`

**Issues**:

1. **Distance Calculation** (line 78):
   ```rust
   let distance = (player_pos.position - ai_transform.translation.xy()).length();
   ```
   - Used only for blend calculation
   - Could use `length_squared()` and compare squared thresholds (avoids sqrt)

2. **Flee Direction** (line 98):
   ```rust
   let flee_dir = -(player_pos.position - ai_transform.translation.xy()).normalize_or_zero();
   ```
   - Recalculates `player_pos.position - ai_transform.translation.xy()` (already computed for distance)
   - Could reuse distance vector

3. **Blended Direction** (line 109):
   ```rust
   let blended_dir = flee_dir.lerp(wander_dir, blend);
   ```
   - Lerp recalculates direction even when blend is 0.0 or 1.0 (pure flee/wander)

**Impact**: Minor performance gain, but improves code clarity.

**Recommendations**:
1. Use `length_squared()` and squared distance thresholds
2. Cache `ai_to_player` vector and reuse
3. Early return for pure flee/wander cases (skip lerp)

**Priority**: **LOW** - Address in Phase 2 if time permits

---

### 1.4 RNG Allocation (LOW IMPACT)

**Location**: `src/ai/flee.rs:221`

**Issue**:
```rust
let mut rng = rand::rng();
```

**Analysis**: `rand::rng()` creates a thread-local RNG. This is efficient, but called every frame for wander angle displacement.

**Impact**: Minimal - thread-local RNG is fast, but could be cached per-AI if needed.

**Recommendations**: 
- Consider caching RNG in `FleeAI` component if profiling shows bottleneck
- Current approach is acceptable for now

**Priority**: **LOW** - Monitor, optimize only if profiling shows issue

---

### 1.5 Spatial Partitioning Opportunity (HIGH IMPACT - FUTURE)

**Current State**: No spatial partitioning. All raycasts check all polygons.

**Opportunity**: 
- Level is grid-based (32x32 tiles from `level.json`)
- Could partition polygons into grid cells
- Raycast only checks polygons in cells along ray path

**Potential Improvement**: 
- Worst case: O(M × E) → O(sqrt(M) × E) for typical level layouts
- Best case: O(M × E) → O(1 × E) for localized checks

**Recommendations**:
- Implement grid-based spatial partitioning in `Level` resource
- Add `get_polygons_in_region()` method
- Update raycast functions to use spatial queries

**Priority**: **HIGH** - Address in Phase 3 (Advanced Optimization) if needed

---

## 2. Behavior Logic Analysis

### 2.1 Flee Direction Calculation (GOOD)

**Location**: `src/ai/flee.rs:98`

**Current Implementation**:
```rust
let flee_dir = -(player_pos.position - ai_transform.translation.xy()).normalize_or_zero();
```

**Analysis**: 
- ✅ Correct: Directly opposite of player direction
- ✅ Handles zero case with `normalize_or_zero()`
- ✅ Simple and effective for flee behavior

**Edge Cases**:
- When AI and player are at same position: `Vec2::ZERO` (handled correctly)
- When player is very far: Direction still accurate (no distance normalization needed)

**Verdict**: **OPTIMAL** - No changes needed

---

### 2.2 Wander Behavior Smoothness (GOOD)

**Location**: `src/ai/flee.rs:187-226`

**Analysis**:
- ✅ Persistent `wander_angle` creates smooth transitions
- ✅ Displacement range (`0.3` radians ≈ 17°) is reasonable
- ✅ Wander circle radius (`50.0`) provides good exploration area
- ✅ Velocity-based projection creates natural forward movement

**Potential Issues**:
- When velocity is zero, wander point is at AI position (circle radius 0)
- **Fix**: Use last non-zero velocity or default direction

**Recommendations**:
1. Handle zero velocity case in `get_wander_dir()`
2. Consider adding minimum wander circle size

**Priority**: **LOW** - Minor edge case

---

### 2.3 Blend Transition (GOOD WITH MINOR ISSUES)

**Location**: `src/ai/flee.rs:79-84`

**Current Implementation**:
```rust
let blend = if !can_see_player {
    (ai_data.blend + time.delta_secs()).min(1.0)  // Gradual increase toward wander
} else {
    ((distance - AI_MIN_FLEE_DISTANCE) / (AI_MAX_DETECTION_DISTANCE - AI_MIN_FLEE_DISTANCE))
        .max(0.0)
};
```

**Analysis**:
- ✅ LOS blocked: Gradual transition to wander (smooth)
- ✅ LOS clear: Distance-based blend (responsive)
- ⚠️ **Issue**: When LOS is regained, blend jumps immediately to distance-based value (no smoothing)
- ⚠️ **Issue**: Blend can oscillate rapidly when AI is near threshold distance

**Edge Cases**:
- Player at exactly `AI_MIN_FLEE_DISTANCE`: `blend = 0.0` (pure flee) ✅
- Player at exactly `AI_MAX_DETECTION_DISTANCE`: `blend = 1.0` (pure wander) ✅
- Player beyond max distance: `blend = 1.0` (pure wander) ✅

**Recommendations**:
1. **Smooth LOS regain**: When LOS is regained, lerp current blend toward distance-based blend over 0.1-0.2 seconds
2. **Hysteresis**: Add small dead zone to prevent oscillation at threshold boundaries
3. **Blend rate limiting**: Cap blend change rate per frame (e.g., max 0.1 per frame)

**Priority**: **MEDIUM** - Improves feel, address in Phase 3 (Polish)

---

### 2.4 Cornered AI Edge Case (CRITICAL LOGIC BUG)

**Location**: `src/ai/flee.rs:137-171`

**Current Behavior**:
```rust
let mut actual_dir = Vec2::ZERO;  // Default to zero if all directions blocked

for i in dir_indices {
    // ... test direction ...
    if !obstructed {
        actual_dir = dir;
        break;
    }
}

// If all directions blocked, actual_dir remains Vec2::ZERO
let desired_velocity = actual_dir * lerp(FLEE_MAX_SPEED, WANDER_MAX_SPEED, blend);
// desired_velocity = Vec2::ZERO
```

**Problem**: 
- When AI is cornered (all 16 directions blocked), `actual_dir = Vec2::ZERO`
- `desired_velocity = Vec2::ZERO`
- Steering reduces velocity toward zero
- **AI stops completely** instead of backing up or finding alternative path

**Expected Behavior**:
- AI should back away from walls
- Or find least-obstructed direction (even if partially blocked)
- Or use collision system to push AI away from walls

**Impact**: **HIGH** - AI can get stuck in corners

**Recommendations**:
1. **Fallback to least-obstructed direction**: Track obstruction distance, use direction with longest clear path
2. **Back away from walls**: Use collision normal to determine retreat direction
3. **Partial obstruction tolerance**: Allow movement in directions with minor obstructions (within threshold)

**Priority**: **CRITICAL** - Address in Phase 2 (Logic Fixes)

---

### 2.5 16-Direction Sampling (ADEQUATE)

**Location**: `src/ai/flee.rs:115-120`

**Current**: 16 directions = 22.5° increments

**Analysis**:
- ✅ Provides good directional coverage
- ✅ Balance between precision and performance
- ⚠️ May miss narrow passages between obstacles

**Alternatives**:
- **8 directions** (45°): Faster, but less precise
- **32 directions** (11.25°): More precise, but 2x raycasts

**Recommendations**:
- Keep 16 directions for now
- Consider making configurable per-AI if needed
- Add adaptive sampling: Use 8 directions normally, 16 when near obstacles

**Priority**: **LOW** - Current approach is adequate

---

### 2.6 Blend Calculation Edge Cases

**Location**: `src/ai/flee.rs:79-84`

**Edge Cases**:

1. **Player exactly at min distance** (`distance = 200.0`):
   - `blend = (200.0 - 200.0) / (400.0 - 200.0) = 0.0` ✅ Pure flee

2. **Player exactly at max distance** (`distance = 400.0`):
   - `blend = (400.0 - 200.0) / (400.0 - 200.0) = 1.0` ✅ Pure wander

3. **Player beyond max distance** (`distance > 400.0`):
   - `blend = 1.0` ✅ Pure wander (correct)

4. **Player closer than min distance** (`distance < 200.0`):
   - `blend = negative value → .max(0.0) = 0.0` ✅ Pure flee (correct)

5. **Division by zero** (if `AI_MAX_DETECTION_DISTANCE == AI_MIN_FLEE_DISTANCE`):
   - Current code would panic (division by zero)
   - **Fix**: Add assertion or handle case explicitly

**Recommendations**:
- Add assertion: `debug_assert!(AI_MAX_DETECTION_DISTANCE > AI_MIN_FLEE_DISTANCE)`
- Or handle equal case: `if AI_MAX_DETECTION_DISTANCE == AI_MIN_FLEE_DISTANCE { blend = 0.0 }`

**Priority**: **LOW** - Defensive programming

---

## 3. Code Quality Analysis

### 3.1 Bevy Anti-Patterns

#### 3.1.1 Monolithic System Function (MEDIUM)

**Location**: `src/ai/flee.rs:44-184`

**Issue**: `s_flee_ai_movement` is 140 lines handling:
- LOS detection
- Blend calculation
- Color update
- Direction calculation (flee + wander)
- Directional weighting
- Obstruction checks
- Steering calculation
- Movement update

**Bevy Best Practice**: Systems should be focused and single-purpose.

**Recommendations**:
Split into multiple systems:
1. `s_ai_los_check` - Calculate LOS and blend
2. `s_ai_direction_calculation` - Calculate flee/wander directions
3. `s_ai_obstruction_check` - Test directions for obstructions
4. `s_ai_steering` - Apply steering behavior
5. `s_ai_movement` - Update velocity and position

**System Ordering**:
```rust
.add_systems(Update, (
    s_ai_los_check,
    s_ai_direction_calculation.after(s_ai_los_check),
    s_ai_obstruction_check.after(s_ai_direction_calculation),
    s_ai_steering.after(s_ai_obstruction_check),
    s_ai_movement.after(s_ai_steering),
).chain().before(s_collision))
```

**Priority**: **MEDIUM** - Improves maintainability, address in Phase 4 (Refactoring)

---

#### 3.1.2 Gizmos Mutability (MINOR)

**Location**: `src/ai/flee.rs:48, 103`

**Issue**: `mut gizmos: Gizmos` passed to system, but gizmos are typically not mutated (they're a command buffer).

**Analysis**: This is actually correct for Bevy 0.16.1 - `Gizmos` requires `mut` for drawing operations. No change needed.

**Verdict**: **CORRECT** - No changes needed

---

### 3.2 Magic Numbers (ALREADY ADDRESSED)

**Location**: Various constants in `src/ai/flee.rs:13-26`

**Status**: ✅ Constants are already extracted:
- `WANDER_MAX_SPEED`, `FLEE_MAX_SPEED`
- `AI_MAX_DETECTION_DISTANCE`, `AI_MIN_FLEE_DISTANCE`
- `AI_RAYCAST_DISTANCE`, `AI_WANDER_RADIUS`, etc.

**Verdict**: **GOOD** - No changes needed

---

### 3.3 Floating-Point Precision Issues

**Location**: `src/ai/flee.rs:117, 140`

**Potential Issues**:

1. **Dot Product Precision** (line 117):
   ```rust
   let weight = dir.dot(blended_dir);
   ```
   - Dot product can be slightly > 1.0 or < -1.0 due to floating-point error
   - Currently not clamped (acceptable for weighting)

2. **Angle Calculation** (line 140):
   ```rust
   let angle = i as f32 * PI / 8.0;
   ```
   - Accumulated rounding error over 16 iterations
   - Last angle: `15 * PI / 8.0 = 5.890486...` (should be `2 * PI - PI/8`)
   - **Issue**: Last direction may not align perfectly with first direction

**Analysis**:
- Dot product precision: Acceptable (weights are relative, not absolute)
- Angle precision: Minor issue, but 16 directions should cover full circle

**Recommendations**:
1. Normalize dot product results: `weight.clamp(-1.0, 1.0)` (defensive)
2. Calculate angles more precisely: `let angle = (i as f32 / 16.0) * 2.0 * PI;` (ensures full circle)

**Priority**: **LOW** - Minor precision improvements

---

### 3.4 Error Handling

**Location**: `src/ai/flee.rs:132`

**Issue**:
```rust
.partial_cmp(&ai_data.dir_weights[*a])
    .unwrap_or(std::cmp::Ordering::Equal)
```

**Analysis**: 
- ✅ Handles NaN case gracefully
- ⚠️ `unwrap_or(Equal)` may hide bugs (NaN values shouldn't occur)

**Recommendations**:
- Add `debug_assert!(!dir_weights.iter().any(|w| w.is_nan()))` in debug builds
- Or use `unwrap_or_else(|| panic!("NaN in dir_weights"))` to catch bugs

**Priority**: **LOW** - Defensive programming

---

## 4. Steering Behavior Analysis

### 4.1 Steering Scale and Max Speed (GOOD)

**Location**: `src/ai/flee.rs:13-16, 173-174`

**Current Values**:
- `STEERING_SCALE = 0.1` (10% steering force per frame)
- `WANDER_MAX_SPEED = 3.0`
- `FLEE_MAX_SPEED = 5.0`

**Analysis**:
- ✅ Steering scale creates smooth acceleration (not instant direction changes)
- ✅ Flee speed > wander speed (appropriate for urgency)
- ✅ Blended speed: `lerp(5.0, 3.0, blend)` transitions smoothly

**Frame-by-Frame Behavior**:
- At 60 FPS: Steering applies 10% of difference per frame
- Time to reach max speed: ~10 frames (0.17 seconds) - feels responsive

**Recommendations**: 
- Current values are well-tuned
- Consider making configurable per-AI if needed for variety

**Verdict**: **OPTIMAL** - No changes needed

---

### 4.2 Acceleration/Deceleration Responsiveness (GOOD)

**Location**: `src/ai/flee.rs:174-178`

**Current Implementation**:
```rust
let steering = (desired_velocity - ai_physics.velocity) * STEERING_SCALE;
ai_physics.acceleration = steering;
let new_velocity = ai_physics.velocity + ai_physics.acceleration;
```

**Analysis**:
- ✅ Smooth acceleration toward desired velocity
- ✅ Deceleration when changing direction (steering reduces velocity)
- ✅ No velocity clamping (allows overshoot, then correction)

**Potential Issue**:
- No max acceleration limit (steering can be very large if velocity difference is large)
- **Impact**: AI can accelerate very quickly when switching from wander to flee

**Recommendations**:
- Consider clamping steering magnitude: `steering.clamp_length_max(max_acceleration)`
- Or use separate acceleration limits for flee vs wander

**Priority**: **LOW** - Current behavior feels good, optimize only if needed

---

### 4.3 Collision Resolution Interaction (GOOD)

**Location**: System ordering in `src/ai/flee.rs:32`

**Current Ordering**:
```rust
app.add_systems(Update, s_flee_ai_movement.before(s_collision));
```

**Flow**:
1. AI calculates desired movement
2. AI updates position based on steering
3. Collision system resolves penetrations
4. Collision system adjusts velocity (removes normal component)

**Analysis**:
- ✅ Correct ordering: AI moves first, then collision resolves
- ✅ Collision system removes velocity in normal direction (prevents sliding along walls)
- ⚠️ **Potential Issue**: AI steering may fight collision resolution (steering pushes into wall, collision pushes back)

**Behavior**:
- AI approaches wall → steering pushes forward
- Collision detects penetration → pushes AI back
- Collision removes velocity normal → AI slides along wall
- Next frame: Steering pushes forward again → cycle repeats

**Impact**: AI may "vibrate" against walls or get stuck pushing into obstacles

**Recommendations**:
1. **Obstruction-aware steering**: Reduce steering force when moving toward obstacles (use collision normal)
2. **Velocity damping near walls**: Reduce velocity when near walls (use collision normal from previous frame)
3. **Separate collision avoidance**: Add separate steering behavior for obstacle avoidance (beyond just raycast checks)

**Priority**: **MEDIUM** - Improves AI behavior near walls, address in Phase 3

---

## 5. Recommendations Summary

### Phase 2: Performance Optimization (CRITICAL)

1. **Implement spatial partitioning** for raycast optimization
   - Grid-based or quadtree
   - Reduce raycast complexity from O(M × E) to O(sqrt(M) × E)

2. **Fix cornered AI edge case**
   - Fallback to least-obstructed direction
   - Use collision normal for retreat direction

3. **Optimize direction sorting**
   - Only sort when weights change significantly
   - Or use iterator-based max selection

4. **Use `length_squared()` for distance checks**
   - Avoid sqrt in hot path
   - Compare squared distances

### Phase 3: Behavior Polish (MEDIUM)

1. **Smooth blend transitions**
   - Lerp blend when LOS is regained
   - Add hysteresis to prevent oscillation

2. **Improve collision interaction**
   - Reduce steering force when moving toward walls
   - Add velocity damping near obstacles

3. **Handle zero velocity in wander**
   - Use last non-zero velocity or default direction

### Phase 4: Code Quality (LOW)

1. **Split monolithic system**
   - Break into 5 focused systems
   - Improve maintainability

2. **Improve angle precision**
   - Use `(i / 16.0) * 2.0 * PI` for full circle coverage

3. **Add defensive assertions**
   - Check for NaN values
   - Assert distance thresholds are valid

---

## 6. Complexity Analysis Summary

### Current Complexity (Per AI Per Frame)

| Operation | Complexity | Notes |
|-----------|------------|-------|
| LOS Check | O(M × E) | M polygons, E edges avg |
| Direction Weighting | O(16) | Constant |
| Direction Sorting | O(16 log 16) | ~64 operations |
| Obstruction Checks | O(16 × M × E) | Worst case: all directions |
| Steering Calculation | O(1) | Constant |
| **Total** | **O(17 × M × E)** | Dominated by raycasts |

### Example Performance (10 polygons, 8 edges avg, 60 FPS)

- **Per AI**: 17 × 10 × 8 = 1,360 raycasts/frame = 81,600 raycasts/sec
- **5 AIs**: 408,000 raycasts/sec
- **10 AIs**: 816,000 raycasts/sec

### Optimized Complexity (With Spatial Partitioning)

| Operation | Complexity | Improvement |
|-----------|------------|-------------|
| LOS Check | O(sqrt(M) × E) | ~3x faster (typical) |
| Obstruction Checks | O(16 × sqrt(M) × E) | ~3x faster (typical) |
| **Total** | **O(17 × sqrt(M) × E)** | Significant improvement |

---

## 7. Verification Checklist

- [x] All source files have been read and analyzed
- [x] Performance complexity has been calculated (Big O notation)
- [x] At least 3 concrete performance improvements identified (spatial partitioning, array allocation, redundant calculations)
- [x] At least 3 concrete logic improvements identified (cornered AI, blend smoothing, collision interaction)
- [x] Recommendations are specific and actionable
- [x] Analysis document contains specific line numbers and code references
- [x] Recommendations are prioritized and feasible

---

## Conclusion

The AI system is functionally correct but has significant performance bottlenecks in raycast operations. The primary optimization target is spatial partitioning to reduce raycast complexity. Logic improvements focus on edge cases (cornered AI) and behavior polish (blend transitions, collision interaction).

The analysis provides a clear roadmap for optimization phases, with critical issues (performance, cornered AI) addressed first, followed by behavior polish and code quality improvements.

