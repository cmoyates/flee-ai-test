<objective>
Thoroughly analyze this small Bevy ECS game codebase for antipatterns and implement appropriate design pattern improvements.

The goal is to improve code quality and maintainability while respecting the small scope of this project. This is NOT a large enterprise application - avoid over-engineering. Focus on practical improvements that make the code cleaner, more idiomatic, and easier to extend.
</objective>

<context>
This is a small Rust game built with Bevy 0.16.1 featuring:
- A player entity with movement controls
- AI entities with flee/wander steering behaviors  
- Collision detection against level geometry
- 2D rendering with gizmos

Project structure:

- `src/main.rs` - Entry point, player systems, input handling, rendering
- `src/ai/flee.rs` - FleeAI component and steering behavior systems
- `src/ai/mod.rs` - AI module declarations
- `src/collisions.rs` - Collision detection plugin and systems
- `src/level.rs` - Level generation from JSON
- `src/utils.rs` - Math utilities

First, read CLAUDE.md and AGENTS.md for project conventions, then examine all source files.
</context>

<analysis_requirements>
Deeply analyze the codebase for these categories of antipatterns:

1. **Bevy/ECS Antipatterns**:

   - Systems doing too much (violating single responsibility)
   - Queries that are too broad or could use better filters
   - Missing or incorrect system ordering dependencies
   - Improper use of `Res` vs `ResMut`
   - Components that should be resources (or vice versa)
   - Missing component/resource derives that would be useful

2. **Rust Antipatterns**:

   - Unnecessary clones or allocations in hot paths
   - Missing lifetime elision opportunities
   - Overly complex type signatures
   - Code that could use more idiomatic Rust patterns
   - Public API inconsistencies

3. **Game Architecture Antipatterns**:

   - Magic numbers that should be constants
   - Hardcoded values that should be configurable
   - Tight coupling between systems that should be decoupled
   - Missing abstraction boundaries
   - Repeated code that could be shared

4. **What NOT to "fix"**:
   - Don't add unnecessary abstractions for a small game
   - Don't create complex generic systems for simple behaviors
   - Don't add configuration files for values that won't change
   - Don't split working code just to have more files
   - Don't add traits/interfaces unless there's actual polymorphism needed
     </analysis_requirements>

<implementation_guidelines>
When implementing fixes, follow these principles:

1. **Minimal viable improvement**: Make the smallest change that fixes the antipattern
2. **Preserve behavior**: All existing functionality must work identically
3. **Follow existing conventions**: Match the code style already in the project
4. **Document significant changes**: Add brief comments explaining non-obvious improvements

Appropriate patterns for a small ECS game:

- Constants for magic numbers (in a `const` block or module)
- Clear system naming that indicates purpose
- Query filters to improve performance and clarity
- Proper system ordering with `.before()` / `.after()`
- Separating concerns into focused systems (but not to excess)
- Using Bevy's built-in types appropriately (Timer, Vec2, etc.)

Patterns to AVOID adding:

- Event-driven architecture (unless clearly needed)
- Complex state machines (unless clearly needed)
- Factory patterns or builders (overkill for small games)
- Multiple levels of abstraction
- Generic/trait-based systems unless polymorphism exists
  </implementation_guidelines>

<output>
After analysis, create a summary of findings and implement the fixes:

1. **First**, create `./docs/antipattern-analysis.md` documenting:

   - Each antipattern found (with file:line references)
   - Severity (low/medium/high)
   - The fix applied
   - Rationale for the change

2. **Then**, modify the source files to implement the fixes:
   - `./src/main.rs`
   - `./src/ai/flee.rs`
   - `./src/ai/mod.rs`
   - `./src/collisions.rs`
   - `./src/level.rs`
   - `./src/utils.rs`

Keep changes focused and minimal. If a file has no antipatterns, leave it unchanged.
</output>

<verification>
Before completing, verify:

1. **Build check**: Run `cargo build` - must compile without errors
2. **Lint check**: Run `cargo clippy --all-targets --all-features -D warnings` - must pass
3. **Format check**: Run `cargo fmt` - code must be formatted
4. **Behavior check**: The game should run identically to before (`cargo run`)

If any verification fails, fix the issues before completing.
</verification>

<success_criteria>

- All identified antipatterns are documented with rationale
- Fixes are implemented without breaking existing functionality
- Code passes clippy with no warnings
- Changes follow the "minimal viable improvement" principle
- No over-engineering or unnecessary abstractions added
- The codebase is cleaner and more maintainable than before
  </success_criteria>
