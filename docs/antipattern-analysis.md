# Antipattern Analysis and Fixes

This document catalogs antipatterns found in the codebase and the fixes applied.

## Summary

**Total Issues Found**: 14  
**Severity Breakdown**: High (2), Medium (7), Low (5)

**Note**: One issue (rand API) was initially flagged but found to be correct - rand 0.9.x uses `rand::rng()`.

---

## 1. Bevy/ECS Antipatterns

### 1.1 Unnecessary Tuple Wrapping in System Registration
**File**: `src/main.rs:37`  
**Severity**: Low  
**Issue**: System ordering wrapped in unnecessary tuple `(s_player_movement.before(s_collision),)`

**Fix**: Remove tuple wrapper, use direct system ordering  
**Rationale**: Cleaner syntax, no functional change but more idiomatic Bevy code

---

### 1.2 Unnecessary `.to_vec()` Allocation
**File**: `src/main.rs:196`  
**Severity**: Medium  
**Issue**: `polygon.points.to_vec()` creates unnecessary allocation when `polygon.points` is already a `Vec<Vec2>`

**Fix**: Use `&polygon.points` directly  
**Rationale**: Avoids unnecessary heap allocation in render loop

---

### 1.3 Using `iter_mut()` for Read-Only Access
**File**: `src/main.rs:205`  
**Severity**: Low  
**Issue**: `player_query.iter_mut()` used when only reading transform and physics data

**Fix**: Use `iter()` instead  
**Rationale**: More accurate intent, allows Bevy to optimize queries better

---

### 1.4 Using `.get().unwrap()` Instead of Iterators
**File**: `src/ai/flee.rs:47`, `src/collisions.rs:24`  
**Severity**: Medium  
**Issue**: Index-based access with `.unwrap()` instead of iterator-based access

**Fix**: Replace with iterator-based loops  
**Rationale**: Safer, more idiomatic Rust, avoids potential panics

---

## 2. Rust Antipatterns

### 2.1 Incorrect Rand API Usage
**File**: `src/ai/flee.rs:207`, `src/level.rs:15`  
**Severity**: Low  
**Issue**: Code already uses `rand::rng()` correctly for rand 0.9.x API (no change needed)

**Fix**: No change required - `rand::rng()` is correct for rand 0.9.x  
**Rationale**: Verified that `rand::rng()` is the correct API for rand 0.9.2

---

### 2.2 Typo: `diplace_range` Instead of `displace_range`
**File**: `src/ai/flee.rs:209`  
**Severity**: Low  
**Issue**: Variable name typo

**Fix**: Rename to `displace_range`  
**Rationale**: Improves code readability

---

### 2.3 Unnecessary Vec Allocation in Hot Path
**File**: `src/ai/flee.rs:119`  
**Severity**: Medium  
**Issue**: `Vec<usize>` allocation for direction indices that could use array or iterator

**Fix**: Use array `[0, 1, 2, ..., 15]` or iterator-based approach  
**Rationale**: Avoids heap allocation in frequently-called system

---

### 2.4 Using `.unwrap()` on `partial_cmp()` Result
**File**: `src/ai/flee.rs:125`  
**Severity**: Medium  
**Issue**: `.unwrap()` on `partial_cmp()` which can return `None` for NaN values

**Fix**: Use `unwrap_or(Ordering::Equal)` or handle NaN case explicitly  
**Rationale**: Safer handling of edge cases (though unlikely in this context)

---

### 2.5 Double `.unwrap()` on JSON Parsing
**File**: `src/level.rs:18`  
**Severity**: Medium  
**Issue**: Double `.unwrap()` on UTF-8 conversion and JSON parsing

**Fix**: Use `expect()` with descriptive messages or proper error handling  
**Rationale**: Better error messages if asset loading fails

---

### 2.6 Using `.unwrap()` on Option
**File**: `src/level.rs:444`  
**Severity**: Low  
**Issue**: `.unwrap()` on Option that was just checked

**Fix**: Use `if let Some(...)` pattern or `expect()` with message  
**Rationale**: More explicit about assumptions

---

### 2.7 Incorrect Flee Direction Calculation
**File**: `src/ai/flee.rs:91`  
**Severity**: High  
**Issue**: Using `prev_position` instead of current position for flee direction calculation

**Fix**: Use `ai_transform.translation.xy()` instead  
**Rationale**: Bug fix - flee direction should be from current position, not previous

---

## 3. Game Architecture Antipatterns

### 3.1 Magic Numbers: Entity Radii
**File**: `src/main.rs:100, 112`  
**Severity**: Medium  
**Issue**: Hardcoded radii values (12.0 for player, 8.0 for AI)

**Fix**: Extract to constants `PLAYER_RADIUS` and `AI_RADIUS`  
**Rationale**: Makes values configurable and self-documenting

---

### 3.2 Magic Numbers: AI Spawn Position
**File**: `src/main.rs:107`  
**Severity**: Low  
**Issue**: Hardcoded spawn position (100.0, 100.0)

**Fix**: Extract to constant `AI_SPAWN_POSITION`  
**Rationale**: Makes spawn position configurable

---

### 3.3 Magic Numbers: AI Behavior Thresholds
**File**: `src/ai/flee.rs:71-72`  
**Severity**: Medium  
**Issue**: Hardcoded distance thresholds (400.0, 200.0)

**Fix**: Extract to constants `AI_MAX_DETECTION_DISTANCE` and `AI_MIN_FLEE_DISTANCE`  
**Rationale**: Makes AI behavior tuning easier

---

### 3.4 Magic Numbers: Raycast and Wander Parameters
**File**: `src/ai/flee.rs:146, 192, 209`  
**Severity**: Low  
**Issue**: Hardcoded raycast distance (100.0), wander radius (50.0), displacement range (0.3)

**Fix**: Extract to constants  
**Rationale**: Makes AI behavior parameters configurable

---

### 3.5 Magic Numbers: Collision Detection Parameters
**File**: `src/collisions.rs:39, 60`  
**Severity**: Medium  
**Issue**: Hardcoded raycast direction (2.0, 1.0), distance (10000.0), touch threshold (0.5)

**Fix**: Extract to constants  
**Rationale**: Makes collision system parameters configurable and self-documenting

---

### 3.6 Magic Numbers: Visualization Parameters
**File**: `src/ai/flee.rs:202-204, 224, 237`  
**Severity**: Low  
**Issue**: Hardcoded visualization sizes (5.0, 30.0, 8.0)

**Fix**: Extract to constants (optional, low priority)  
**Rationale**: Makes debug visualization configurable

---

## Implementation Notes

All fixes follow the "minimal viable improvement" principle:
- No breaking changes to existing functionality
- No over-engineering or unnecessary abstractions
- Constants grouped logically near their usage
- Error handling improved where appropriate
- Code remains idiomatic and maintainable

