use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AssemblySocket {
    pub name: String,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AssemblyComponent {
    pub name: String,
    pub component_type: String, // "barrel", "crate", "flora", "mesh"
    pub position: [f32; 3],
    pub scale: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AssemblyGrammar {
    pub root_name: String,
    pub components: Vec<AssemblyComponent>,
    pub sockets: Vec<AssemblySocket>,
    #[serde(default = "default_collider_type")]
    pub collider_type: String,
}

fn default_collider_type() -> String {
    "convex".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledAssembly {
    pub root_name: String,
    pub total_vertices: usize,
    pub total_triangles: usize,
    pub component_count: usize,
    pub sockets: Vec<String>,
    pub collider_node: String,
    pub manifest: blender_mcp_geom::GameAssetManifest,
}

pub fn compile_assembly_grammar(grammar: &AssemblyGrammar) -> CompiledAssembly {
    let mut total_vertices = 0;
    let mut total_triangles = 0;

    for comp in &grammar.components {
        match comp.component_type.as_str() {
            "barrel" => {
                let p = crate::props::synthesize_municipal_prop(&crate::props::PropSynthParams {
                    prop_type: crate::props::MunicipalPropType::WesternBarrel,
                    scale: comp.scale[0],
                    distress: 0.1,
                });
                total_vertices += p.mesh.vertex_count();
                total_triangles += p.mesh.triangle_count();
            }
            "crate" => {
                let p = crate::props::synthesize_municipal_prop(&crate::props::PropSynthParams {
                    prop_type: crate::props::MunicipalPropType::WoodenCrate,
                    scale: comp.scale[0],
                    distress: 0.0,
                });
                total_vertices += p.mesh.vertex_count();
                total_triangles += p.mesh.triangle_count();
            }
            _ => {
                total_vertices += 24;
                total_triangles += 12;
            }
        }
    }

    let socket_names: Vec<String> = grammar.sockets.iter().map(|s| s.name.clone()).collect();
    let prefix = if grammar.collider_type == "box" {
        "UBX_"
    } else if grammar.collider_type == "capsule" {
        "UCP_"
    } else {
        "UCX_"
    };
    let collider_node = format!("{}{}_01", prefix, grammar.root_name);

    let manifest = blender_mcp_geom::GameAssetManifest {
        name: grammar.root_name.clone(),
        version: "0.1.0".to_string(),
        triangle_count: total_triangles,
        vertex_count: total_vertices,
        material_slots: 1,
        dimensions: [1.0, 1.0, 1.0],
        colliders: vec![collider_node.clone()],
        sockets: socket_names.clone(),
        lod_levels: vec!["LOD0".to_string(), "LOD1".to_string()],
    };

    CompiledAssembly {
        root_name: grammar.root_name.clone(),
        total_vertices,
        total_triangles,
        component_count: grammar.components.len(),
        sockets: socket_names,
        collider_node,
        manifest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_assembly_grammar() {
        let grammar = AssemblyGrammar {
            root_name: "Saloon_Entrance_Diorama".to_string(),
            components: vec![
                AssemblyComponent {
                    name: "Stave_Barrel".to_string(),
                    component_type: "barrel".to_string(),
                    position: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0],
                },
                AssemblyComponent {
                    name: "Ammo_Crate".to_string(),
                    component_type: "crate".to_string(),
                    position: [0.5, 0.0, 0.0],
                    scale: [0.5, 0.5, 0.5],
                },
            ],
            sockets: vec![AssemblySocket {
                name: "SOCKET_hitch_ring".to_string(),
                position: [0.0, 1.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
            }],
            collider_type: "convex".to_string(),
        };

        let compiled = compile_assembly_grammar(&grammar);
        assert_eq!(compiled.root_name, "Saloon_Entrance_Diorama");
        assert_eq!(compiled.component_count, 2);
        assert!(compiled.total_triangles > 100);
        assert_eq!(compiled.sockets, vec!["SOCKET_hitch_ring"]);
        assert_eq!(compiled.collider_node, "UCX_Saloon_Entrance_Diorama_01");
        assert_eq!(compiled.manifest.material_slots, 1);
    }
}
