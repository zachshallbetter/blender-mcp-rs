use crate::mesh::ProceduralMesh;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DesertFloraType {
    Saguaro,
    PricklyPear,
    Yucca,
    DesertRock,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FloraSynthParams {
    pub flora_type: DesertFloraType,
    #[serde(default = "default_height")]
    pub height: f32,
    #[serde(default = "default_arm_count")]
    pub arm_count: u32,
    #[serde(default = "default_seed")]
    pub seed: u64,
}

fn default_height() -> f32 {
    3.5
}
fn default_arm_count() -> u32 {
    2
}
fn default_seed() -> u64 {
    42
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizedFlora {
    pub flora_type: DesertFloraType,
    pub mesh: ProceduralMesh,
    pub sockets: Vec<String>,
    pub collider_type: blender_mcp_geom::ColliderType,
}

pub fn synthesize_desert_flora(params: &FloraSynthParams) -> SynthesizedFlora {
    match params.flora_type {
        DesertFloraType::Saguaro => synthesize_saguaro(params),
        DesertFloraType::PricklyPear => synthesize_prickly_pear(params),
        DesertFloraType::Yucca => synthesize_yucca(params),
        DesertFloraType::DesertRock => synthesize_desert_rock(params),
    }
}

fn synthesize_saguaro(params: &FloraSynthParams) -> SynthesizedFlora {
    let mut mesh = ProceduralMesh::new("Desert_Flora_Saguaro", "M_Cactus_Ribbed");
    let ribs = 16;
    let segments = 24;
    let radius = 0.35;
    let height = params.height;

    // Generate fluted cylindrical trunk with vertical ribs
    for s in 0..=segments {
        let v = s as f32 / segments as f32;
        let y = v * height;
        // Natural fluting and taper towards top
        let taper = if v > 0.85 {
            1.0 - (v - 0.85) / 0.15 * 0.4
        } else {
            1.0
        };

        for r in 0..ribs {
            let u = r as f32 / ribs as f32;
            let angle = u * 2.0 * PI;
            // High-frequency fluting along ribs
            let rib_indent = 1.0 + 0.12 * (angle * (ribs as f32 * 0.5)).cos();
            let current_radius = radius * taper * rib_indent;

            let x = angle.cos() * current_radius;
            let z = angle.sin() * current_radius;

            mesh.vertices.push([x, y, z]);
            mesh.normals.push([angle.cos(), 0.0, angle.sin()]);
            mesh.uvs.push([u, v]);
        }
    }

    // Connect quad grid
    for s in 0..segments {
        for r in 0..ribs {
            let next_r = (r + 1) % ribs;
            let idx0 = s * ribs + r;
            let idx1 = s * ribs + next_r;
            let idx2 = (s + 1) * ribs + next_r;
            let idx3 = (s + 1) * ribs + r;

            mesh.indices.push([idx0 as u32, idx1 as u32, idx2 as u32]);
            mesh.indices.push([idx0 as u32, idx2 as u32, idx3 as u32]);
        }
    }

    let mut sockets = vec!["SOCKET_root".to_string(), "SOCKET_crown".to_string()];
    for a in 0..params.arm_count {
        sockets.push(format!("SOCKET_arm_{:02}", a + 1));
    }

    SynthesizedFlora {
        flora_type: DesertFloraType::Saguaro,
        mesh,
        sockets,
        collider_type: blender_mcp_geom::ColliderType::Capsule,
    }
}

fn synthesize_prickly_pear(_params: &FloraSynthParams) -> SynthesizedFlora {
    let mut mesh = ProceduralMesh::new("Desert_Flora_PricklyPear", "M_Cactus_Pad");
    // Generate flattened oval pads
    let segments = 16;
    for i in 0..segments {
        let u = i as f32 / segments as f32;
        let angle = u * 2.0 * PI;
        let x = angle.cos() * 0.4;
        let y = (angle.sin() + 1.0) * 0.5;
        let z = 0.08 * (u * PI).sin();
        mesh.vertices.push([x, y, z]);
        mesh.normals.push([0.0, 0.0, 1.0]);
        mesh.uvs.push([u, 0.5]);
    }

    SynthesizedFlora {
        flora_type: DesertFloraType::PricklyPear,
        mesh,
        sockets: vec!["SOCKET_root".to_string(), "SOCKET_flower".to_string()],
        collider_type: blender_mcp_geom::ColliderType::Convex,
    }
}

fn synthesize_yucca(_params: &FloraSynthParams) -> SynthesizedFlora {
    let mut mesh = ProceduralMesh::new("Desert_Flora_Yucca", "M_Yucca_Blade");
    // Radial rosette of sharp dagger-like leaves
    let blades = 24;
    for b in 0..blades {
        let angle = (b as f32 / blades as f32) * 2.0 * PI;
        let tip_x = angle.cos() * 0.8;
        let tip_y = 0.6;
        let tip_z = angle.sin() * 0.8;

        let base_idx = mesh.vertices.len() as u32;
        mesh.vertices.push([0.0, 0.1, 0.0]);
        mesh.vertices.push([tip_x, tip_y, tip_z]);
        mesh.vertices
            .push([angle.cos() * 0.05, 0.0, angle.sin() * 0.05]);

        mesh.normals.push([0.0, 1.0, 0.0]);
        mesh.normals.push([angle.cos(), 0.5, angle.sin()]);
        mesh.normals.push([0.0, 1.0, 0.0]);

        mesh.uvs.push([0.0, 0.0]);
        mesh.uvs.push([0.5, 1.0]);
        mesh.uvs.push([1.0, 0.0]);

        mesh.indices.push([base_idx, base_idx + 1, base_idx + 2]);
    }

    SynthesizedFlora {
        flora_type: DesertFloraType::Yucca,
        mesh,
        sockets: vec!["SOCKET_root".to_string(), "SOCKET_flower_stalk".to_string()],
        collider_type: blender_mcp_geom::ColliderType::Convex,
    }
}

fn synthesize_desert_rock(_params: &FloraSynthParams) -> SynthesizedFlora {
    let mut mesh = ProceduralMesh::new("Desert_Rock_Boulder", "M_Desert_Stone");
    // Faceted low-poly weathered rock
    let vertices = vec![
        [-0.6, 0.0, -0.5],
        [0.7, 0.0, -0.4],
        [0.5, 0.0, 0.6],
        [-0.5, 0.0, 0.5],
        [-0.4, 0.8, -0.3],
        [0.4, 0.7, -0.2],
        [0.3, 0.9, 0.4],
        [-0.3, 0.7, 0.3],
    ];

    for v in &vertices {
        mesh.vertices.push(*v);
        mesh.normals.push([v[0], 0.5, v[2]]);
        mesh.uvs.push([v[0] + 0.5, v[2] + 0.5]);
    }

    let indices = vec![
        [0, 1, 5],
        [0, 5, 4],
        [1, 2, 6],
        [1, 6, 5],
        [2, 3, 7],
        [2, 7, 6],
        [3, 0, 4],
        [3, 4, 7],
        [4, 5, 6],
        [4, 6, 7],
    ];
    mesh.indices = indices;

    SynthesizedFlora {
        flora_type: DesertFloraType::DesertRock,
        mesh,
        sockets: vec!["SOCKET_ground".to_string()],
        collider_type: blender_mcp_geom::ColliderType::Convex,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saguaro_synthesis() {
        let params = FloraSynthParams {
            flora_type: DesertFloraType::Saguaro,
            height: 4.0,
            arm_count: 3,
            seed: 1234,
        };
        let flora = synthesize_desert_flora(&params);
        assert_eq!(flora.flora_type, DesertFloraType::Saguaro);
        assert!(flora.mesh.vertex_count() > 100);
        assert!(flora.mesh.triangle_count() > 100);
        assert_eq!(flora.sockets.len(), 5); // root + crown + 3 arms
        assert_eq!(flora.collider_type, blender_mcp_geom::ColliderType::Capsule);
    }
}
