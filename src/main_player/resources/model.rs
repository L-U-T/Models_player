use crate::{main_player::error::PlayerErrorResult, requests};

use self::{material::Material, mesh::Mesh};

mod draw_trait;
mod material;
mod mesh;
mod vertex;

#[derive(Debug)]
pub(crate) struct Model {
    pub(crate) meshes: Vec<Mesh>,
    pub(crate) materials: Vec<Material>,
}

use std::{
    io::{BufReader, Cursor},
    ops::Range,
};

use super::texture;
impl Model {
    pub(crate) async fn from_file_name(
        name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) -> PlayerErrorResult<Self> {
        let obj_text = requests::request_string(&format!("/static/obj/{}", name))
            .await
            .unwrap();
        let obj_cursor = Cursor::new(obj_text);
        let mut obj_reader = BufReader::new(obj_cursor);

        let (models, obj_materials) = tobj::load_obj_buf_async(
            &mut obj_reader,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            |p| async move {
                let mat_text = requests::request_string(&format!("/static/mtl/{}", p))
                    .await
                    .unwrap();
                tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
            },
        )
        .await?;

        let mut materials = Vec::new();
        for m in obj_materials? {
            materials.push(Material::from_tobj_materials(
                &m.name,
                requests::Image::from_name(&m.diffuse_texture).await?,
                requests::Image::from_name(&m.normal_texture).await?,
                device,
                queue,
                layout,
            )?)
        }

        let meshes = models
            .into_iter()
            .map(|m| Mesh::from_tobj_model(&m.name, &m, device))
            .collect::<Vec<_>>();

        Ok(Self { meshes, materials })
    }
}
