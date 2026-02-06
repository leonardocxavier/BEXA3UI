use std::sync::Arc;

use bexa_ui_core::{
    clear_active_widgets, collect_focus_paths, dispatch_event, dispatch_scroll, draw_widgets,
    build_taffy, sync_styles, update_widget_measures, widget_mut_at_path, Renderer, Theme, WidgetNode,
};
use bytemuck::{Pod, Zeroable};
use glyphon::{Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextAtlas, TextRenderer, TextArea, TextBounds, Viewport, Weight};
use taffy::prelude::*;
use wgpu::util::DeviceExt;
use winit::event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowBuilder};

// Each vertex carries: position, UV within the quad, color+alpha,
// the rect dimensions in pixels, border_radius, border_width, border_color.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
    rect_center: [f32; 2],
    rect_half: [f32; 2],
    border_radius: f32,
    border_width: f32,
    border_color: [f32; 4],
}

impl Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem::size_of;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // uv
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 2]>() as u64,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // color (rgba)
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as u64,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // rect_center
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as u64,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // rect_half
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 10]>() as u64,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // border_radius
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 12]>() as u64,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32,
                },
                // border_width
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 13]>() as u64,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32,
                },
                // border_color
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 14]>() as u64,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

const SHADER_SRC: &str = r#"
struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) rect_center: vec2<f32>,
    @location(3) rect_half: vec2<f32>,
    @location(4) border_radius: f32,
    @location(5) border_width: f32,
    @location(6) border_color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) rect_center: vec2<f32>,
    @location(4) rect_half: vec2<f32>,
    @location(5) border_radius: f32,
    @location(6) border_width: f32,
    @location(7) border_color: vec4<f32>,
) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = uv;
    out.color = color;
    out.rect_center = rect_center;
    out.rect_half = rect_half;
    out.border_radius = border_radius;
    out.border_width = border_width;
    out.border_color = border_color;
    return out;
}

// SDF for a rounded rectangle
fn sdf_rounded_rect(p: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let r = min(radius, min(half_size.x, half_size.y));
    let q = abs(p) - half_size + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    // pixel position relative to rect center (in pixel coords)
    let p = in.uv * in.rect_half;

    let dist = sdf_rounded_rect(p, in.rect_half, in.border_radius);

    // Anti-alias edge (1px feather)
    let aa = 1.0;
    let fill_alpha = 1.0 - smoothstep(-aa, 0.0, dist);

    if fill_alpha < 0.001 {
        discard;
    }

    var final_color = in.color;

    // Border: blend border color in the border band
    if in.border_width > 0.0 {
        let inner_dist = dist + in.border_width;
        let border_mix = 1.0 - smoothstep(-aa, 0.0, inner_dist);
        // border_mix = 1.0 inside the fill, 0.0 outside
        // We want border color where dist is between -border_width and 0
        final_color = mix(in.border_color, in.color, border_mix);
    }

    final_color.a = final_color.a * fill_alpha;
    return final_color;
}
"#;

struct DrawBatch {
    start: u32,
    count: u32,
    clip: Option<(f32, f32, f32, f32)>,
}

pub struct App {
    root: WidgetNode,
    theme: Theme,
    title: String,
}

impl App {
    pub fn new(root: WidgetNode) -> Self {
        Self {
            root,
            theme: Theme::ocean(),
            title: "BexaUI".to_string(),
        }
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn run(self) {
        let event_loop = EventLoop::new().expect("create event loop");
        let window = Arc::new(
            WindowBuilder::new()
                .with_title(&self.title)
                .build(&event_loop)
                .expect("create window"),
        );

        let mut state = pollster::block_on(State::new(window, self.root, self.theme));

        event_loop
            .run(move |event, elwt| match event {
                Event::WindowEvent { event, window_id }
                    if window_id == state.window().id() => match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(size) => state.resize(size),
                        WindowEvent::CursorMoved { .. }
                        | WindowEvent::MouseInput { .. } => state.handle_window_event(&event),
                        WindowEvent::MouseWheel { delta, .. } => state.handle_mouse_wheel(delta),
                        WindowEvent::KeyboardInput { event, .. } => {
                            if event.state == ElementState::Pressed {
                                state.handle_keyboard_input(&event);
                            }
                        }
                        WindowEvent::ModifiersChanged(modifiers) => {
                            state.modifiers = modifiers.state();
                        }
                        WindowEvent::RedrawRequested => match state.render() {
                            Ok(()) => {}
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                            Err(err) => eprintln!("render error: {err:?}"),
                        },
                        _ => {}
                    },
                Event::AboutToWait => state.window().request_redraw(),
                _ => {}
            })
            .expect("run event loop");
    }
}

