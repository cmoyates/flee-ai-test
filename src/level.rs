use bevy::{color::Color, math::Vec2};
use rand::Rng;

use crate::utils::line_intersect;

pub struct Polygon {
    pub points: Vec<Vec2>,
    pub collision_side: f32,
    pub color: Color,
}

const LEVEL_DATA: &[u8] = include_bytes!("../assets/level.json");

pub fn generate_level_polygons(grid_size: f32) -> (Vec<Polygon>, Vec2, Vec2) {
    let mut rng = rand::rng();

    let json_str =
        std::str::from_utf8(LEVEL_DATA).expect("Failed to convert level data to UTF-8 string");
    let json_data: Vec<Vec<u32>> =
        serde_json::from_str(json_str).expect("Failed to parse level JSON data");

    let size = Vec2::new(json_data[0].len() as f32, json_data.len() as f32);

    let offset = Vec2::new(size.x * -grid_size / 2.0, size.y * grid_size / 2.0);

    let mut line_points: Vec<Vec2> = Vec::new();

    for y in 0..size.y as usize {
        for x in 0..size.x as usize {
            let tile = json_data[y][x];

            match tile {
                1 => {
                    // Squares

                    // Left edge
                    if x == 0 || json_data[y][x - 1] == 0 {
                        line_points.push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));
                        line_points
                            .push(Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size));
                    }
                    // Right edge
                    if x == json_data[y].len() - 1 || json_data[y][x + 1] == 0 {
                        line_points
                            .push(Vec2::new((x + 1) as f32 * grid_size, y as f32 * grid_size));
                        line_points.push(Vec2::new(
                            (x + 1) as f32 * grid_size,
                            (y + 1) as f32 * grid_size,
                        ));
                    }
                    // Top edge
                    if y == 0 || json_data[y - 1][x] == 0 {
                        line_points.push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));
                        line_points
                            .push(Vec2::new((x + 1) as f32 * grid_size, y as f32 * grid_size));
                    }
                    // Bottom edge
                    if y == size.y as usize - 1 || json_data[y + 1][x] == 0 {
                        line_points
                            .push(Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size));
                        line_points.push(Vec2::new(
                            (x + 1) as f32 * grid_size,
                            (y + 1) as f32 * grid_size,
                        ));
                    }
                }
                2..=5 => {
                    // Right triangles

                    let triangle_type = tile - 2;

                    match triangle_type {
                        0 => {
                            // Bottom left

                            // Hypotenuse
                            line_points.push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));
                            line_points.push(Vec2::new(
                                (x + 1) as f32 * grid_size,
                                (y + 1) as f32 * grid_size,
                            ));

                            // Bottom edge
                            if y == size.y as usize - 1 || json_data[y + 1][x] == 0 {
                                line_points.push(Vec2::new(
                                    x as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                            }

                            // Left edge
                            if x == 0 || json_data[y][x - 1] == 0 {
                                line_points
                                    .push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));
                                line_points.push(Vec2::new(
                                    x as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                            }
                        }
                        1 => {
                            // Bottom right

                            // Hypotenuse
                            line_points
                                .push(Vec2::new((x + 1) as f32 * grid_size, y as f32 * grid_size));
                            line_points
                                .push(Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size));

                            // Bottom edge
                            if y == size.y as usize - 1 || json_data[y + 1][x] == 0 {
                                line_points.push(Vec2::new(
                                    x as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                            }

                            // Right edge
                            if x == json_data[y].len() - 1 || json_data[y][x + 1] == 0 {
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    y as f32 * grid_size,
                                ));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                            }
                        }
                        2 => {
                            // Top left

                            // Hypotenuse
                            line_points
                                .push(Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size));
                            line_points
                                .push(Vec2::new((x + 1) as f32 * grid_size, y as f32 * grid_size));

                            // Top edge
                            if y == 0 || json_data[y - 1][x] == 0 {
                                line_points
                                    .push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    y as f32 * grid_size,
                                ));
                            }

                            // Left edge
                            if x == 0 || json_data[y][x - 1] == 0 {
                                line_points
                                    .push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));
                                line_points.push(Vec2::new(
                                    x as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                            }
                        }
                        3 => {
                            // Top right

                            // Hypotenuse
                            line_points.push(Vec2::new(
                                (x + 1) as f32 * grid_size,
                                (y + 1) as f32 * grid_size,
                            ));
                            line_points.push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));

                            // Top edge
                            if y == 0 || json_data[y - 1][x] == 0 {
                                line_points
                                    .push(Vec2::new(x as f32 * grid_size, y as f32 * grid_size));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    y as f32 * grid_size,
                                ));
                            }

                            // Right edge
                            if x == json_data[y].len() - 1 || json_data[y][x + 1] == 0 {
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    y as f32 * grid_size,
                                ));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_size,
                                    (y + 1) as f32 * grid_size,
                                ));
                            }
                        }
                        _ => {}
                    }
                }
                6..=9 => {
                    // // Isosceles triangles

                    // let triangle_type = tile - 6;

                    // match triangle_type {
                    //     0 => {
                    //         // Bottom

                    //         // Top left
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Top right
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 1.0) as f32 * grid_size,
                    //                 (y + 1) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Bottom edge
                    //         if y == json_data.len() - 1 || json_data[y + 1][x] == 0 {
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size),
                    //                 Color::RED,
                    //             ));
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new(
                    //                     (x + 1) as f32 * grid_size,
                    //                     (y + 1) as f32 * grid_size,
                    //                 ),
                    //                 Color::RED,
                    //             ));
                    //         }
                    //     }
                    //     1 => {
                    //         // Top

                    //         // Bottom left
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(x as f32 * grid_size, y as f32 * grid_size),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Bottom right
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 1.0) as f32 * grid_size,
                    //                 y as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Top edge
                    //         if y == 0 || json_data[y - 1][x] == 0 {
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new(x as f32 * grid_size, y as f32 * grid_size),
                    //                 Color::RED,
                    //             ));
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new((x + 1) as f32 * grid_size, y as f32 * grid_size),
                    //                 Color::RED,
                    //             ));
                    //         }
                    //     }
                    //     2 => {
                    //         // Left

                    //         // Top right
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(x as f32 * grid_size, y as f32 * grid_size),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Bottom right
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Left edge
                    //         if x == 0 || json_data[y][x - 1] == 0 {
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new(x as f32 * grid_size, y as f32 * grid_size),
                    //                 Color::RED,
                    //             ));
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size),
                    //                 Color::RED,
                    //             ));
                    //         }
                    //     }
                    //     3 => {
                    //         // Right

                    //         // Top left
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new((x + 1) as f32 * grid_size, y as f32 * grid_size),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Bottom left
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new((x + 1) as f32 * grid_size, (y + 1) as f32 * grid_size),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Right edge
                    //         if x == json_data[y].len() - 1 || json_data[y][x + 1] == 0 {
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new((x + 1) as f32 * grid_size, y as f32 * grid_size),
                    //                 Color::RED,
                    //             ));
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new(
                    //                     (x + 1) as f32 * grid_size,
                    //                     (y + 1) as f32 * grid_size,
                    //                 ),
                    //                 Color::RED,
                    //             ));
                    //         }
                    //     }
                    //     _ => {}
                    // }
                }
                _ => {}
            }
        }
    }

    let mut line_count = line_points.len() / 2;

    // Remove superfluous points

    let mut point_removal_data = Some(((0, 0), (0, 0)));

    // While there are points to remove
    while point_removal_data.is_some() {
        point_removal_data = None;

        'outer: for i in 0..line_count {
            for j in 0..line_count {
                // If the lines are the same, skip
                if i == j {
                    continue;
                }

                // Check if either of the points are shared

                let line_1_start = line_points[i * 2];
                let line_1_end = line_points[i * 2 + 1];

                let line_2_start = line_points[j * 2];
                let line_2_end = line_points[j * 2 + 1];

                let mut shared_point: Option<(usize, usize)> = None;
                let mut unique_points: Option<(usize, usize)> = None;

                if line_1_start == line_2_start {
                    shared_point = Some((i * 2, j * 2));
                    unique_points = Some((i * 2 + 1, j * 2 + 1));
                } else if line_1_start == line_2_end {
                    shared_point = Some((i * 2, j * 2 + 1));
                    unique_points = Some((i * 2 + 1, j * 2));
                } else if line_1_end == line_2_start {
                    shared_point = Some((i * 2 + 1, j * 2));
                    unique_points = Some((i * 2, j * 2 + 1));
                } else if line_1_end == line_2_end {
                    shared_point = Some((i * 2 + 1, j * 2 + 1));
                    unique_points = Some((i * 2, j * 2));
                }

                // If there is no shared point, skip
                if shared_point.is_none() {
                    continue;
                }

                // Check if the lines are parallel

                let dot = (line_1_start - line_1_end)
                    .normalize()
                    .dot((line_2_start - line_2_end).normalize());
                if dot.abs() == 1.0 {
                    // if so flag the point for removal and break out of the outer for loop
                    if let (Some(shared), Some(unique)) = (shared_point, unique_points) {
                        point_removal_data = Some((shared, unique));
                        break 'outer;
                    }
                }
            }
        }

        // If there is a point to remove
        if let Some(point_removal_data) = point_removal_data {
            // Store the unique vertices
            let unique_vert_1 = line_points[point_removal_data.1 .0];
            let unique_vert_2 = line_points[point_removal_data.1 .1];

            // Remove the shared and unique vertices
            let mut removal_indices = vec![
                point_removal_data.0 .0,
                point_removal_data.0 .1,
                point_removal_data.1 .0,
                point_removal_data.1 .1,
            ];
            removal_indices.sort();
            removal_indices.reverse();
            for i in removal_indices {
                line_points.remove(i);
            }

            // Add the unique vertices back
            line_points.push(unique_vert_1);
            line_points.push(unique_vert_2);

            // Update the line count
            line_count -= 1;
        }
    }

    for point in &mut line_points {
        point.x += offset.x;
        point.y *= -1.0;
        point.y += offset.y;
    }

    // Separate the lines into polygons
    let mut polygons: Vec<Polygon> = Vec::new();

    let container_color = Color::srgb(
        rng.random_range(0.0..=1.0),
        rng.random_range(0.0..=1.0),
        rng.random_range(0.0..=1.0),
    );

    // While there are lines left
    while line_count > 0 {
        // Create a new polygon
        let mut polygon_lines: Vec<Vec2> = Vec::new();

        // Add the first line to the polygon
        polygon_lines.push(line_points[0]);
        polygon_lines.push(line_points[1]);

        // Remove the first line from the list of lines
        line_points.remove(0);
        line_points.remove(0);

        // Decrement the line count
        line_count -= 1;

        let start_vert = polygon_lines[0];
        let mut current_vert = polygon_lines[polygon_lines.len() - 1];

        // While the polygon is not closed
        while start_vert != current_vert {
            let mut found = false;
            let mut i = 0;
            while i < line_count {
                let line_start = line_points[i * 2];
                let line_end = line_points[i * 2 + 1];

                // If the line starts at the current vertex
                if line_start == current_vert {
                    // Add the line to the polygon
                    polygon_lines.push(line_end);

                    // Remove the line from the list of lines
                    line_points.remove(i * 2);
                    line_points.remove(i * 2);

                    // Decrement the line count
                    line_count -= 1;

                    // Set the current vertex to the end of the line
                    current_vert = line_end;

                    // Break out of the while loop
                    found = true;
                    break;
                }
                // If the line ends at the current vertex
                else if line_end == current_vert {
                    // Add the line to the polygon
                    polygon_lines.push(line_start);

                    // Remove the line from the list of lines
                    line_points.remove(i * 2);
                    line_points.remove(i * 2);

                    // Decrement the line count
                    line_count -= 1;

                    // Set the current vertex to the start of the line
                    current_vert = line_start;

                    // Break out of the while loop
                    found = true;
                    break;
                } else {
                    i += 1;
                }
            }
            if !found {
                break;
            }
        }

        let is_container = point_in_polygon(&polygon_lines, Vec2::new(0.0, 0.0));
        let mut collision_side = calculate_winding_order(&polygon_lines).signum();
        if is_container {
            collision_side *= -1.0;
        }

        let color = if is_container {
            container_color
        } else {
            Color::srgb(
                rng.random_range(0.0..=1.0),
                rng.random_range(0.0..=1.0),
                rng.random_range(0.0..=1.0),
            )
        };

        // Add the polygon to the list of polygons
        polygons.push(Polygon {
            points: polygon_lines,
            collision_side,
            color,
        });
    }

    (polygons, size, size / 2.0)
}

fn calculate_winding_order(vertices: &[Vec2]) -> f32 {
    let mut sum = 0.0;

    for i in 0..vertices.len() {
        let p1 = vertices[i];
        let p2 = vertices[(i + 1) % vertices.len()];
        sum += (p2.x - p1.x) * (p2.y + p1.y);
    }

    sum
}

fn point_in_polygon(polygon_lines: &[Vec2], point: Vec2) -> bool {
    let test_line_start = point;
    let test_line_end = point + Vec2::new(2.0, 1.0) * 1000.0;

    let mut intersect_counter = 0;

    for i in 1..polygon_lines.len() {
        let start = polygon_lines[i - 1];
        let end = polygon_lines[i];

        let intersection = line_intersect(start, end, test_line_start, test_line_end);

        if intersection.is_some() {
            intersect_counter += 1;
        }
    }

    intersect_counter % 2 == 1
}
