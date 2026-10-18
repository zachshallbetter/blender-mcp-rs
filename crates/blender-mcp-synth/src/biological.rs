use crate::mesh::ProceduralMesh;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BiologicalGrowthParams {
    /// Growth archetype: "rhizome-foliage-mat", "phragmites-colony", "fungal-network"
    #[serde(default = "default_bio_archetype")]
    pub archetype: String,
    /// Radial extent of patch in meters (default 0.9m)
    #[serde(default = "default_patch_radius")]
    pub patch_radius: f32,
    /// Number of vegetative shoots / fruiting bodies
    #[serde(default = "default_node_count")]
    pub node_count: usize,
    /// Average shoot height in meters (default 1.2m)
    #[serde(default = "default_shoot_height")]
    pub shoot_height: f32,
    /// Base stalk / stipe radius in meters (default 0.015m)
    #[serde(default = "default_stalk_radius")]
    pub stalk_radius: f32,
    /// Deterministic PRNG seed
    #[serde(default = "default_seed")]
    pub seed: u64,
    /// Destination GLB or asset path
    #[serde(default = "default_bio_output")]
    pub output_path: String,
}

fn default_bio_archetype() -> String {
    "rhizome-foliage-mat".to_string()
}
fn default_patch_radius() -> f32 {
    0.9
}
fn default_node_count() -> usize {
    24
}
fn default_shoot_height() -> f32 {
    1.2
}
fn default_stalk_radius() -> f32 {
    0.015
}
fn default_seed() -> u64 {
    42
}
fn default_bio_output() -> String {
    "./assets/biological_mat.glb".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiologicalSynthResult {
    pub status: String,
    pub archetype: String,
    pub root_node: String,
    pub total_nodes: usize,
    pub total_triangles: usize,
    pub sockets: Vec<String>,
    pub colliders: Vec<String>,
    pub output_path: String,
    pub manifest: serde_json::Value,
}

/// Simple deterministic linear congruential PRNG
struct LcgRng {
    state: u64,
}

impl LcgRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x5DEECE66D,
        }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(25214903917).wrapping_add(11);
        let val = (self.state >> 16) & 0xFFFFFF;
        val as f32 / 16777216.0
    }
}

pub fn synthesize_biological_mat(params: &BiologicalGrowthParams) -> BiologicalSynthResult {
    let mut rng = LcgRng::new(params.seed);
    let root_node = format!("BioMat_{}", params.archetype.replace('-', "_"));
    let mut mesh = ProceduralMesh::new(&root_node, "M_Biological_Foliage");
    let mut sockets = vec!["SOCKET_patch_center".to_string()];

    for i in 0..params.node_count {
        // Golden ratio spiral distribution inside radius
        let r = params.patch_radius * (rng.next_f32()).sqrt();
        let theta = 2.0 * PI * rng.next_f32();
        let cx = r * theta.cos();
        let cy = r * theta.sin();

        let h_variation = 0.8 + 0.4 * rng.next_f32();
        let shoot_h = params.shoot_height * h_variation;
        let stalk_r = params.stalk_radius * (0.8 + 0.3 * rng.next_f32());

        // Generate tapered stalk with 5 segments along height
        let segments = 5;
        let ring_verts = 6;
        let start_vert_idx = mesh.vertices.len() as u32;

        for s in 0..=segments {
            let t = s as f32 / segments as f32;
            let z = t * shoot_h;
            // slight organic bend
            let bend_x = cx + (t * t * 0.1 * (rng.next_f32() - 0.5));
            let bend_y = cy + (t * t * 0.1 * (rng.next_f32() - 0.5));
            let cur_r = stalk_r * (1.0 - 0.6 * t);

            for v in 0..ring_verts {
                let phi = 2.0 * PI * (v as f32) / (ring_verts as f32);
                let vx = bend_x + cur_r * phi.cos();
                let vy = bend_y + cur_r * phi.sin();
                mesh.vertices.push([vx, vy, z]);
                mesh.normals.push([phi.cos(), phi.sin(), 0.1]);
                mesh.uvs.push([phi / (2.0 * PI), t]);
            }
        }

        // Connect rings with quad pairs (triangles)
        for s in 0..segments {
            for v in 0..ring_verts {
                let next_v = (v + 1) % ring_verts;
                let i0 = start_vert_idx + (s * ring_verts + v) as u32;
                let i1 = start_vert_idx + (s * ring_verts + next_v) as u32;
                let i2 = start_vert_idx + ((s + 1) * ring_verts + v) as u32;
                let i3 = start_vert_idx + ((s + 1) * ring_verts + next_v) as u32;

                mesh.indices.push([i0, i1, i2]);
                mesh.indices.push([i1, i3, i2]);
            }
        }

        if i % 4 == 0 {
            sockets.push(format!("SOCKET_shoot_{:02}", i));
        }
    }

    let colliders = vec![format!("UCX_{}_01", root_node)];
    let total_triangles = mesh.triangle_count();

    let manifest = serde_json::json!({
        "schema_version": "1.0.0",
        "asset_id": format!("bio.{}", params.archetype),
        "name": root_node,
        "author": "Zachary Shallbetter",
        "license": "Proprietary",
        "triangle_budget": {
            "max": 5000,
            "actual": total_triangles
        },
        "material_slots": [
            {
                "slot_name": "mat_biological_foliage",
                "shader": "PBR_MetallicRoughness"
            }
        ],
        "sockets": sockets,
        "colliders": colliders,
        "parameters": {
            "patch_radius": params.patch_radius,
            "node_count": params.node_count,
            "shoot_height": params.shoot_height,
            "seed": params.seed
        }
    });

    BiologicalSynthResult {
        status: "success".to_string(),
        archetype: params.archetype.clone(),
        root_node,
        total_nodes: params.node_count,
        total_triangles,
        sockets,
        colliders,
        output_path: params.output_path.clone(),
        manifest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biological_mat_synthesis() {
        let params = BiologicalGrowthParams {
            archetype: "phragmites-colony".to_string(),
            patch_radius: 1.0,
            node_count: 12,
            shoot_height: 1.5,
            stalk_radius: 0.02,
            seed: 12345,
            output_path: "/tmp/phragmites.glb".to_string(),
        };

        let res = synthesize_biological_mat(&params);
        assert_eq!(res.status, "success");
        assert_eq!(res.total_nodes, 12);
        assert!(res.total_triangles > 0);
        assert!(res.sockets.contains(&"SOCKET_patch_center".to_string()));
        assert!(res.colliders[0].starts_with("UCX_"));
    }
}
