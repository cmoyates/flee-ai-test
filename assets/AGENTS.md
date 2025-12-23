# assets/AGENTS.md - Asset Management

## Directory Overview

This directory contains game assets loaded at runtime. Currently, all assets are **embedded at compile time** via `include_bytes!()` for zero-runtime overhead.

**Key File**: `level.json` - Level geometry data

---

## Asset Loading Pattern

### Compile-Time Embedding

Assets are embedded directly into the binary:

```rust
// From src/level.rs
const LEVEL_DATA: &[u8] = include_bytes!("../assets/level.json");

// Usage:
let json_str = std::str::from_utf8(LEVEL_DATA)?;
let data: Vec<Vec<u32>> = serde_json::from_str(json_str)?;
```

**Benefits**:
- Zero runtime I/O
- No asset path resolution needed
- Single binary deployment
- Fast startup (no loading delays)

**Trade-offs**:
- Binary size increases
- No hot-reloading (requires rebuild)
- No runtime asset swapping

---

## level.json Format

### Structure

A 2D array of tile IDs:

```json
[
  [1, 1, 1, 1, ...],
  [1, 0, 0, 0, ...],
  [1, 0, 0, 0, ...],
  ...
]
```

**Dimensions**: 19 columns × 21 rows (as of current file)

**Coordinate System**:
- `[y][x]` indexing (row, column)
- `(0, 0)` = top-left corner
- Y increases downward (converted to world space in `level.rs`)

### Tile Types

| ID | Type | Description |
|---|---|---|
| `0` | Empty | Walkable space (no collision) |
| `1` | Square | Full 1×1 tile (4 edges) |
| `2` | Triangle BL | Right triangle, bottom-left (hypotenuse top-right) |
| `3` | Triangle BR | Right triangle, bottom-right (hypotenuse top-left) |
| `4` | Triangle TL | Right triangle, top-left (hypotenuse bottom-right) |
| `5` | Triangle TR | Right triangle, top-right (hypotenuse bottom-left) |
| `6-9` | Isosceles | Isosceles triangles (currently **unused/commented**) |

### Tile Visual Reference

```
Square (1):      Triangle BL (2):  Triangle BR (3):
┌─────┐          ┌─────┐          ┌─────┐
│     │          │    ╱│          │╲    │
│     │          │  ╱  │          │  ╲  │
└─────┘          │╱    │          │    ╲│
                └─────┘          └─────┘

Triangle TL (4): Triangle TR (5):
┌─────┐          ┌─────┐
│    ╲│          │╱    │
│  ╲  │          │  ╲  │
│╲    │          │    ╲│
└─────┘          └─────┘
```

---

## Level Generation Process

**Function**: `generate_level_polygons(grid_size: f32)` in `src/level.rs`

### Steps

1. **Load JSON**: Parse `level.json` into `Vec<Vec<u32>>`
2. **Extract Edges**: For each tile, generate edge lines (only exposed edges)
3. **Optimize Lines**: Remove collinear adjacent segments
4. **Assemble Polygons**: Connect lines into closed polygons
5. **Determine Collision Side**: Calculate winding order, invert for container polygon
6. **Assign Colors**: Random colors for visualization

### Edge Extraction Rules

**Square (1)**:
- Left edge: If `x == 0` OR `tile[x-1][y] == 0`
- Right edge: If `x == width-1` OR `tile[x+1][y] == 0`
- Top edge: If `y == 0` OR `tile[x][y-1] == 0`
- Bottom edge: If `y == height-1` OR `tile[x][y+1] == 0`

**Triangles (2-5)**:
- Hypotenuse: Always generated
- Legs: Only if adjacent tile is empty (same rules as square edges)

### Polygon Optimization

**Collinear Removal**:
- If two adjacent lines share a point and are parallel, merge them
- Reduces vertex count for performance

**Polygon Assembly**:
- Start with first line
- Find connected lines (share start/end point)
- Continue until polygon closes (start == end)

### Collision Side Calculation

**Winding Order**:
- Positive winding = counter-clockwise (collision on inside)
- Negative winding = clockwise (collision on outside)

