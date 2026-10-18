use std::path::Path;
use parry3d::math::Point;
use parry3d::transformation::convex_hull;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeomError {
    #[error("glTF parsing error: {0}")]
    Gltf(#[from] gltf::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Geometry computation error: {0}")]
    Computation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAuditReport {
    pub file_path: String,
    pub total_triangles: usize,
    pub total_vertices: usize,
    pub mesh_count: usize,
    pub material_count: usize,
    pub has_uv_coordinates: bool,
    pub bounding_box_min: [f32; 3],
    pub bounding_box_max: [f32; 3],
    pub dimensions: [f32; 3],
    pub passes_game_ready_check: bool,
    pub warnings: Vec<String>,
}

pub struct ModelAuditor;

impl ModelAuditor {
    pub fn audit_glb(path: impl AsRef<Path>) -> Result<ModelAuditReport, GeomError> {
        let path_ref = path.as_ref();
        let (document, buffers, _images) = gltf::import(path_ref)?;

        let mut total_triangles = 0;
        let mut total_vertices = 0;
        let mut mesh_count = 0;
        let mut has_uv = false;

        let mut min_bound = [f32::INFINITY; 3];
        let mut max_bound = [f32::NEG_INFINITY; 3];

        for mesh in document.meshes() {
            mesh_count += 1;
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|b| Some(&buffers[b.index()]));

                if let Some(iter) = reader.read_positions() {
                    for pos in iter {
                        total_vertices += 1;
                        for i in 0..3 {
                            if pos[i] < min_bound[i] {
                                min_bound[i] = pos[i];
                            }
                            if pos[i] > max_bound[i] {
                                max_bound[i] = pos[i];
                            }
                        }
                    }
                }

                if let Some(indices) = reader.read_indices() {
                    let count = indices.into_u32().count();
                    total_triangles += count / 3;
                }

                if reader.read_tex_coords(0).is_some() {
                    has_uv = true;
                }
            }
        }

        let material_count = document.materials().count();
        let mut warnings = Vec::new();

        if material_count > 2 {
            warnings.push(format!(
                "Material slots count ({}) exceeds game performance budget (<= 2).",
                material_count
            ));
        }

        if !has_uv {
            warnings.push("Asset lacks UV coordinates in channel 0.".to_string());
        }

        let dimensions = [
            (max_bound[0] - min_bound[0]).abs(),
            (max_bound[1] - min_bound[1]).abs(),
            (max_bound[2] - min_bound[2]).abs(),
        ];

        let passes_game_ready_check = warnings.is_empty() && has_uv;

        Ok(ModelAuditReport {
            file_path: path_ref.display().to_string(),
            total_triangles,
            total_vertices,
            mesh_count,
            material_count,
            has_uv_coordinates: has_uv,
            bounding_box_min: min_bound,
            bounding_box_max: max_bound,
            dimensions,
            passes_game_ready_check,
            warnings,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvexHullMesh {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
}

/// Computes a minimal convex hull for physics simulation using Parry3D.
pub fn compute_convex_hull(points: &[[f32; 3]]) -> Result<ConvexHullMesh, GeomError> {
    if points.len() < 4 {
        return Err(GeomError::Computation(
            "Convex hull requires at least 4 non-coplanar points".to_string(),
        ));
    }

    let parry_points: Vec<Point<f32>> = points
        .iter()
        .map(|p| Point::new(p[0], p[1], p[2]))
        .collect();

    let (hull_points, hull_indices) = convex_hull(&parry_points);

    let vertices = hull_points.into_iter().map(|p| [p.x, p.y, p.z]).collect();
    let indices = hull_indices;

    Ok(ConvexHullMesh { vertices, indices })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_convex_hull_cube() {
        let cube_points = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];

        let hull = compute_convex_hull(&cube_points).expect("Hull computation failed");
        assert_eq!(hull.vertices.len(), 8);
        assert_eq!(hull.indices.len(), 12); // 12 triangles for 6 cube faces
    }

    #[test]
    fn test_compute_convex_hull_insufficient_points() {
        let few_points = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let err = compute_convex_hull(&few_points);
        assert!(err.is_err());
    }
}
