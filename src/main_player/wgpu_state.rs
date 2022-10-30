use once_cell::sync::OnceCell;
use std::{
    borrow::Borrow,
    cell::{Cell, RefCell},
};
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt;

use super::{
    error::{MainPlayerError, PlayerErrorResult},
    resources::{camera, texture},
};

static mut STATE: OnceCell<State> = OnceCell::new();

#[derive(Debug)]
pub(super) struct State {
    surface: wgpu::Surface,
    config: RefCell<wgpu::SurfaceConfiguration>,

    device: wgpu::Device,
    queue: wgpu::Queue,

    camera: Cell<camera::Camera>,
    camera_uniform: Cell<camera::CameraUniform>,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    depth_texture: texture::Texture,

    height: Cell<u32>,
    width: Cell<u32>,
}

impl State {
    pub fn get<'a>() -> PlayerErrorResult<&'a Self> {
        if let Some(state) = unsafe { STATE.get() } {
            Ok(state)
        } else {
            Err(MainPlayerError::StateNotInitError)
        }
    }

    pub async fn get_or_init<'a>(canvas: &HtmlCanvasElement) -> PlayerErrorResult<&'a State> {
        let (width, height) = (canvas.width(), canvas.height());

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = instance.create_surface_from_canvas(&canvas);
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        //==Camera==
        let camera = camera::Camera {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 0.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        //==DeepBuffer==
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let config = RefCell::new(config);
        let (camera, camera_uniform) = (Cell::new(camera), Cell::new(camera_uniform));
        let (width, height) = (Cell::new(width), Cell::new(height));

        Ok(unsafe {
            STATE.get_or_init(|| Self {
                surface,
                config,
                device,
                queue,

                camera,
                camera_uniform,
                camera_buffer,
                camera_bind_group,

                depth_texture,

                height,
                width,
            })
        })
    }

    pub fn resize(&self, width: u32, height: u32) {
        self.width.set(width);
        self.height.set(height);

        {
            let mut config = self.config.borrow_mut();
            config.width = width;
            config.height = height;
        }

        self.camera.set(camera::Camera {
            aspect: width as f32 / height as f32,
            ..self.camera.get()
        });

        {
            let mut camera_uniform = self.camera_uniform.get();
            camera_uniform.update_view_proj(&self.camera.get());
            self.camera_uniform.set(camera_uniform);
        }
    }

    pub fn render(&self) -> PlayerErrorResult<()> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
        }

        Ok(())
    }
}