struct State {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_viewport: Viewport,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
    taffy: TaffyTree,
    root: WidgetNode,
    root_node: NodeId,
    renderer: Renderer,
    text_buffers: Vec<Buffer>,
    focus_paths: Vec<Vec<usize>>,
    focused_index: Option<usize>,
    modifiers: ModifiersState,
    cursor_pos: (f32, f32),
    draw_batches: Vec<DrawBatch>,
    theme: Theme,
}

impl State {
    async fn new(window: Arc<Window>, mut root: WidgetNode, theme: Theme) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("create surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("find GPU adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await
            .expect("create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quad SDF Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Quad Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Quad Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let mut font_system = FontSystem::new();
        // Embed and load the Nerd Font for icon support
        let nerd_font_data = include_bytes!("../../../assets/fonts/SymbolsNerdFont-Regular.ttf");
        font_system.db_mut().load_font_data(nerd_font_data.to_vec());
        let swash_cache = SwashCache::new();
        let text_cache = Cache::new(&device);
        let text_viewport = Viewport::new(&device, &text_cache);
        let mut text_atlas = TextAtlas::new(&device, &queue, &text_cache, config.format);
        let text_renderer =
            TextRenderer::new(&mut text_atlas, &device, wgpu::MultisampleState::default(), None);

        let mut taffy = TaffyTree::new();
        let root_node = build_taffy(&mut root, &mut taffy);
        let mut focus_paths = Vec::new();
        collect_focus_paths(&root, &mut Vec::new(), &mut focus_paths);

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Quad Vertex Buffer"),
            size: 4,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let mut state = Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            vertex_count: 0,
            font_system,
            swash_cache,
            text_viewport,
            text_atlas,
            text_renderer,
            taffy,
            root,
            root_node,
            renderer: Renderer::new(),
            text_buffers: Vec::new(),
            focus_paths,
            focused_index: None,
            modifiers: ModifiersState::default(),
            cursor_pos: (0.0, 0.0),
            draw_batches: Vec::new(),
            theme,
        };

        if !state.focus_paths.is_empty() {
            state.set_focus(Some(0));
        }

        state
    }

    fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn update_layout(&mut self) {
        let width = self.size.width as f32;
        let height = self.size.height as f32;
        if width == 0.0 || height == 0.0 {
            return;
        }

        sync_styles(&mut self.root, &mut self.taffy, width, height, true);
        let available_space = Size {
            width: AvailableSpace::Definite(width),
            height: AvailableSpace::Definite(height),
        };
        self.taffy
            .compute_layout(self.root_node, available_space)
            .expect("compute layout");
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.update_layout();

        let viewport = (self.size.width as f32, self.size.height as f32);
        self.renderer.clear();
        draw_widgets(&self.root, &self.taffy, &mut self.renderer);

        self.build_quad_vertices(viewport);
        self.text_viewport.update(
            &self.queue,
            Resolution {
                width: self.config.width,
                height: self.config.height,
            },
        );

        let text_areas = build_text_areas(
            &self.renderer.text_commands,
            &mut self.text_buffers,
            &mut self.font_system,
            &mut self.renderer.text_measures,
        );

        // Feed measured pixel widths back to TextInput widgets
        update_widget_measures(&mut self.root, &self.renderer.text_measures);

        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.text_atlas,
                &self.text_viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .expect("prepare text");

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
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.theme.background[0] as f64,
                            g: self.theme.background[1] as f64,
                            b: self.theme.background[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            let sw = self.size.width;
            let sh = self.size.height;
            for batch in &self.draw_batches {
                if let Some((cx, cy, cw, ch)) = batch.clip {
                    // Clamp scissor rect to render target bounds
                    let sx = (cx.max(0.0) as u32).min(sw);
                    let sy = (cy.max(0.0) as u32).min(sh);
                    let right = ((cx + cw).max(0.0) as u32).min(sw);
                    let bottom = ((cy + ch).max(0.0) as u32).min(sh);
                    let swidth = right.saturating_sub(sx);
                    let sheight = bottom.saturating_sub(sy);
                    if swidth == 0 || sheight == 0 {
                        continue; // Clip entirely outside viewport
                    }
                    render_pass.set_scissor_rect(sx, sy, swidth, sheight);
                } else {
                    render_pass.set_scissor_rect(0, 0, sw, sh);
                }
                render_pass.draw(batch.start..batch.start + batch.count, 0..1);
            }

            // Reset scissor for text rendering
            render_pass.set_scissor_rect(0, 0, sw, sh);
            self.text_renderer
                .render(&self.text_atlas, &self.text_viewport, &mut render_pass)
                .expect("render text");
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();

        self.text_atlas.trim();

        Ok(())
    }

    fn handle_window_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.cursor_pos = (position.x as f32, position.y as f32);
        }

