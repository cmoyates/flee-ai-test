use std::collections::HashMap;

use bevy::math::Vec2;

use crate::level::Polygon;

/// Represents an edge segment for spatial partitioning
#[derive(Clone, Copy)]
pub struct Edge {
    pub start: Vec2,
    pub end: Vec2,
}

/// Grid-based spatial hash for efficient raycast queries
///
/// Partitions polygon edges into grid cells to reduce raycast complexity
/// from O(all_edges) to O(nearby_edges).
#[derive(bevy::prelude::Resource)]
pub struct SpatialGrid {
    cell_size: f32,
    grid: HashMap<(i32, i32), Vec<Edge>>,
}

impl SpatialGrid {
    /// Create a new spatial grid from level polygons
    ///
    /// # Arguments
    /// * `polygons` - All polygons in the level
    /// * `cell_size` - Size of each grid cell (should match level grid size for best performance)
    pub fn new(polygons: &[Polygon], cell_size: f32) -> Self {
        let mut grid: HashMap<(i32, i32), Vec<Edge>> = HashMap::new();

        // Insert all edges into grid cells
        for polygon in polygons {
            for i in 1..polygon.points.len() {
                let start = polygon.points[i - 1];
                let end = polygon.points[i];
                let edge = Edge { start, end };

                // Find all grid cells this edge intersects
                let cells = Self::cells_for_edge(start, end, cell_size);
                for cell in cells {
                    grid.entry(cell).or_default().push(edge);
                }
            }
        }

        Self { cell_size, grid }
    }

    /// Get all edges that potentially intersect a ray
    ///
    /// Returns edges in grid cells along the ray path.
    /// This reduces the number of edge tests from O(all_edges) to O(cells_along_ray × edges_per_cell).
    pub fn edges_along_ray(&self, start: Vec2, end: Vec2) -> Vec<Edge> {
        let mut edges = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Walk through grid cells along the ray
        let cells = Self::cells_for_ray(start, end, self.cell_size);
        for cell in cells {
            if let Some(cell_edges) = self.grid.get(&cell) {
                for edge in cell_edges {
                    // Use edge endpoints as key to avoid duplicates
                    let key = (
                        (edge.start.x * 1000.0) as i32,
                        (edge.start.y * 1000.0) as i32,
                        (edge.end.x * 1000.0) as i32,
                        (edge.end.y * 1000.0) as i32,
                    );
                    if seen.insert(key) {
                        edges.push(*edge);
                    }
                }
            }
        }

        edges
    }

    /// Find all grid cells that an edge intersects
    fn cells_for_edge(start: Vec2, end: Vec2, cell_size: f32) -> Vec<(i32, i32)> {
        let mut cells = std::collections::HashSet::new();

        // Get cell coordinates for endpoints
        let start_cell = (
            (start.x / cell_size).floor() as i32,
            (start.y / cell_size).floor() as i32,
        );
        let end_cell = (
            (end.x / cell_size).floor() as i32,
            (end.y / cell_size).floor() as i32,
        );

        cells.insert(start_cell);
        cells.insert(end_cell);

        // Walk along the edge using Bresenham-like algorithm
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let steps = (dx.abs().max(dy.abs()) / cell_size).ceil() as i32 + 1;

        for i in 0..=steps {
            let t = if steps > 0 {
                i as f32 / steps as f32
            } else {
                0.0
            };
            let x = start.x + dx * t;
            let y = start.y + dy * t;
            let cell = (
                (x / cell_size).floor() as i32,
                (y / cell_size).floor() as i32,
            );
            cells.insert(cell);
        }

        cells.into_iter().collect()
    }

    /// Find all grid cells that a ray passes through
    fn cells_for_ray(start: Vec2, end: Vec2, cell_size: f32) -> Vec<(i32, i32)> {
        let mut cells = Vec::new();
        let mut seen = std::collections::HashSet::new();

        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let length = (dx * dx + dy * dy).sqrt();
        let steps = (length / cell_size).ceil() as i32 + 1;

        for i in 0..=steps {
            let t = if steps > 0 {
                i as f32 / steps as f32
            } else {
                0.0
            };
            let x = start.x + dx * t;
            let y = start.y + dy * t;
            let cell = (
                (x / cell_size).floor() as i32,
                (y / cell_size).floor() as i32,
            );
            if seen.insert(cell) {
                cells.push(cell);
            }
        }

        cells
    }
}
