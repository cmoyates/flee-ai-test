<objective>
Improve the AI fleeing and wandering behavior logic to create smarter, more natural-feeling autonomous agents.

This phase focuses on behavioral improvements without major performance changes. Performance optimization comes in phase 3.
</objective>

<context>
This is a Bevy 0.16.1 game with AI agents that flee from the player and wander when safe.

Read these files before making changes:
- `src/ai/flee.rs` - Main AI behavior to improve
- `src/ai/AGENTS.md` - Current behavior documentation
- `./docs/ai-system-analysis.md` - Analysis from phase 1 (if available)

Reference the CLAUDE.md and AGENTS.md files for project conventions.
</context>

<requirements>
Implement the following behavior improvements:

### 1. Smarter Flee Behavior
- Replace simple "opposite direction" flee with **predictive fleeing**:
  - Consider player velocity/direction when choosing flee path
  - Prefer fleeing toward open areas rather than into corners
  - Add slight tangential component to avoid predictable straight-line fleeing
- Implement **flee path weighting** that considers:
  - Distance to nearest wall in each direction
  - Whether a direction leads to a dead end

### 2. Improved Wander Behavior  
- Smooth out the wander direction changes (reduce jitter)
- Implement **coherent noise** instead of pure random displacement:
  - Consider using a simple smoothed random walk
  - Maintain momentum in wander direction
- Add **area awareness**: prefer wandering toward open spaces
- Implement **boundary avoidance**: start turning away from walls before hitting them

### 3. Better Blend Transitions
- Implement **hysteresis** in the flee/wander transition to prevent oscillation
- Add a brief "alert" state when first detecting player
- Smooth the color transition to match behavior changes
- Consider player velocity in blend calculation (approaching player = more urgent)

### 4. Edge Case Handling
- Handle "cornered" state: when most directions are blocked, prioritize any escape
- Add "panic" behavior when player is very close with no good escape
- Ensure AI doesn't freeze when all directions are initially blocked
</requirements>

<implementation>
Modify `src/ai/flee.rs` following these guidelines:
- Keep the existing plugin structure
- Add new constants at the top for new parameters
- Consider splitting `s_flee_ai_movement` if it gets too large
- Update `FleeAI` component if new state is needed
- Maintain compatibility with the collision system

Do NOT focus on performance yet - that's phase 3.
</implementation>

<constraints>
- Maintain existing system ordering (before s_collision)
- Keep the 16-direction sampling approach for now
- Preserve debug visualization (gizmos)
- Don't change the Physics component structure
- Ensure AI still works with the existing level geometry
</constraints>

<output>
Modify files:
- `./src/ai/flee.rs` - Updated behavior logic
- `./src/ai/mod.rs` - If new modules are added

Update documentation:
- `./src/ai/AGENTS.md` - Reflect new behaviors
</output>

<verification>
Before completing:
- [ ] Run `cargo build` - no compilation errors
- [ ] Run `cargo clippy --all-targets --all-features -D warnings` - no warnings
- [ ] Run `cargo fmt` - code is formatted
- [ ] Test the game: AI should flee more intelligently
- [ ] Test the game: Wandering should be smoother
- [ ] Test edge case: AI cornered by player
- [ ] Debug visualization still works (press G)
</verification>

<success_criteria>
- AI flee behavior is less predictable and smarter
- Wander behavior is smoother with less jitter
- Blend transitions feel natural
- No regressions in existing functionality
- Code compiles without warnings
</success_criteria>