        let mut path = Vec::new();
        if let Some(consumed_path) = dispatch_event(
            &mut self.root,
            &self.taffy,
            event,
            &mut path,
        ) {
            if matches!(event, WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. }) {
                self.set_focus_by_path(&consumed_path);
            }
        }
    }

    fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let delta_y = match delta {
            MouseScrollDelta::LineDelta(_, y) => y * 40.0,
            MouseScrollDelta::PixelDelta(d) => d.y as f32,
        };
        let (cx, cy) = self.cursor_pos;
        dispatch_scroll(&mut self.root, delta_y, cx, cy, &self.taffy);
    }

    fn handle_keyboard_input(&mut self, event: &winit::event::KeyEvent) {
        // 1. Try dispatching to the focused widget first
        if let Some(idx) = self.focused_index {
            if let Some(path) = self.focus_paths.get(idx).cloned() {
                if let Some(widget) = widget_mut_at_path(&mut self.root, &path) {
                    if widget.handle_key_event(event, self.modifiers) {
                        return; // widget consumed it
                    }
                }
            }
        }

        // 2. Global keyboard shortcuts (only if widget didn't consume)
        match &event.logical_key {
            Key::Named(NamedKey::Tab) => {
                let reverse = self.modifiers.shift_key();
                self.focus_next(reverse);
            }
            Key::Named(NamedKey::Enter) | Key::Named(NamedKey::Space) => {
                self.activate_focused();
            }
            Key::Named(NamedKey::Escape) => {
                self.clear_active();
            }
            _ => {}
        }
    }

    fn focus_next(&mut self, reverse: bool) {
        if self.focus_paths.is_empty() {
            return;
        }

        let count = self.focus_paths.len();
        let current = self.focused_index.unwrap_or(0);
        let next = if reverse {
            (current + count - 1) % count
        } else {
            (current + 1) % count
        };

        self.set_focus(Some(next));
    }

    fn activate_focused(&mut self) {
        let Some(index) = self.focused_index else {
            return;
        };

        if let Some(path) = self.focus_paths.get(index).cloned() {
            if let Some(widget) = widget_mut_at_path(&mut self.root, &path) {
                widget.activate();
            }
        }
    }

    fn clear_active(&mut self) {
        clear_active_widgets(&mut self.root);
    }

    fn set_focus(&mut self, index: Option<usize>) {
        self.focused_index = index;
        for (i, path) in self.focus_paths.iter().enumerate() {
            if let Some(widget) = widget_mut_at_path(&mut self.root, path) {
                widget.set_focus(Some(i) == index);
            }
        }
    }

    fn set_focus_by_path(&mut self, path: &[usize]) {
        if let Some(index) = self.focus_paths.iter().position(|p| p == path) {
            self.set_focus(Some(index));
        }
    }

    /// A draw batch: a range of vertices and optional scissor clip rect.
    fn build_quad_vertices(&mut self, viewport: (f32, f32)) {
        let mut vertices = Vec::with_capacity(self.renderer.quad_commands.len() * 6);
        let (vw, vh) = viewport;

        // Build draw batches grouped by clip rect
        self.draw_batches.clear();
        let mut current_clip: Option<(f32, f32, f32, f32)> = None;
        let mut batch_start: u32 = 0;

        for cmd in &self.renderer.quad_commands {
            // If clip changed, close current batch and start new one
            if cmd.clip != current_clip {
                let vert_count = vertices.len() as u32;
                if vert_count > batch_start {
                    self.draw_batches.push(DrawBatch {
                        start: batch_start,
                        count: vert_count - batch_start,
                        clip: current_clip,
                    });
                }
                current_clip = cmd.clip;
                batch_start = vert_count;
            }

            let (x, y, w, h) = cmd.rect;

            // NDC corners
            let x0 = (x / vw) * 2.0 - 1.0;
            let x1 = ((x + w) / vw) * 2.0 - 1.0;
            let y0 = 1.0 - (y / vh) * 2.0;
            let y1 = 1.0 - ((y + h) / vh) * 2.0;

            // Rect center and half-size in pixel space
            let cx = x + w * 0.5;
            let cy = y + h * 0.5;
            let hx = w * 0.5;
            let hy = h * 0.5;

            let make_vertex = |px: f32, py: f32, u: f32, v: f32| Vertex {
                position: [px, py],
                uv: [u, v],
                color: cmd.color,
                rect_center: [cx, cy],
                rect_half: [hx, hy],
                border_radius: cmd.border_radius,
                border_width: cmd.border_width,
                border_color: cmd.border_color,
            };

            // Two triangles forming a quad
            vertices.push(make_vertex(x0, y1, -1.0, 1.0));
            vertices.push(make_vertex(x1, y1, 1.0, 1.0));
            vertices.push(make_vertex(x1, y0, 1.0, -1.0));

            vertices.push(make_vertex(x0, y1, -1.0, 1.0));
            vertices.push(make_vertex(x1, y0, 1.0, -1.0));
            vertices.push(make_vertex(x0, y0, -1.0, -1.0));
        }

        // Close last batch
        let vert_count = vertices.len() as u32;
        if vert_count > batch_start {
            self.draw_batches.push(DrawBatch {
                start: batch_start,
                count: vert_count - batch_start,
                clip: current_clip,
            });
        }

        self.vertex_count = vertices.len() as u32;
        if vertices.is_empty() {
            self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Quad Vertex Buffer"),
                size: 4,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
        } else {
            self.vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Quad Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        }
    }
}

