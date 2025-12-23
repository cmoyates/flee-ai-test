<objective>
Fix all `cargo clippy` linting errors and warnings in the codebase.

Clippy is Rust's official linter that catches common mistakes and suggests idiomatic improvements. A clean clippy run ensures code quality and adherence to Rust best practices.
</objective>

<context>
This is a Rust project using Bevy 0.12.1 game engine. The source code is in `./src/`.

Read CLAUDE.md for project conventions before making changes.
</context>

<requirements>
1. Run `cargo clippy` to identify all linting issues
2. Fix each warning/error following clippy's suggestions
3. Preserve the existing functionality - do not change behavior, only improve code quality
4. Follow Rust idioms and best practices as suggested by clippy
</requirements>

<implementation>
Common clippy fixes include:
- Removing unnecessary clones or references
- Using more idiomatic patterns (e.g., `if let` instead of `match` with one arm)
- Simplifying expressions
- Fixing unused variables or imports
- Using proper method chains

Do NOT:

- Change any game logic or behavior
- Add new features or functionality
- Refactor beyond what clippy suggests
  </implementation>

<verification>
After making all fixes, run `cargo clippy` again to confirm:
- Zero warnings
- Zero errors
- All code compiles successfully with `cargo build`
</verification>

<success_criteria>

- `cargo clippy` produces no warnings or errors
- `cargo build` succeeds
- All existing functionality remains unchanged
  </success_criteria>
