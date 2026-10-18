use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PolyhavenAsset {
    pub id: String,
    pub name: String,
    pub asset_type: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawPolyhavenEntry {
    name: String,
    #[serde(rename = "type")]
    raw_type: Option<u8>,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}

pub async fn search_polyhaven(
    query: &str,
    category: Option<&str>,
    asset_type: Option<&str>,
    limit: usize,
) -> Result<Vec<PolyhavenAsset>, anyhow::Error> {
    let type_param = match asset_type {
        Some("textures") => "?t=textures",
        Some("models") => "?t=models",
        Some("hdris") => "?t=hdris",
        _ => "",
    };

    let url = format!("https://api.polyhaven.com/assets{}", type_param);
    let client = reqwest::Client::builder()
        .user_agent("blender-mcp-rs/0.4.0 (github.com/zachshallbetter/blender-mcp-rs)")
        .build()?;

    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "Polyhaven API error: HTTP {}",
            resp.status()
        ));
    }

    let raw_map: HashMap<String, RawPolyhavenEntry> = resp.json().await?;
    let q_lower = query.to_lowercase();
    let cat_lower = category.map(|c| c.to_lowercase());

    let mut results = Vec::new();
    for (id, entry) in raw_map {
        let type_str = match entry.raw_type {
            Some(0) => "hdris",
            Some(1) => "textures",
            Some(2) => "models",
            _ => "unknown",
        };

        // Filter by category if requested
        if let Some(ref cat) = cat_lower {
            if !entry
                .categories
                .iter()
                .any(|c| c.to_lowercase().contains(cat))
            {
                continue;
            }
        }

        // Filter by query if non-empty
        if !q_lower.is_empty() {
            let matches_name = entry.name.to_lowercase().contains(&q_lower);
            let matches_id = id.to_lowercase().contains(&q_lower);
            let matches_tags = entry
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(&q_lower));
            let matches_cats = entry
                .categories
                .iter()
                .any(|c| c.to_lowercase().contains(&q_lower));

            if !matches_name && !matches_id && !matches_tags && !matches_cats {
                continue;
            }
        }

        results.push(PolyhavenAsset {
            id,
            name: entry.name,
            asset_type: type_str.to_string(),
            categories: entry.categories,
            tags: entry.tags,
        });

        if results.len() >= limit {
            break;
        }
    }

    Ok(results)
}

pub async fn get_polyhaven_download_url(
    id: &str,
    resolution: &str,
) -> Result<String, anyhow::Error> {
    let url = format!("https://api.polyhaven.com/files/{}", id);
    let client = reqwest::Client::builder()
        .user_agent("blender-mcp-rs/0.4.0 (github.com/zachshallbetter/blender-mcp-rs)")
        .build()?;

    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "Polyhaven files lookup failed for {}: HTTP {}",
            id,
            resp.status()
        ));
    }

    let files_val: serde_json::Value = resp.json().await?;

    // Check if it's a model (.gltf or .blend)
    if let Some(gltf_url) = files_val
        .get("gltf")
        .and_then(|g| g.get(resolution).or_else(|| g.get("1k")))
        .and_then(|res| res.get("gltf").or_else(|| res.get("glb")))
        .and_then(|f| f.get("url"))
        .and_then(|u| u.as_str())
    {
        return Ok(gltf_url.to_string());
    }

    // Check if it's a texture (Diffuse map)
    if let Some(diff_url) = files_val
        .get("Diffuse")
        .and_then(|d| d.get(resolution).or_else(|| d.get("1k")))
        .and_then(|res| res.get("jpg").or_else(|| res.get("png")))
        .and_then(|f| f.get("url"))
        .and_then(|u| u.as_str())
    {
        return Ok(diff_url.to_string());
    }

    // Check if it's an HDRI
    if let Some(hdri_url) = files_val
        .get("hdri")
        .and_then(|h| h.get(resolution).or_else(|| h.get("1k")))
        .and_then(|res| res.get("hdr").or_else(|| res.get("exr")))
        .and_then(|f| f.get("url"))
        .and_then(|u| u.as_str())
    {
        return Ok(hdri_url.to_string());
    }

    Err(anyhow::anyhow!(
        "No download URL found for asset '{}' at resolution '{}'",
        id,
        resolution
    ))
}

pub async fn download_asset_file(url: &str, dest_path: &Path) -> Result<PathBuf, anyhow::Error> {
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "Download failed from {}: HTTP {}",
            url,
            resp.status()
        ));
    }

    let bytes = resp.bytes().await?;
    if let Some(parent) = dest_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(dest_path, &bytes).await?;
    Ok(dest_path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_polyhaven_parse() {
        let json_data = r#"{
            "rock_moss": {
                "name": "Rock Moss",
                "type": 1,
                "categories": ["nature", "stone"],
                "tags": ["rock", "moss", "green"]
            }
        }"#;

        let parsed: HashMap<String, RawPolyhavenEntry> = serde_json::from_str(json_data).unwrap();
        assert_eq!(parsed.len(), 1);
        let entry = parsed.get("rock_moss").unwrap();
        assert_eq!(entry.name, "Rock Moss");
        assert_eq!(entry.raw_type, Some(1));
    }
}
