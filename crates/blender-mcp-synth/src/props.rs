use crate::mesh::ProceduralMesh;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MunicipalPropType {
    WesternBarrel,
    WoodenCrate,
    HitchingPost,
    Lantern,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PropSynthParams {
    pub prop_type: MunicipalPropType,
    #[serde(default = "default_scale")]
    pub scale: f32,
    #[serde(default = "default_distress")]
    pub distress: f32,
}

fn default_scale() -> f32 {
    1.0
}
fn default_distress() -> f32 {
    0.1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizedProp {
    pub prop_type: MunicipalPropType,
    pub mesh: ProceduralMesh,
    pub sockets: Vec<String>,
    pub collider_type: blender_mcp_geom::ColliderType,
}

pub fn synthesize_municipal_prop(params: &PropSynthParams) -> SynthesizedProp {
    match params.prop_type {
        MunicipalPropType::WesternBarrel => synthesize_barrel(params),
        MunicipalPropType::WoodenCrate => synthesize_crate(params),
        MunicipalPropType::HitchingPost => synthesize_hitching_post(params),
        MunicipalPropType::Lantern => synthesize_lantern(params),
    }
}

fn synthesize_barrel(params: &PropSynthParams) -> SynthesizedProp {
    let mut mesh = ProceduralMesh::new("Western_Barrel_Master", "M_Western_Wood");
    let staves = 16;
    let rings = 8;
    let height = 1.0 * params.scale;
    let base_radius = 0.38 * params.scale;
    let belly_radius = 0.46 * params.scale;

    for r in 0..=rings {
        let v = r as f32 / rings as f32;
        let y = (v - 0.5) * height;
        // Parabolic stave bulge (barrel curvature)
        let bulge = 1.0 - 4.0 * (v - 0.5) * (v - 0.5);
        let current_radius = base_radius + (belly_radius - base_radius) * bulge;

        for s in 0..staves {
            let u = s as f32 / staves as f32;
            let angle = u * 2.0 * PI;
            let x = angle.cos() * current_radius;
            let z = angle.sin() * current_radius;

            mesh.vertices.push([x, y, z]);
            mesh.normals
                .push([angle.cos(), 0.1 * (v - 0.5), angle.sin()]);
            mesh.uvs.push([u, v]);
        }
    }

    for r in 0..rings {
        for s in 0..staves {
            let next_s = (s + 1) % staves;
            let idx0 = r * staves + s;
            let idx1 = r * staves + next_s;
            let idx2 = (r + 1) * staves + next_s;
            let idx3 = (r + 1) * staves + s;

            mesh.indices.push([idx0 as u32, idx1 as u32, idx2 as u32]);
            mesh.indices.push([idx0 as u32, idx2 as u32, idx3 as u32]);
        }
    }

    SynthesizedProp {
        prop_type: MunicipalPropType::WesternBarrel,
        mesh,
        sockets: vec![
            "SOCKET_grip_primary".to_string(),
            "SOCKET_grip_secondary".to_string(),
            "SOCKET_bung_hole".to_string(),
        ],
        collider_type: blender_mcp_geom::ColliderType::Capsule,
    }
}

fn synthesize_crate(params: &PropSynthParams) -> SynthesizedProp {
    let mut mesh = ProceduralMesh::new("Frontier_Crate_Master", "M_Crate_Wood");
    let size = 0.8 * params.scale;
    let h = size * 0.5;

    let corners = [
        [-h, -h, -h],
        [h, -h, -h],
        [h, h, -h],
        [-h, h, -h],
        [-h, -h, h],
        [h, -h, h],
        [h, h, h],
        [-h, h, h],
    ];

    for c in &corners {
        mesh.vertices.push(*c);
        mesh.normals.push([c[0], c[1], c[2]]);
        mesh.uvs.push([c[0] + 0.5, c[1] + 0.5]);
    }

    let indices = [
        // Front
        [0, 1, 2],
        [0, 2, 3],
        // Back
        [5, 4, 7],
        [5, 7, 6],
        // Top
        [3, 2, 6],
        [3, 6, 7],
        // Bottom
        [4, 5, 1],
        [4, 1, 0],
        // Right
        [1, 5, 6],
        [1, 6, 2],
        // Left
        [4, 0, 3],
        [4, 3, 7],
    ];
    mesh.indices.extend_from_slice(&indices);

    SynthesizedProp {
        prop_type: MunicipalPropType::WoodenCrate,
        mesh,
        sockets: vec!["SOCKET_mount_top".to_string(), "SOCKET_ground".to_string()],
        collider_type: blender_mcp_geom::ColliderType::Box,
    }
}

fn synthesize_hitching_post(params: &PropSynthParams) -> SynthesizedProp {
    let mut mesh = ProceduralMesh::new("Saloon_Hitching_Post", "M_Weathered_Wood");
    let h = 1.2 * params.scale;
    let r = 0.12 * params.scale;
    let segments = 8;

    for y_step in [0.0, h] {
        for s in 0..segments {
            let u = s as f32 / segments as f32;
            let angle = u * 2.0 * PI;
            let x = angle.cos() * r;
            let z = angle.sin() * r;
            mesh.vertices.push([x, y_step, z]);
            mesh.normals.push([angle.cos(), 0.0, angle.sin()]);
            mesh.uvs.push([u, y_step / h]);
        }
    }

    for s in 0..segments {
        let next_s = (s + 1) % segments;
        let idx0 = s;
        let idx1 = next_s;
        let idx2 = segments + next_s;
        let idx3 = segments + s;
        mesh.indices.push([idx0 as u32, idx1 as u32, idx2 as u32]);
        mesh.indices.push([idx0 as u32, idx2 as u32, idx3 as u32]);
    }

    SynthesizedProp {
        prop_type: MunicipalPropType::HitchingPost,
        mesh,
        sockets: vec!["SOCKET_hitch_ring".to_string(), "SOCKET_ground".to_string()],
        collider_type: blender_mcp_geom::ColliderType::Capsule,
    }
}

fn synthesize_lantern(params: &PropSynthParams) -> SynthesizedProp {
    let mut mesh = ProceduralMesh::new("Kerosene_Lantern", "M_Lantern_Metal");
    let h = 0.45 * params.scale;
    let r = 0.14 * params.scale;
    let segments = 12;

    for y_step in [0.0, h * 0.3, h * 0.7, h] {
        for s in 0..segments {
            let u = s as f32 / segments as f32;
            let angle = u * 2.0 * PI;
            let current_r = if y_step > h * 0.3 && y_step < h * 0.7 {
                r * 1.2
            } else {
                r
            };
            mesh.vertices
                .push([angle.cos() * current_r, y_step, angle.sin() * current_r]);
            mesh.normals.push([angle.cos(), 0.0, angle.sin()]);
            mesh.uvs.push([u, y_step / h]);
        }
    }

    SynthesizedProp {
        prop_type: MunicipalPropType::Lantern,
        mesh,
        sockets: vec![
            "SOCKET_grip_handle".to_string(),
            "SOCKET_fx_light".to_string(),
            "SOCKET_fx_smoke".to_string(),
        ],
        collider_type: blender_mcp_geom::ColliderType::Convex,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_barrel_synthesis() {
        let params = PropSynthParams {
            prop_type: MunicipalPropType::WesternBarrel,
            scale: 1.0,
            distress: 0.1,
        };
        let prop = synthesize_municipal_prop(&params);
        assert_eq!(prop.prop_type, MunicipalPropType::WesternBarrel);
        assert!(prop.mesh.triangle_count() > 100);
        assert!(prop.sockets.contains(&"SOCKET_grip_primary".to_string()));
        assert_eq!(prop.collider_type, blender_mcp_geom::ColliderType::Capsule);
    }

    #[test]
    fn test_crate_synthesis() {
        let params = PropSynthParams {
            prop_type: MunicipalPropType::WoodenCrate,
            scale: 1.0,
            distress: 0.0,
        };
        let prop = synthesize_municipal_prop(&params);
        assert_eq!(prop.collider_type, blender_mcp_geom::ColliderType::Box);
        assert_eq!(prop.mesh.triangle_count(), 12);
    }
}
