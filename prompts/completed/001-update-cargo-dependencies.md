<objective>
Update all dependencies in `Cargo.toml` to their latest stable versions and fix any breaking changes in the codebase to ensure the project compiles and runs correctly.

This is a Bevy game engine project currently on version 0.12.1. Bevy has significant breaking API changes between major versions, so this task requires both dependency updates AND code migration.
</objective>

<context>
Project: flee-ai-test - A Rust game using Bevy ECS for AI flee behavior demonstration
Current dependencies in @Cargo.toml:
- bevy = "0.12.1" (outdated - likely 2+ major versions behind)
- rand = "0.8.5"
- serde = "1.0.196"
- serde_json = "1.0.112"

Key source files that use Bevy APIs:

- @src/main.rs - App setup, plugins, systems, input handling
- @src/ai/flee.rs - AI steering behaviors, physics
- @src/ai/mod.rs - AI module declarations
- @src/collisions.rs - Collision detection systems
- @src/level.rs - Level generation
- @src/utils.rs - Utility functions
  </context>

<research>
Before making changes, thoroughly research:

1. Use `btca ask -t bevy-docs -q "What is the latest stable Bevy version?"` to find current Bevy version
2. Use `btca ask -t bevy -q "What are the major breaking changes from Bevy 0.12 to latest?"` to understand migration requirements
3. Check crates.io or use `cargo search` to find latest versions of rand, serde, serde_json

Document findings before proceeding with updates.
</research>

<requirements>
1. Update `Cargo.toml` with latest stable versions of all dependencies
2. Review and update all source files for breaking API changes, focusing on:
   - Bevy's system scheduling API changes
   - Plugin registration changes
   - Query and Resource API changes
   - Gizmos API changes (used for debug visualization)
   - Input handling changes
   - Transform/physics changes
3. Preserve all existing functionality:
   - Player movement with arrow keys
   - Gizmo toggle functionality
   - Flee AI behavior
   - Collision detection
   - Level rendering
4. Maintain code organization and patterns already established in the project
</requirements>

<implementation>
Follow these steps in order:

1. **Research Phase**

   - Use btca to query Bevy documentation for latest version and migration guides
   - Document the specific API changes that affect this codebase

2. **Update Cargo.toml**

   - Update bevy to latest stable version
   - Update rand, serde, serde_json to latest versions
   - Run `cargo check` to see initial compilation errors

3. **Fix Breaking Changes**

   - Address each compilation error systematically
   - Consult Bevy migration guides for each breaking change
   - Update system registration (likely `.add_systems()` changes)
   - Update Query/Resource access patterns if needed
   - Fix any Gizmos API changes
   - Update input handling if API changed

4. **Verify Functionality**
   - Ensure project compiles with `cargo build`
   - Run the project to verify gameplay works as expected

WHY this order matters: Bevy's breaking changes cascade through the codebase. Understanding what changed first prevents repeated refactoring.
</implementation>

<output>
Modify existing files:
- `./Cargo.toml` - Updated dependency versions
- `./src/main.rs` - Fixed for new Bevy APIs
- `./src/ai/flee.rs` - Fixed for new Bevy APIs (if needed)
- `./src/collisions.rs` - Fixed for new Bevy APIs (if needed)
- `./src/level.rs` - Fixed for new Bevy APIs (if needed)
- Any other files requiring migration changes

Update `./CLAUDE.md` or workspace rules with new version information if significant API patterns change.
</output>

<verification>
Before declaring complete, verify:

1. `cargo build` compiles without errors or warnings
2. `cargo clippy` shows no significant lints
3. Run the application with `cargo run` and verify:
   - Window opens and displays correctly
   - Arrow keys move the player
   - AI entities exhibit flee behavior
   - Collision detection works
   - Gizmo toggle (if applicable) still functions

Report any functionality that couldn't be preserved and why.
</verification>

<success_criteria>

- All dependencies updated to latest stable versions
- Project compiles without errors
- All existing gameplay functionality preserved
- Code follows Bevy best practices for the new version
- No regression in performance or behavior
  </success_criteria>
