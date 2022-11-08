use cgmath::Rotation3;
use once_cell::sync::OnceCell;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    f32::consts::PI,
};
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt;

use super::{
    error::{MainPlayerError, PlayerErrorResult},
    resources::{camera, instance, light, model, shader, texture},
};

static mut STATE: OnceCell<State> = OnceCell::new();

pub(super) struct State {
    pub surface: wgpu::Surface,
    pub config: RefCell<wgpu::SurfaceConfiguration>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub obj_models: HashMap<String, model::Model>,

    pub light_uniform: Cell<light::LightUniform>,
    pub light_buffer: wgpu::Buffer,
    pub light_bind_group: wgpu::BindGroup,

    pub camera: Cell<camera::Camera>,
    pub camera_uniform: Cell<camera::CameraUniform>,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,

    pub instances: Vec<instance::Instance>,
    pub instance_buffer: wgpu::Buffer,

    pub light_render_pipeline: wgpu::RenderPipeline,
    pub yueqin_render_pipeline: wgpu::RenderPipeline,

    pub depth_texture: texture::Texture,

    pub height: Cell<u32>,
    pub width: Cell<u32>,

    animation: OnceCell<(
        RefCell<HashMap<String, Box<dyn Fn(&State)>>>,
        gloo::timers::callback::Interval,
    )>,
}

impl State {
    pub fn get<'a>() -> PlayerErrorResult<&'a Self> {
        if let Some(state) = unsafe { STATE.get() } {
            Ok(state)
        } else {
            Err(MainPlayerError::StateNotInitError)
        }
    }

    pub async fn init<'a>(canvas: &HtmlCanvasElement) -> PlayerErrorResult<&'a State> {
        unsafe {
            STATE = OnceCell::new();
        }

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
            .expect("unexpected error cause adapter not available");

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
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        //==Model==
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let mut obj_models = HashMap::new();

        let yueqin_obj_model =
            model::Model::from_file_name("Yueqin.obj", &device, &queue, &texture_bind_group_layout)
                .await
                .unwrap();
        obj_models.insert("yueqin".to_owned(), yueqin_obj_model);

        let cube_obj_model =
            model::Model::from_file_name("cube.obj", &device, &queue, &texture_bind_group_layout)
                .await
                .unwrap();
        obj_models.insert("cube".to_owned(), cube_obj_model);

        let plane_obj_model =
            model::Model::from_file_name("plane.obj", &device, &queue, &texture_bind_group_layout)
                .await
                .unwrap();
        obj_models.insert("plane".to_owned(), plane_obj_model);

        //==Camera==
        let camera = camera::Camera {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (100.0, 0.0, 0.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 20.0,
            znear: 0.1,
            zfar: 3.0e28,
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

        //==Light==
        let light_uniform = light::LightUniform {
            position: [2.0, 5.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
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

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        //==Instances==
        let mut instances = Vec::new();

        instances.push(instance::Instance {
            position: cgmath::Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
            scale: 1.0,
        });

        instances.push(instance::Instance {
            position: cgmath::Vector3 {
                x: 0.0,
                y: -10.0,
                z: 0.0,
            },
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
            scale: 1.0,
        });

        let instance_data = instances
            .iter()
            .map(instance::Instance::to_raw)
            .collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        //==z-Buffer==
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        //==Shader==
        let shader = shader::Shader::from_file_name("Normal Shader", "bp.wgsl");

        let yueqin_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

            shader.await?.create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[
                    model::vertex::ModelVertex::desc(),
                    instance::InstanceRaw::desc(),
                ],
            )
        };

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });

            shader::Shader::from_file_name("Light Shader", "pure.wgsl")
                .await?
                .create_render_pipeline(
                    &device,
                    &layout,
                    config.format,
                    Some(texture::Texture::DEPTH_FORMAT),
                    &[model::vertex::ModelVertex::desc()],
                )
        };

        let config = RefCell::new(config);
        let light_uniform = Cell::new(light_uniform);
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

                obj_models,

                light_uniform,
                light_buffer,
                light_bind_group,

                instances,
                instance_buffer,

                light_render_pipeline,
                yueqin_render_pipeline,

                height,
                width,

                animation: OnceCell::new(),
            })
        })
    }

    pub fn display_change(
        &self,
        width: u32,
        height: u32,
        cursor_to: (f32, f32),
        wheel_to: f32,
    ) -> PlayerErrorResult<()> {
        self.width.set(width);
        self.height.set(height);

        {
            let mut config = self.config.borrow_mut();
            config.width = width;
            config.height = height;
        }

        // camera controller
        let camera_pos = {
            let mut camera_pos = self.camera.get().get_pos();

            let r = wheel_to;

            let r_xy = (cursor_to.1 * PI).sin() * r;
            camera_pos.1 = (cursor_to.1 * PI).cos() * r;

            camera_pos.0 = (cursor_to.0 * PI).cos() * r_xy;
            camera_pos.2 = (cursor_to.0 * PI).sin() * r_xy;

            camera_pos
        };

        let camera = camera::Camera {
            eye: cgmath::Point3 {
                x: camera_pos.0,
                y: camera_pos.1,
                z: camera_pos.2,
            },
            aspect: width as f32 / height as f32,
            ..self.camera.get()
        };

        self.camera.set(camera);

        // effect camera change
        {
            let mut camera_uniform = self.camera_uniform.get();
            camera_uniform.update_view_proj(&camera);
            self.camera_uniform.set(camera_uniform);

            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[self.camera_uniform.get()]),
            );
        }

        self.render()
    }

    /// # TODO
    ///  - add time difference as attribute for f
    ///  - fix preemption problem if possible
    pub fn animation_insert(&self, lable: String, f: Box<dyn for<'r> Fn(&'r State)>) {
        if let Some(state) = unsafe { STATE.get() } {
            if let Some((anima_loop, _)) = state.animation.get() {
                anima_loop.borrow_mut().insert(lable, f);
            } else {
                state.animation.get_or_init(|| {
                    (
                        {
                            let mut hashmap = HashMap::new();
                            hashmap.insert(lable, f);
                            RefCell::new(hashmap)
                        },
                        gloo::timers::callback::Interval::new(17, || {
                            if let Some((animation_loop, _)) = state.animation.get() {
                                animation_loop.borrow().iter().for_each(|(_, f)| f(state));
                            };

                            state.render().unwrap();
                        }),
                    )
                });
            }
        }
    }

    pub fn animation_clear(&self) {
        if let Some(state) = unsafe { STATE.get() } {
            if let Some((animation_loop, _)) = state.animation.get() {
                animation_loop.borrow_mut().clear();
            }
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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            // render()

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            use light::DrawLight;
            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model(
                &self.obj_models.get("cube").unwrap(),
                &self.camera_bind_group,
                &self.light_bind_group,
            );

            render_pass.set_pipeline(&self.yueqin_render_pipeline);
            model::draw_trait::DrawModel::draw_model_instanced(
                &mut render_pass,
                &self.obj_models.get("yueqin").unwrap(),
                0..1,
                &self.camera_bind_group,
                &self.light_bind_group,
            );

            render_pass.set_pipeline(&self.yueqin_render_pipeline);
            model::draw_trait::DrawModel::draw_model_instanced(
                &mut render_pass,
                &self.obj_models.get("plane").unwrap(),
                1..self.instances.len() as u32,
                &self.camera_bind_group,
                &self.light_bind_group,
            );
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub(crate) trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}