**Container Detection**:
- Test if origin `(0, 0)` is inside polygon (ray-casting method)
- If container: invert collision side (walls are outside)

---

## World Space Conversion

**Grid Size**: `32.0` pixels per tile (configurable)

**Coordinate Transform**:
```rust
let offset = Vec2::new(size.x * -grid_size / 2.0, size.y * grid_size / 2.0);

// For each point:
point.x += offset.x;
point.y *= -1.0;  // Flip Y (JSON Y-down → world Y-up)
point.y += offset.y;
```

**Result**: Level centered at world origin `(0, 0)`

---

## Adding New Assets

### Pattern for New Asset Type

1. **Create asset file** in `assets/` directory
2. **Embed in source**:
   ```rust
   const ASSET_DATA: &[u8] = include_bytes!("../assets/my_asset.json");
   ```
3. **Parse with serde**:
   ```rust
   let data: MyType = serde_json::from_slice(ASSET_DATA)?;
   ```
4. **Load in startup system** or resource initialization

### Example: Adding Sprite Data

```rust
// assets/sprites.json
{
  "player": { "path": "sprites/player.png", "size": [32, 32] },
  "enemy": { "path": "sprites/enemy.png", "size": [24, 24] }
}

// src/assets.rs
const SPRITE_DATA: &[u8] = include_bytes!("../assets/sprites.json");

#[derive(Deserialize)]
struct SpriteData {
    path: String,
    size: [u32; 2],
}

pub fn load_sprites() -> HashMap<String, SpriteData> {
    let json_str = std::str::from_utf8(SPRITE_DATA).unwrap();
    serde_json::from_str(json_str).unwrap()
}
```

---

## Runtime Asset Loading (Future)

If you need runtime loading (e.g., for modding or DLC):

1. **Use Bevy AssetServer**:
   ```rust
   let handle: Handle<LevelAsset> = asset_server.load("levels/level1.json");
   ```

2. **Define Asset Type**:
   ```rust
   #[derive(Asset, TypePath, Deserialize)]
   struct LevelAsset {
       tiles: Vec<Vec<u32>>,
   }
   ```

3. **Register Asset Type**:
   ```rust
   app.init_asset::<LevelAsset>();
   ```

**Note**: Current implementation uses compile-time embedding for simplicity and performance.

---

## Asset Validation

### Level.json Validation Rules

- Must be valid JSON array of arrays
- All rows must have same length (rectangular grid)
- Tile IDs must be 0-9 (or extend enum if adding new types)
- At least one container polygon (outer boundary)

### Validation Function (Suggested)

```rust
fn validate_level(data: &Vec<Vec<u32>>) -> Result<(), String> {
    if data.is_empty() {
        return Err("Level is empty".to_string());
    }
    
    let width = data[0].len();
    for (y, row) in data.iter().enumerate() {
        if row.len() != width {
            return Err(format!("Row {} has inconsistent width", y));
        }
        for tile in row {
            if *tile > 9 {
                return Err(format!("Invalid tile ID: {}", tile));
            }
        }
    }
    Ok(())
}
```

---

## Performance Notes

### Current Approach

- **Load Time**: O(1) - embedded at compile time
- **Parse Time**: O(n) where n = tile count
- **Polygon Generation**: O(n²) worst case (line optimization)

### Optimization Opportunities

- **Spatial Indexing**: For large levels, use grid/quadtree for collision queries
- **Precomputed Polygons**: Generate polygons at build time, embed as JSON
- **LOD System**: Multiple polygon detail levels for large levels

---

## Quick Reference

### Find Asset Loading
```bash
rg -n "include_bytes!" src/
```

### Find Asset Parsing
```bash
rg -n "serde_json::from" src/
```

### Find Tile Type Usage
```bash
rg -n "match tile|tile_type" src/level.rs
```

---

## Next Steps

- **Level generation code**: See [../src/level.rs](../src/level.rs)
- **Core systems**: See [../src/AGENTS.md](../src/AGENTS.md)
- **Root guide**: See [../AGENTS.md](../AGENTS.md)

