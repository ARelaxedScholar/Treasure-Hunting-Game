mod state;

use glfw::{fail_on_errors, Action, Context, Key, Window};
use wgpu::{
    util::{DeviceExt, RenderEncoder},
    Color, InstanceDescriptor,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // 1.
            step_mode: wgpu::VertexStepMode::Vertex,                            // 2.
            attributes: &[
                // 3.
                wgpu::VertexAttribute {
                    offset: 0,                             // 4.
                    shader_location: 0,                    // 5.
                    format: wgpu::VertexFormat::Float32x3, // 6.
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

fn update_vertex_buffer(
    device: &wgpu::Device,
    vertices: &[Vertex],
    vertex_buffer: &mut wgpu::Buffer,
) {
    *vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
}

#[repr(C)]
#[derive(Debug)]
struct Player<'a> {
    vertices: &'a mut [Vertex],
    indices: &'a [u16],
}

// Tileset variables
const ORIGINAL_TILE_SIZE: u32 = 16;
const SCALE: u32 = 3;

// Game Screen Variables
const MAX_SCREEN_COLUMNS: u32 = 16;
const MAX_SCREEN_ROWS: u32 = 12;

// Derived Constants
const TILE_SIZE: u32 = ORIGINAL_TILE_SIZE * SCALE;
const SCREEN_WIDTH: u32 = TILE_SIZE * MAX_SCREEN_COLUMNS;
const SCREEN_HEIGHT: u32 = TILE_SIZE * MAX_SCREEN_ROWS;

#[tokio::main]
async fn main() {
    let mut glfw = glfw::init(fail_on_errors).expect("Failed to create a glfw");

    let (mut window, events) = glfw
        .create_window(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "My First Game From Scratch",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create window.");

    window.make_current();
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);

    // Boiler plate for wgpu
    let size = window.get_size();
    let instance = wgpu::Instance::new(&InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        flags: wgpu::InstanceFlags::default(),
        backend_options: wgpu::BackendOptions::default(),
    });

    let target =
        unsafe { wgpu::SurfaceTargetUnsafe::from_window(&window) }.expect("Failed to get target");
    let surface = unsafe { instance.create_surface_unsafe(target) }.expect("Failed to get surface");

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to get adapter");
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
            },
            None,
        )
        .await
        .expect("Failed to get device & queue.");

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .find(|format| format.is_srgb())
        .copied()
        .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.0 as u32,
        height: size.1 as u32,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    // Use the same layout (is probably fine?)
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    // Default Pipeline
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Default Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[Vertex::desc()],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    });

    let mut player_vertices = vec![
        Vertex {
            position: [0., 0.25, 0.],
            color: [0., 1., 0.],
        },
        Vertex {
            position: [-0.25, 0.25, 0.],
            color: [1., 0., 0.],
        },
        Vertex {
            position: [-0.25, 0., 0.],
            color: [0., 0., 1.],
        },
        Vertex {
            position: [0., 0., 0.],
            color: [1., 0., 1.],
        },
    ];

    let indices = vec![0, 1, 2, 0, 2, 3];

    // Information I need
    let player = Player {
        vertices: player_vertices.as_mut_slice(),
        indices: indices.as_slice(),
    };

    // Setting up the drawing stuff
    let mut vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(player.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(player.indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    // Main Loop
    while !window.should_close() {
        glfw.poll_events();
        // User input
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                glfw::WindowEvent::Key(Key::Up, _, Action::Press | Action::Repeat, _) => {
                    player
                        .vertices
                        .iter_mut()
                        .for_each(|vertex| vertex.position[1] += 0.05);
                    update_vertex_buffer(&device, &player.vertices, &mut vertex_buffer);
                }
                glfw::WindowEvent::Key(Key::Down, _, Action::Press | Action::Repeat, _) => {
                    player
                        .vertices
                        .iter_mut()
                        .for_each(|vertex| vertex.position[1] -= 0.05);
                    update_vertex_buffer(&device, &player.vertices, &mut vertex_buffer);
                }
                glfw::WindowEvent::Key(Key::Left, _, Action::Press | Action::Repeat, _) => {
                    player
                        .vertices
                        .iter_mut()
                        .for_each(|vertex| vertex.position[0] -= 0.05);
                    update_vertex_buffer(&device, &player.vertices, &mut vertex_buffer);
                }
                glfw::WindowEvent::Key(Key::Right, _, Action::Press | Action::Repeat, _) => {
                    player
                        .vertices
                        .iter_mut()
                        .for_each(|vertex| vertex.position[0] += 0.05);
                    update_vertex_buffer(&device, &player.vertices, &mut vertex_buffer);
                }
                event => {
                    println!("entered here");
                    println!("{:?}", event);
                }
            }
        }

        // Print To Screen
        let output = surface
            .get_current_texture()
            .expect("Failed to get texture");
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Simple Render Encoder"),
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Simple Clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&render_pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..player.indices.len() as u32, 0, 0..1);
        drop(render_pass);

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
