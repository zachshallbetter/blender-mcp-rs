use serde::{Deserialize, Serialize};

/// Compiler Semantic Graph exchange schema adapted for 3D spatial scenes (`rr-csg-0.1+3d`).
/// Based on reflective-rust's CSG protocol with 3D spatial entity extensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialCsgGraph {
    pub schema_version: String,
    pub provenance: CsgProvenance,
    pub nodes: Vec<CsgNode>,
    pub edges: Vec<CsgEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsgProvenance {
    pub kind: String,      // "extractor"
    pub extractor: String, // "blender-mcp-rs"
    pub domain: String,    // "spatial-3d"
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CsgNodeKind {
    Scene,
    Object,
    Mesh,
    Empty,
    Camera,
    Light,
    Socket,
    Collider,
    Material,
    Texture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsgSpan {
    pub file: String,
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsgNode {
    pub id: String,
    pub name: String,
    pub kind: CsgNodeKind,
    pub minimum_access: String, // "public", "member", "owner", "administrator"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<CsgTransform>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triangle_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsgTransform {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CsgRelationship {
    Contains,
    Parents,
    Anchors,
    CollidesWith,
    AppliesMaterial,
    SamplesTexture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsgEdge {
    pub from: String,
    pub to: String,
    pub relationship: CsgRelationship,
    pub minimum_access: String,
}

impl SpatialCsgGraph {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            schema_version: "rr-csg-0.1+3d".to_string(),
            provenance: CsgProvenance {
                kind: "extractor".to_string(),
                extractor: "blender-mcp-rs".to_string(),
                domain: "spatial-3d".to_string(),
                source: source.into(),
            },
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: CsgNode) {
        self.nodes.push(node);
    }

    pub fn add_edge(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        rel: CsgRelationship,
    ) {
        self.edges.push(CsgEdge {
            from: from.into(),
            to: to.into(),
            relationship: rel,
            minimum_access: "public".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_csg_construction() {
        let mut graph = SpatialCsgGraph::new("dustwater_saloon.blend");

        let root_node = CsgNode {
            id: "obj:Hero_Barrel".to_string(),
            name: "Hero_Barrel".to_string(),
            kind: CsgNodeKind::Object,
            minimum_access: "public".to_string(),
            transform: Some(CsgTransform {
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            }),
            triangle_count: None,
            metadata: None,
        };

        let mesh_node = CsgNode {
            id: "mesh:Hero_Barrel_Mesh".to_string(),
            name: "Hero_Barrel_Mesh".to_string(),
            kind: CsgNodeKind::Mesh,
            minimum_access: "public".to_string(),
            transform: None,
            triangle_count: Some(1840),
            metadata: None,
        };

        let socket_node = CsgNode {
            id: "socket:SOCKET_grip_primary".to_string(),
            name: "SOCKET_grip_primary".to_string(),
            kind: CsgNodeKind::Socket,
            minimum_access: "public".to_string(),
            transform: Some(CsgTransform {
                position: [0.0, 0.4, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            }),
            triangle_count: None,
            metadata: None,
        };

        let collider_node = CsgNode {
            id: "collider:UCX_Hero_Barrel_01".to_string(),
            name: "UCX_Hero_Barrel_01".to_string(),
            kind: CsgNodeKind::Collider,
            minimum_access: "public".to_string(),
            transform: None,
            triangle_count: Some(64),
            metadata: Some(serde_json::json!({ "type": "convex_hull" })),
        };

        graph.add_node(root_node);
        graph.add_node(mesh_node);
        graph.add_node(socket_node);
        graph.add_node(collider_node);

        graph.add_edge(
            "obj:Hero_Barrel",
            "mesh:Hero_Barrel_Mesh",
            CsgRelationship::Contains,
        );
        graph.add_edge(
            "obj:Hero_Barrel",
            "socket:SOCKET_grip_primary",
            CsgRelationship::Anchors,
        );
        graph.add_edge(
            "obj:Hero_Barrel",
            "collider:UCX_Hero_Barrel_01",
            CsgRelationship::CollidesWith,
        );

        assert_eq!(graph.schema_version, "rr-csg-0.1+3d");
        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.edges.len(), 3);

        let json = serde_json::to_string_pretty(&graph).unwrap();
        assert!(json.contains("rr-csg-0.1+3d"));
        assert!(json.contains("SOCKET_grip_primary"));
        assert!(json.contains("collides_with"));
    }
}
