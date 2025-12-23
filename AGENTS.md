# AGENTS.md - Root Guide

## Project Snapshot

**Repository Type**: Single Rust crate (not a workspace)  
**Primary Tech Stack**: Rust 2021 edition + Bevy 0.16.1 + rand + serde/serde_json  
**Structure**: Single crate with modular source organization (`src/ai/`, `src/collisions.rs`, `src/level.rs`, etc.)  
**Sub-folders**: See [src/AGENTS.md](src/AGENTS.md) for detailed module guidance

---

## Root Setup Commands

### Build & Run

```bash
# Build the project
cargo build

# Run the game (native)
cargo run

# Build for release
cargo build --release

# Build and run WASM version (if configured)
cargo run --target wasm32-unknown-unknown
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint (treat warnings as errors)
cargo clippy --all-targets --all-features -D warnings

# Run all tests
cargo test --all
```

### Optional Checks

```bash
# Check for unused dependencies (requires cargo-udeps)
cargo udeps

# Security audit (requires cargo-audit)
cargo audit

# Find unused code (requires cargo-machete)
cargo machete
```

---

## Universal Conventions

### Rust Style

- **Formatting**: `rustfmt` required (run `cargo fmt` before committing)
- **Linting**: Clippy warnings treated as errors (`-D warnings`)
- **Edition**: Rust 2021 edition

### Bevy Architecture

- **Plugin-first**: Organize systems into plugins (`CollisionPlugin`, `FleeAIPlugin`) rather than monolithic `main.rs` setup
- **System ordering**: Use `.before()` and `.after()` for explicit ordering dependencies
- **ECS patterns**: Prefer `Query` with filters (`With<T>`, `Without<T>`) for efficient entity access
- **Resources**: Use `Res<T>` for read-only, `ResMut<T>` for mutable resources

### Performance

- **Avoid tight-loop allocations**: Profile before optimizing; prefer stack allocations in hot paths
- **System efficiency**: Keep queries focused; avoid iterating over large entity sets unnecessarily
- **Asset loading**: Use `include_bytes!()` for static assets (as done with `level.json`)

### Feature Flags

- No custom feature flags currently defined
- Platform-specific code uses `#[cfg(not(target_arch = "wasm32"))]` for native-only behavior

---

## Security & Secrets

- **No API keys/tokens**: This is a local game project with no external services
- **No PII**: No user data collection or logging
- **Assets**: Level data is embedded at compile time (`include_bytes!()`)
- **Platform differences**: Escape key exit disabled on WASM for web compatibility

---

## JIT Index (what to open, not what to paste)

### Directory Map

- **Core game source**: `src/` → [see src/AGENTS.md](src/AGENTS.md)

  - `src/main.rs`: Application entry point, player controller, input handling, rendering
  - `src/ai/`: AI behavior module → [see src/ai/AGENTS.md](src/ai/AGENTS.md)
  - `src/collisions.rs`: Physics-based collision detection and resolution
  - `src/level.rs`: Level generation from JSON tile data
  - `src/utils.rs`: Math utilities (lerp, line intersection, cross product)

- **Assets**: `assets/` → [see assets/AGENTS.md](assets/AGENTS.md)
  - `assets/level.json`: Embedded level geometry data (2D tile array)

### Quick Find Commands

```bash
# Find all components
rg -n "derive\(Component\)" -S .

# Find all systems (functions that take Query/Res/Commands/etc.)
rg -n "fn .*\(.*(Query|Res|Commands|EventReader|EventWriter)" -S src

# Find all plugins
rg -n "impl Plugin for" -S src

# Find all resources
rg -n "derive\(Resource\)" -S src

# Find system ordering dependencies
rg -n "\.(before|after)\(" -S src

# Find asset loading
rg -n "include_bytes!|asset_server\.load|AssetServer|Handle<" -S src

# Find platform-specific code
rg -n "#\[cfg" -S src
```

### Key Files Reference

- **Entry point**: `src/main.rs` (App setup, systems, resources)
- **AI behavior**: `src/ai/flee.rs` (FleeAI component, steering behaviors, LOS detection)
- **Collision system**: `src/collisions.rs` (CollisionPlugin, line projection, penetration resolution)
- **Level data**: `src/level.rs` (Polygon generation, JSON parsing, optimization)
- **Math utilities**: `src/utils.rs` (Line intersection, lerp, cross product)

---

## Common Patterns

### Plugin Structure

```rust
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, my_system.before(other_system));
    }
}
```

### Component Definition

```rust
#[derive(Component)]
pub struct MyComponent {
    pub field: Type,
}
```

### Resource Definition

```rust
#[derive(Resource)]
pub struct MyResource {
    pub data: Type,
}
```

### System with Query

```rust
pub fn my_system(
    mut query: Query<&mut Transform, With<MyComponent>>,
    resource: Res<MyResource>,
) {
    for mut transform in query.iter_mut() {
        // System logic
    }
}
```

---

## Anti-Patterns to Avoid

- ❌ **Monolithic systems**: Don't put all logic in `main.rs`; use plugins
- ❌ **Excessive `ResMut`**: Prefer `Res` when mutation isn't needed
- ❌ **Large queries in tight loops**: Keep queries focused and use filters
- ❌ **Runtime asset loading for static data**: Use `include_bytes!()` for compile-time assets
- ❌ **Ignoring system ordering**: Use `.before()`/`.after()` for dependencies

---

## Next Steps

For detailed guidance on specific modules:

- **Game systems & architecture**: See [src/AGENTS.md](src/AGENTS.md)
- **AI behavior**: See [src/ai/AGENTS.md](src/ai/AGENTS.md)
- **Asset management**: See [assets/AGENTS.md](assets/AGENTS.md)

---

## btca

Trigger: user says "use btca" (for codebase/docs questions).

Run:

- btca ask -t <tech> -q "<question>"

## btca

Trigger: user says "use btca" (for codebase/docs questions).

Run:

- btca ask -t <tech> -q "<question>"

Available <tech>:

- bevy
- bevy-docs
- bevy-cheatbook
- rust-book