fn build_text_areas<'a>(
    commands: &'a [bexa_ui_core::TextCommand],
    text_buffers: &'a mut Vec<Buffer>,
    font_system: &mut FontSystem,
    measures_out: &mut Vec<Vec<f32>>,
) -> Vec<TextArea<'a>> {
    let mut areas = Vec::with_capacity(commands.len());
    measures_out.clear();
    measures_out.resize(commands.len(), vec![]);

    if text_buffers.len() < commands.len() {
        for _ in text_buffers.len()..commands.len() {
            text_buffers.push(Buffer::new(font_system, Metrics::new(16.0, 20.0)));
        }
    }

    for (idx, command) in commands.iter().enumerate() {
        let buffer = &mut text_buffers[idx];
        *buffer = Buffer::new(font_system, command.metrics);

        let family = match &command.font_family {
            Some(name) => Family::Name(name),
            None => Family::SansSerif,
        };
        let attrs = Attrs::new().family(family).weight(Weight::MEDIUM);

        buffer.set_text(
            font_system,
            &command.text,
            &attrs,
            Shaping::Advanced,
            Some(command.align),
        );
        buffer.set_size(
            font_system,
            Some(command.bounds.0),
            Some(command.bounds.1),
        );
        buffer.shape_until_scroll(font_system, false);

        // Measure text widths at requested char positions
        if !command.measure_chars.is_empty() {
            let mut results = Vec::with_capacity(command.measure_chars.len());
            for &char_pos in &command.measure_chars {
                let byte_pos = command.text
                    .char_indices()
                    .nth(char_pos)
                    .map(|(i, _)| i)
                    .unwrap_or(command.text.len());

                let mut width = 0.0f32;
                for run in buffer.layout_runs() {
                    for glyph in run.glyphs.iter() {
                        if glyph.start < byte_pos {
                            width = glyph.x + glyph.w;
                        }
                    }
                }
                results.push(width);
            }
            measures_out[idx] = results;
        }
    }

    for (idx, command) in commands.iter().enumerate() {
        let buffer = &text_buffers[idx];

        let mut left = command.pos.0 as i32;
        let mut top = command.pos.1 as i32;
        let mut right = (command.pos.0 + command.bounds.0) as i32;
        let mut bottom = (command.pos.1 + command.bounds.1) as i32;

        // Apply clip rect to text bounds
        if let Some((cx, cy, cw, ch)) = command.clip {
            left = left.max(cx as i32);
            top = top.max(cy as i32);
            right = right.min((cx + cw) as i32);
            bottom = bottom.min((cy + ch) as i32);
        }

        // Skip text entirely outside clip
        if right <= left || bottom <= top {
            continue;
        }

        areas.push(TextArea {
            buffer,
            left: command.pos.0,
            top: command.pos.1,
            scale: 1.0,
            bounds: TextBounds { left, top, right, bottom },
            default_color: Color::rgb(command.color[0], command.color[1], command.color[2]),
            custom_glyphs: &[],
        });
    }

    areas
}
