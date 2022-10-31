use image::RgbaImage;

use super::{binary::request_binary, error::RequestResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    diffuse_rgba: RgbaImage,
    /// (width, height) of image texture
    dimensions: (u32, u32),
}

impl Image {
    pub async fn from_name(name: &str) -> RequestResult<Self> {
        Ok(Self::from_bytes(
            &request_binary(&format!("/static/image/{}", name)).await?,
        ))
    }

    pub fn from_bytes(diffuse_bytes: &[u8]) -> Self {
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();

        Self {
            diffuse_rgba,
            dimensions,
        }
    }

    pub fn into_diffuse_rgba(self) -> RgbaImage {
        self.diffuse_rgba
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }
}
