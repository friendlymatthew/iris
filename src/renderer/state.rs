use crate::renderer::feature_uniform::{FeatureUniform, TransformAction};
use crate::renderer::mouse_state::MouseState;
use crate::{
    png::grammar::Png,
    renderer::{Texture, Vertex},
};
use anyhow::{anyhow, Result};
use std::iter;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent, BlendState, Buffer,
    BufferBindingType, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, Device, DeviceDescriptor, Features, FragmentState, FrontFace,
    IndexFormat, Instance, InstanceDescriptor, Limits, LoadOp, MultisampleState, Operations,
    PipelineLayoutDescriptor, PolygonMode, PowerPreference, PrimitiveState, PrimitiveTopology,
    Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, SamplerBindingType, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StoreOp, Surface, SurfaceConfiguration, SurfaceError,
    TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension, VertexState,
};
use winit::window::CursorIcon;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

use super::draw_uniform::DrawUniform;
use super::shape::{compute_radius, Shape, ShapeStack};

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
];

const INDICES: &[u16] = &[
    0, 1, 2, // first triangle
    2, 1, 3, // second triangle
];

struct State<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    image_render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    #[allow(dead_code)]
    diffuse_texture: Texture,
    diffuse_bind_group: BindGroup,
    window: &'a Window,

    feature_uniform: FeatureUniform,
    feature_buffer: Buffer,
    feature_bind_group: BindGroup,

    draw_uniform: DrawUniform,
    draw_buffer: Buffer,
    draw_bind_group: BindGroup,

    mouse_state: MouseState,

    shape_stack: ShapeStack,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window, png: &'a Png) -> Result<State<'a>> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = Instance::new(InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("Failed to get adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        Limits::downlevel_webgl2_defaults()
                    } else {
                        Limits::default()
                    },
                    memory_hints: Default::default(),
                },
                None, // Trace path
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let diffuse_texture = Texture::from_bytes(&device, &queue, png)?;

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let feature_uniform = FeatureUniform::new(config.width, config.height, png.gamma);

        let feature_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Feature Buffer"),
            contents: bytemuck::cast_slice(&[feature_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let feature_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("feature_bind_group_layout"),
            });

        let feature_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &feature_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: feature_buffer.as_entire_binding(),
            }],
            label: Some("feature_bind_group"),
        });

        let draw_uniform = DrawUniform::new();

        let draw_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Draw Buffer"),
            contents: bytemuck::cast_slice(&[draw_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let draw_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("draw_bind_group_layout"),
        });

        let draw_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &draw_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: draw_buffer.as_entire_binding(),
            }],
            label: Some("draw_bind_group"),
        });

        let image_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("image_shader.wgsl").into()),
        });

        let image_render_pipeline_layout =
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &feature_bind_group_layout,
                    &draw_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let image_render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Image Render Pipeline"),
            layout: Some(&image_render_pipeline_layout),
            vertex: VertexState {
                module: &image_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &image_shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState {
                        color: BlendComponent::REPLACE,
                        alpha: BlendComponent::REPLACE,
                    }),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview renderer pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
            // Useful for optimizing shader compilation on Android
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;

        let mouse_state = MouseState::default();
        let shape_stack = ShapeStack::new();

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            image_render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_texture,
            diffuse_bind_group,
            window,
            feature_uniform,
            feature_buffer,
            feature_bind_group,
            draw_uniform,
            draw_buffer,
            draw_bind_group,
            mouse_state,
            shape_stack,
        })
    }

    pub const fn window(&self) -> &Window {
        self.window
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        // if new_size.width > 0 && new_size.height > 0 {
        //     let new_height = (new_size.width * self.config.height) / self.config.width;
        //     self.size = new_size;
        //     self.size.height = new_height;
        //
        //     self.config.width = new_size.width;
        //     self.config.height = new_height;
        //     self.surface.configure(&self.device, &self.config);
        // }

        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.feature_uniform
                .update_window_dimensions(self.config.width, self.config.height);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        let feature_uniform = &mut self.feature_uniform;
        let draw_uniform = &mut self.draw_uniform;

        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    let prev_state = self.mouse_state.pressed();

                    self.mouse_state
                        .set_pressed(matches!(state, ElementState::Pressed));

                    if !draw_uniform.crosshair() {
                        return true;
                    }

                    match (prev_state, self.mouse_state.pressed()) {
                        (false, true) => {
                            let (start_x, start_y) = self.mouse_state.position();

                            dbg!("start drag", start_x, start_y);
                            self.mouse_state.set_start_drag(Some((start_x, start_y)));
                            draw_uniform.set_circle_center(start_x, start_y);
                        }
                        (true, false) => {
                            let initial_drag_position = self.mouse_state.start_drag();

                            if initial_drag_position.is_none() {
                                panic!("Logic error occured. Mouse state once finished pressing doesn't have initial drag position set.");
                            }

                            let (x, y) = initial_drag_position.unwrap();
                            let (edge_x, edge_y) = self.mouse_state.position();
                            let radius = compute_radius((x, y), (edge_x, edge_y));
                            self.shape_stack.push(Shape::Circle { x, y, radius });

                            // clear state
                            self.mouse_state.set_start_drag(None);
                            dbg!("stop drag");
                            draw_uniform.set_circle_radius(0.0);
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x as f32, position.y as f32);

                if let Some(center) = self.mouse_state.start_drag() {
                    let radius = compute_radius(center, (x, y));
                    dbg!("dragging: radius", radius);
                    self.draw_uniform.set_circle_radius(radius);
                }

                self.mouse_state.update_position(x, y);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => match (keycode, state) {
                (KeyCode::KeyA, ElementState::Pressed) => {
                    draw_uniform.toggle_crosshair();

                    if draw_uniform.crosshair() {
                        self.window.set_cursor_icon(CursorIcon::Crosshair);
                    } else {
                        self.window.set_cursor_icon(CursorIcon::Default);
                    }
                }
                (KeyCode::KeyC, ElementState::Pressed) => {
                    feature_uniform.reset_features();
                }
                (KeyCode::KeyB, ElementState::Pressed) => {
                    feature_uniform.toggle_blur();
                }
                (KeyCode::ArrowUp, ElementState::Pressed) => {
                    if feature_uniform.blur() {
                        feature_uniform.increase_blur_radius();
                    }

                    if feature_uniform.sharpen() {
                        feature_uniform.increase_sharpen_factor();
                    }
                }
                (KeyCode::ArrowDown, ElementState::Pressed) => {
                    if feature_uniform.blur() {
                        feature_uniform.decrease_blur_radius();
                    }

                    if feature_uniform.sharpen() {
                        feature_uniform.decrease_sharpen_factor();
                    }
                }
                (KeyCode::KeyG, ElementState::Pressed) => {
                    feature_uniform.toggle_grayscale();
                }
                (KeyCode::KeyS, ElementState::Pressed) => {
                    feature_uniform.toggle_sharpen();
                }
                (KeyCode::KeyI, ElementState::Pressed) => {
                    feature_uniform.toggle_invert();
                }
                (KeyCode::KeyE, ElementState::Pressed) => {
                    feature_uniform.toggle_edge_detect();
                }
                (KeyCode::KeyX, ElementState::Pressed) => {
                    feature_uniform.apply_transform(TransformAction::FlipX);
                }
                (KeyCode::KeyY, ElementState::Pressed) => {
                    feature_uniform.apply_transform(TransformAction::FlipY);
                }
                _ => return false,
            },
            _ => return false,
        }

        true
    }

    fn update(&self) {
        self.queue.write_buffer(
            &self.feature_buffer,
            0,
            bytemuck::cast_slice(&[self.feature_uniform]),
        );

        self.queue.write_buffer(
            &self.draw_buffer,
            0,
            bytemuck::cast_slice(&[self.draw_uniform]),
        );
    }

    fn render(&self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.image_render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.feature_bind_group, &[]);
            render_pass.set_bind_group(2, &self.draw_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

            self.shape_stack.shapes().iter().for_each(|shape| {
                let &Shape::Circle { x, y, radius } = shape;

                let shape_uniform = DrawUniform {
                    crosshair: self.draw_uniform.crosshair,
                    circle_center_x: x,
                    circle_center_y: y,
                    circle_radius: radius,
                };

                self.queue.write_buffer(
                    &self.draw_buffer,
                    0,
                    bytemuck::cast_slice(&[shape_uniform]),
                );

                render_pass.set_bind_group(2, &self.draw_bind_group, &[]);
            });

            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[allow(clippy::future_not_send)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run(png: Png) -> anyhow::Result<()> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new()?;

    let (width, height) = png.dimensions();

    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(width, height))
        .with_title("iris")
        .build(&event_loop)?;

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(&window, &png).await?;
    let mut surface_configured = false;

    event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => {
                            surface_configured = true;
                            state.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            // This tells winit that we want another frame after this one
                            state.window().request_redraw();

                            if !surface_configured {
                                return;
                            }

                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if it's lost or outdated
                                Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                                    state.resize(state.size)
                                }
                                // The system is out of memory, we should probably quit
                                Err(SurfaceError::OutOfMemory) => {
                                    log::error!("OutOfMemory");
                                    control_flow.exit();
                                }

                                // This happens when a frame takes too long to present
                                Err(SurfaceError::Timeout) => {
                                    log::warn!("Surface timeout")
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}
