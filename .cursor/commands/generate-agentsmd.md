# Task: Analyze this Bevy (Rust) game codebase and generate a hierarchical AGENTS.md structure

## Context & Principles

You are going to help me create a **hierarchical AGENTS.md system** for this codebase. This is critical for AI coding agents to work efficiently with minimal token usage.

### Core Principles:

1. **Root AGENTS.md is LIGHTWEIGHT** - Only universal guidance + links to sub-files
2. **Nearest-wins hierarchy** - Agents read the closest AGENTS.md to the file being edited
3. **JIT (Just-In-Time) indexing** - Provide paths/globs/commands, NOT full content
4. **Token efficiency** - Small, actionable guidance over encyclopedic documentation
5. **Sub-folder AGENTS.md files have MORE detail** - Specific patterns, examples, commands

## Your Process

### Phase 1: Repository Analysis

First, analyze the codebase structure and provide me with:

1. **Repository type**: Workspace (Cargo workspace), multi-crate, or single crate?
2. **Primary technology stack**:
   - Rust edition + toolchain (stable/nightly)
   - Bevy version + major plugins (bevy_egui, rapier, bevy_kira_audio, etc.)
   - Asset pipeline (aseprite, LDtk, Tiled, glTF, custom formats)
3. **Major directories/crates** that should have their own AGENTS.md:
   - Crates (e.g., `crates/game`, `crates/engine`, `crates/ui`, `crates/netcode`)
   - Game modules (e.g., `src/systems`, `src/plugins`, `src/states`, `src/scenes`)
   - Tools (e.g., `tools/asset-pipeline`, `tools/level-compiler`)
   - Assets (e.g., `assets/`, `assets/levels`, `assets/shaders`)
4. **Build system**:
   - Cargo workspace? custom features? build scripts? CI?
   - Profiles (dev/release), target platforms (desktop/web/mobile)
5. **Testing setup**:
   - Unit tests (`#[test]`), integration tests (`tests/`), snapshot tests
   - Any golden/asset validation tests
6. **Key patterns to document**:
   - ECS architecture conventions (plugins, systems, resources, events)
   - State management (AppState/States), schedules (Update/FixedUpdate)
   - Asset loading conventions (AssetServer handles, preloading, hot-reload expectations)
   - Naming/structure conventions for components/resources/events
   - Critical example files + anti-patterns (e.g., monolithic `systems.rs`, excessive `ResMut`, large queries in tight loops)

Present this as a **structured map** before generating any AGENTS.md files.

---

### Phase 2: Generate Root AGENTS.md

Create a **lightweight root AGENTS.md** (~100-200 lines max) that includes:

#### Required Sections:

**1. Project Snapshot** (3-5 lines)

- Repo type (Cargo workspace / single crate)
- Primary tech stack (Rust + Bevy + key plugins)
- Note that sub-crates / folders have their own AGENTS.md files

**2. Root Setup Commands** (copy-paste ready)

- Build all: `cargo build`
- Run game: `cargo run`
- Fast dev run (if features exist): `cargo run --features dev`
- Format: `cargo fmt`
- Lint: `cargo clippy --all-targets --all-features -D warnings`
- Test all: `cargo test --all`
- Optional checks if used: `cargo deny check`, `cargo audit`, `cargo machete`

**3. Universal Conventions** (5-10 lines)

- Rust style: `rustfmt` required; clippy warnings treated as errors
- Prefer Bevy “plugin-first” architecture (small plugins over one giant App setup)
- Avoid tight-loop allocations; profile before optimizing
- Feature flags: document how `dev`, `debug_ui`, `tracing` features are used (if present)

**4. Security & Secrets** (3-5 lines)

- Never commit API keys/tokens
- Config/secrets via `.env` or platform-specific config (if applicable)
- Never log PII; be careful with telemetry if present

**5. JIT Index - Directory Map** (10-20 lines)
Structure like:

```md
## JIT Index (what to open, not what to paste)

### Crate / Folder Map

- Core game crate: `src/` → [see src/AGENTS.md](src/AGENTS.md)
- Shared crates: `crates/**/` → [see crates/<name>/AGENTS.md](crates/<name>/AGENTS.md)
- Asset pipeline: `tools/**/` → [see tools/<name>/AGENTS.md](tools/<name>/AGENTS.md)
- Assets: `assets/` → [see assets/AGENTS.md](assets/AGENTS.md)

### Quick Find Commands

- Find a component: `rg -n "derive\\(Component\\)" -S .`
- Find systems: `rg -n "fn .*\\(.*(Query|Res|Commands|EventReader|EventWriter)" -S src crates`
- Find plugins: `rg -n "impl Plugin for" -S src crates`
- Find States: `rg -n "derive\\(States\\)|NextState<|in_state\\(" -S src crates`
- Find asset loads: `rg -n "asset_server\\.load|AssetServer|Handle<" -S src crates`
```
