use image::{GrayImage, ImageBuffer, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PbrError {
    #[error("Image IO or format error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Dimension mismatch between channels: {0}")]
    DimensionMismatch(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrmPackResult {
    pub width: u32,
    pub height: u32,
    pub output_path: String,
}

/// Packs separate Ambient Occlusion (R), Roughness (G), and Metallic (B) maps into a single ORM texture.
/// Missing channels are filled with game-standard PBR defaults:
/// - AO: 255 (1.0, fully unoccluded)
/// - Roughness: 128 (0.5, balanced roughness)
/// - Metallic: 0 (0.0, non-metallic/dielectric)
pub fn pack_orm(
    ao_path: Option<impl AsRef<Path>>,
    roughness_path: Option<impl AsRef<Path>>,
    metallic_path: Option<impl AsRef<Path>>,
    output_path: impl AsRef<Path>,
) -> Result<OrmPackResult, PbrError> {
    let load_gray = |opt: Option<&Path>| -> Result<Option<GrayImage>, PbrError> {
        match opt {
            Some(p) if p.exists() => {
                let img = image::open(p)?;
                Ok(Some(img.to_luma8()))
            }
            _ => Ok(None),
        }
    };

    let ao_p = ao_path.as_ref().map(|p| p.as_ref());
    let rough_p = roughness_path.as_ref().map(|p| p.as_ref());
    let metal_p = metallic_path.as_ref().map(|p| p.as_ref());

    let ao_img = load_gray(ao_p)?;
    let rough_img = load_gray(rough_p)?;
    let metal_img = load_gray(metal_p)?;

    let (width, height) = match (&ao_img, &rough_img, &metal_img) {
        (Some(img), _, _) => img.dimensions(),
        (_, Some(img), _) => img.dimensions(),
        (_, _, Some(img)) => img.dimensions(),
        _ => (1024, 1024),
    };

    let mut orm_buffer: RgbaImage = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..x_limit(width) {
            let r = ao_img.as_ref().map_or(255, |img| img.get_pixel(x, y)[0]);
            let g = rough_img.as_ref().map_or(128, |img| img.get_pixel(x, y)[0]);
            let b = metal_img.as_ref().map_or(0, |img| img.get_pixel(x, y)[0]);
            orm_buffer.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    let out_ref = output_path.as_ref();
    if let Some(parent) = out_ref.parent() {
        std::fs::create_dir_all(parent)?;
    }

    orm_buffer.save(out_ref)?;

    Ok(OrmPackResult {
        width,
        height,
        output_path: out_ref.display().to_string(),
    })
}

#[inline(always)]
fn x_limit(w: u32) -> u32 {
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_orm_defaults() {
        let temp_dir = std::env::temp_dir();
        let out_file = temp_dir.join("test_orm_output.png");

        let res = pack_orm(None::<&Path>, None::<&Path>, None::<&Path>, &out_file)
            .expect("Packing failed");

        assert_eq!(res.width, 1024);
        assert_eq!(res.height, 1024);

        let img = image::open(&out_file)
            .expect("Failed to load output image")
            .to_rgba8();
        let px = img.get_pixel(0, 0);
        assert_eq!(px[0], 255); // Default AO
        assert_eq!(px[1], 128); // Default Roughness
        assert_eq!(px[2], 0); // Default Metallic
        assert_eq!(px[3], 255); // Alpha

        let _ = std::fs::remove_file(out_file);
    }
}
