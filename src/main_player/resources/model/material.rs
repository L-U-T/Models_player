use crate::{main_player::{error::PlayerErrorResult, resources::texture}, requests};

#[derive(Debug)]
pub(crate) struct Material {
    pub(crate) name: String,
    pub(crate) diffuse_texture: texture::Texture,
    pub normal_texture: texture::Texture,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl Material {
    pub(super) fn from_tobj_materials(
        name: &str,
        texture_img: requests::Image,
        normal_img: requests::Image,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) -> PlayerErrorResult<Self> {
        let diffuse_texture =
            texture::Texture::from_image(device, queue, texture_img, None, false)?;
        let normal_texture = texture::Texture::from_image(device, queue, normal_img, None, true)?;

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
            label: Some(name),
        });

        Ok(Self {
            name: name.to_string(),
            diffuse_texture,
            normal_texture,
            bind_group,
        })
    }
}
