use std::collections::HashMap;
use std::sync::Arc;

use bexa_ui_core::{
    build_taffy, clear_active_widgets, collect_focus_paths, dispatch_event, dispatch_scroll,
    draw_widgets, handle_scrollbar_event, release_scrollbar_drag, sync_styles,
    try_start_scrollbar_drag, update_widget_measures, widget_mut_at_path, Renderer, Theme,
    WidgetNode, WindowRequest, WindowRequests,
};
use bytemuck::{Pod, Zeroable};
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport, Weight,
};
use taffy::prelude::*;
use wgpu::util::DeviceExt;
use winit::event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowBuilder, WindowId};

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
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 2]>() as u64,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as u64,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as u64,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 10]>() as u64,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 12]>() as u64,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 13]>() as u64,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32,
                },
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

fn sdf_rounded_rect(p: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let r = min(radius, min(half_size.x, half_size.y));
    let q = abs(p) - half_size + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let p = in.uv * in.rect_half;
    let dist = sdf_rounded_rect(p, in.rect_half, in.border_radius);
    let aa = 1.0;
    let fill_alpha = 1.0 - smoothstep(-aa, 0.0, dist);

    if fill_alpha < 0.001 {
        discard;
    }

    var final_color = in.color;

    if in.border_width > 0.0 {
        let inner_dist = dist + in.border_width;
        let border_mix = 1.0 - smoothstep(-aa, 0.0, inner_dist);
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

// ── Shared GPU resources (one per application) ──────────────────────────

struct SharedGpu {
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_atlas: TextAtlas,
    surface_format: wgpu::TextureFormat,
}

// ── Per-window state ────────────────────────────────────────────────────

struct WindowState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    // Rendering
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    overlay_vertex_buffer: wgpu::Buffer,
    overlay_vertex_count: u32,
    draw_batches: Vec<DrawBatch>,
    overlay_draw_batches: Vec<DrawBatch>,
    text_renderer: TextRenderer,
    overlay_text_renderer: TextRenderer,
    text_viewport: Viewport,
    text_buffers: Vec<Buffer>,
    overlay_text_buffers: Vec<Buffer>,
    // Widget tree
    root: WidgetNode,
    taffy: TaffyTree,
    root_node: NodeId,
    renderer: Renderer,
    focus_paths: Vec<Vec<usize>>,
    focused_index: Option<usize>,
    modifiers: ModifiersState,
    cursor_pos: (f32, f32),
    theme: Theme,
    is_main: bool,
}

impl WindowState {
    fn new(
        window: Arc<Window>,
        mut root: WidgetNode,
        theme: Theme,
        gpu: &mut SharedGpu,
        is_main: bool,
    ) -> Self {
        let size = window.inner_size();

        let surface = gpu.instance
            .create_surface(window.clone())
            .expect("create surface");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: gpu.surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&gpu.device, &config);

        let text_cache = Cache::new(&gpu.device);
        let text_viewport = Viewport::new(&gpu.device, &text_cache);
        let text_renderer = TextRenderer::new(
            &mut gpu.text_atlas,
            &gpu.device,
            wgpu::MultisampleState::default(),
            None,
        );
        let overlay_text_renderer = TextRenderer::new(
            &mut gpu.text_atlas,
            &gpu.device,
            wgpu::MultisampleState::default(),
            None,
        );

        let mut taffy = TaffyTree::new();
        let root_node = build_taffy(&mut root, &mut taffy);
        let mut focus_paths = Vec::new();
        collect_focus_paths(&root, &mut Vec::new(), &mut focus_paths);

        let vertex_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Quad Vertex Buffer"),
            size: 4,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        let overlay_vertex_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Overlay Vertex Buffer"),
            size: 4,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let mut ws = Self {
            window,
            surface,
            config,
            size,
            vertex_buffer,
            vertex_count: 0,
            overlay_vertex_buffer,
            overlay_vertex_count: 0,
            draw_batches: Vec::new(),
            overlay_draw_batches: Vec::new(),
            text_renderer,
            overlay_text_renderer,
            text_viewport,
            text_buffers: Vec::new(),
            overlay_text_buffers: Vec::new(),
            root,
            taffy,
            root_node,
            renderer: Renderer::new(),
            focus_paths,
            focused_index: None,
            modifiers: ModifiersState::default(),
            cursor_pos: (0.0, 0.0),
            theme,
            is_main,
        };

        if !ws.focus_paths.is_empty() {
            ws.set_focus(Some(0));
        }

        ws
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, device: &wgpu::Device) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(device, &self.config);
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

    fn render(&mut self, gpu: &mut SharedGpu) -> Result<(), wgpu::SurfaceError> {
        self.update_layout();

        let viewport = (self.size.width as f32, self.size.height as f32);
        self.renderer.clear();
        draw_widgets(&self.root, &self.taffy, &mut self.renderer);

        self.build_quad_vertices(viewport, &gpu.device);
        self.build_overlay_vertices(viewport, &gpu.device);

        self.text_viewport.update(
            &gpu.queue,
            Resolution {
                width: self.config.width,
                height: self.config.height,
            },
        );

        let text_areas = build_text_areas(
            &self.renderer.text_commands,
            &mut self.text_buffers,
            &mut gpu.font_system,
            &mut self.renderer.text_measures,
        );

        update_widget_measures(&mut self.root, &self.renderer.text_measures);

        self.text_renderer
            .prepare(
                &gpu.device,
                &gpu.queue,
                &mut gpu.font_system,
                &mut gpu.text_atlas,
                &self.text_viewport,
                text_areas,
                &mut gpu.swash_cache,
            )
            .expect("prepare text");

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            gpu.device
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

            let sw = self.size.width;
            let sh = self.size.height;

            // Pass 1: Main quads
            render_pass.set_pipeline(&gpu.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            for batch in &self.draw_batches {
                if let Some((cx, cy, cw, ch)) = batch.clip {
                    let sx = (cx.max(0.0) as u32).min(sw);
                    let sy = (cy.max(0.0) as u32).min(sh);
                    let right = ((cx + cw).max(0.0) as u32).min(sw);
                    let bottom = ((cy + ch).max(0.0) as u32).min(sh);
                    let swidth = right.saturating_sub(sx);
                    let sheight = bottom.saturating_sub(sy);
                    if swidth == 0 || sheight == 0 {
                        continue;
                    }
                    render_pass.set_scissor_rect(sx, sy, swidth, sheight);
                } else {
                    render_pass.set_scissor_rect(0, 0, sw, sh);
                }
                render_pass.draw(batch.start..batch.start + batch.count, 0..1);
            }

            // Pass 2: Main text
            render_pass.set_scissor_rect(0, 0, sw, sh);
            self.text_renderer
                .render(&gpu.text_atlas, &self.text_viewport, &mut render_pass)
                .expect("render text");

            // Pass 3: Overlay quads
            if self.overlay_vertex_count > 0 {
                render_pass.set_pipeline(&gpu.render_pipeline);
                render_pass.set_vertex_buffer(0, self.overlay_vertex_buffer.slice(..));
                for batch in &self.overlay_draw_batches {
                    if let Some((cx, cy, cw, ch)) = batch.clip {
                        let sx = (cx.max(0.0) as u32).min(sw);
                        let sy = (cy.max(0.0) as u32).min(sh);
                        let right = ((cx + cw).max(0.0) as u32).min(sw);
                        let bottom = ((cy + ch).max(0.0) as u32).min(sh);
                        let swidth = right.saturating_sub(sx);
                        let sheight = bottom.saturating_sub(sy);
                        if swidth == 0 || sheight == 0 {
                            continue;
                        }
                        render_pass.set_scissor_rect(sx, sy, swidth, sheight);
                    } else {
                        render_pass.set_scissor_rect(0, 0, sw, sh);
                    }
                    render_pass.draw(batch.start..batch.start + batch.count, 0..1);
                }
            }
        }

        // Pass 4: Overlay text
        if !self.renderer.overlay_text_commands.is_empty() {
            let overlay_text_areas = build_text_areas(
                &self.renderer.overlay_text_commands,
                &mut self.overlay_text_buffers,
                &mut gpu.font_system,
                &mut vec![],
            );

            self.overlay_text_renderer
                .prepare(
                    &gpu.device,
                    &gpu.queue,
                    &mut gpu.font_system,
                    &mut gpu.text_atlas,
                    &self.text_viewport,
                    overlay_text_areas,
                    &mut gpu.swash_cache,
                )
                .expect("prepare overlay text");

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Overlay Text Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });

                let sw = self.size.width;
                let sh = self.size.height;
                render_pass.set_scissor_rect(0, 0, sw, sh);
                self.overlay_text_renderer
                    .render(&gpu.text_atlas, &self.text_viewport, &mut render_pass)
                    .expect("render overlay text");
            }
        }

        gpu.queue.submit(Some(encoder.finish()));
        output.present();
        gpu.text_atlas.trim();

        Ok(())
    }

    fn handle_window_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.cursor_pos = (position.x as f32, position.y as f32);
        }

        if handle_scrollbar_event(&mut self.root, &self.taffy, event) {
            return;
        }

        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } = event
        {
            let (cx, cy) = self.cursor_pos;
            if try_start_scrollbar_drag(&mut self.root, &self.taffy, cx, cy) {
                return;
            }
        }

        if let WindowEvent::MouseInput {
            state: ElementState::Released,
            button: MouseButton::Left,
            ..
        } = event
        {
            release_scrollbar_drag(&mut self.root);
        }

        let mut path = Vec::new();
        if let Some(consumed_path) =
            dispatch_event(&mut self.root, &self.taffy, event, &mut path)
        {
            if matches!(
                event,
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    ..
                }
            ) {
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
        if let Some(idx) = self.focused_index {
            if let Some(path) = self.focus_paths.get(idx).cloned() {
                if let Some(widget) = widget_mut_at_path(&mut self.root, &path) {
                    if widget.handle_key_event(event, self.modifiers) {
                        return;
                    }
                }
            }
        }

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

    fn build_quad_vertices(&mut self, viewport: (f32, f32), device: &wgpu::Device) {
        let mut vertices = Vec::with_capacity(self.renderer.quad_commands.len() * 6);
        let (vw, vh) = viewport;

        self.draw_batches.clear();
        let mut current_clip: Option<(f32, f32, f32, f32)> = None;
        let mut batch_start: u32 = 0;

        for cmd in &self.renderer.quad_commands {
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
            let x0 = (x / vw) * 2.0 - 1.0;
            let x1 = ((x + w) / vw) * 2.0 - 1.0;
            let y0 = 1.0 - (y / vh) * 2.0;
            let y1 = 1.0 - ((y + h) / vh) * 2.0;
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

            vertices.push(make_vertex(x0, y1, -1.0, 1.0));
            vertices.push(make_vertex(x1, y1, 1.0, 1.0));
            vertices.push(make_vertex(x1, y0, 1.0, -1.0));
            vertices.push(make_vertex(x0, y1, -1.0, 1.0));
            vertices.push(make_vertex(x1, y0, 1.0, -1.0));
            vertices.push(make_vertex(x0, y0, -1.0, -1.0));
        }

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
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Quad Vertex Buffer"),
                size: 4,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
        } else {
            self.vertex_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Quad Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        }
    }

    fn build_overlay_vertices(&mut self, viewport: (f32, f32), device: &wgpu::Device) {
        let mut vertices =
            Vec::with_capacity(self.renderer.overlay_quad_commands.len() * 6);
        let (vw, vh) = viewport;

        self.overlay_draw_batches.clear();
        let mut current_clip: Option<(f32, f32, f32, f32)> = None;
        let mut batch_start: u32 = 0;

        for cmd in &self.renderer.overlay_quad_commands {
            if cmd.clip != current_clip {
                let vert_count = vertices.len() as u32;
                if vert_count > batch_start {
                    self.overlay_draw_batches.push(DrawBatch {
                        start: batch_start,
                        count: vert_count - batch_start,
                        clip: current_clip,
                    });
                }
                current_clip = cmd.clip;
                batch_start = vert_count;
            }

            let (x, y, w, h) = cmd.rect;
            let x0 = (x / vw) * 2.0 - 1.0;
            let x1 = ((x + w) / vw) * 2.0 - 1.0;
            let y0 = 1.0 - (y / vh) * 2.0;
            let y1 = 1.0 - ((y + h) / vh) * 2.0;
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

            vertices.push(make_vertex(x0, y1, -1.0, 1.0));
            vertices.push(make_vertex(x1, y1, 1.0, 1.0));
            vertices.push(make_vertex(x1, y0, 1.0, -1.0));
            vertices.push(make_vertex(x0, y1, -1.0, 1.0));
            vertices.push(make_vertex(x1, y0, 1.0, -1.0));
            vertices.push(make_vertex(x0, y0, -1.0, -1.0));
        }

        let vert_count = vertices.len() as u32;
        if vert_count > batch_start {
            self.overlay_draw_batches.push(DrawBatch {
                start: batch_start,
                count: vert_count - batch_start,
                clip: current_clip,
            });
        }

        self.overlay_vertex_count = vertices.len() as u32;
        if vertices.is_empty() {
            self.overlay_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Overlay Vertex Buffer"),
                size: 4,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
        } else {
            self.overlay_vertex_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Overlay Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        }
    }
}

// ── App (public API) ────────────────────────────────────────────────────

pub struct App {
    root: WidgetNode,
    theme: Theme,
    title: String,
    window_requests: Option<WindowRequests>,
}

impl App {
    pub fn new(root: WidgetNode) -> Self {
        Self {
            root,
            theme: Theme::ocean(),
            title: "BexaUI".to_string(),
            window_requests: None,
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

    pub fn with_requests(mut self, requests: WindowRequests) -> Self {
        self.window_requests = Some(requests);
        self
    }

    /// Create a shared `WindowRequests` handle for widgets to request new windows.
    pub fn window_requests() -> WindowRequests {
        bexa_ui_core::create_window_requests()
    }

    pub fn run(self) {
        let event_loop = EventLoop::new().expect("create event loop");

        // Create initial window
        let window = Arc::new(
            WindowBuilder::new()
                .with_title(&self.title)
                .build(&event_loop)
                .expect("create window"),
        );

        // Initialize shared GPU resources
        let mut gpu = pollster::block_on(init_gpu(window.clone()));

        // Create main window state
        let main_ws = WindowState::new(window.clone(), self.root, self.theme, &mut gpu, true);
        let main_id = main_ws.window.id();

        let mut windows: HashMap<WindowId, WindowState> = HashMap::new();
        windows.insert(main_id, main_ws);

        let window_requests = self.window_requests;

        event_loop
            .run(move |event, elwt| {
                elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);
                match event {
                Event::WindowEvent {
                    event: ref win_event,
                    window_id,
                } => {
                    if let Some(ws) = windows.get_mut(&window_id) {
                        match win_event {
                            WindowEvent::CloseRequested => {
                                if ws.is_main {
                                    elwt.exit();
                                } else {
                                    windows.remove(&window_id);
                                }
                            }
                            WindowEvent::Resized(size) => {
                                ws.resize(*size, &gpu.device);
                            }
                            WindowEvent::CursorMoved { .. }
                            | WindowEvent::MouseInput { .. } => {
                                ws.handle_window_event(win_event);
                            }
                            WindowEvent::MouseWheel { delta, .. } => {
                                ws.handle_mouse_wheel(*delta);
                            }
                            WindowEvent::KeyboardInput { event, .. } => {
                                if event.state == ElementState::Pressed {
                                    ws.handle_keyboard_input(event);
                                }
                            }
                            WindowEvent::ModifiersChanged(modifiers) => {
                                ws.modifiers = modifiers.state();
                            }
                            WindowEvent::RedrawRequested => {
                                match ws.render(&mut gpu) {
                                    Ok(()) => {}
                                    Err(wgpu::SurfaceError::Lost) => {
                                        let size = ws.size;
                                        ws.resize(size, &gpu.device);
                                    }
                                    Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                    Err(_) => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::AboutToWait => {
                    // Process pending window creation requests
                    if let Some(ref reqs) = window_requests {
                        let pending: Vec<WindowRequest> = {
                            let mut lock = reqs.lock().unwrap();
                            lock.drain(..).collect()
                        };
                        for req in pending {
                            let new_window = Arc::new(
                                WindowBuilder::new()
                                    .with_title(&req.title)
                                    .with_inner_size(winit::dpi::LogicalSize::new(
                                        req.width, req.height,
                                    ))
                                    .build(elwt)
                                    .expect("create child window"),
                            );
                            let new_id = new_window.id();
                            let ws = WindowState::new(
                                new_window,
                                req.root,
                                req.theme,
                                &mut gpu,
                                false,
                            );
                            windows.insert(new_id, ws);
                        }
                    }

                    // Request redraw for all windows
                    for ws in windows.values() {
                        ws.window.request_redraw();
                    }
                }
                _ => {}
            }})
            .expect("run event loop");
    }
}

// ── GPU Initialization ──────────────────────────────────────────────────

async fn init_gpu(window: Arc<Window>) -> SharedGpu {
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
                format: surface_format,
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
    let nerd_font_data = include_bytes!("../../../assets/fonts/SymbolsNerdFont-Regular.ttf");
    font_system
        .db_mut()
        .load_font_data(nerd_font_data.to_vec());
    let swash_cache = SwashCache::new();
    let text_cache = Cache::new(&device);
    let _text_viewport = Viewport::new(&device, &text_cache);
    let text_atlas = TextAtlas::new(&device, &queue, &text_cache, surface_format);

    SharedGpu {
        instance,
        device,
        queue,
        render_pipeline,
        font_system,
        swash_cache,
        text_atlas,
        surface_format,
    }
}

// ── Text area builder (shared) ──────────────────────────────────────────

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

        if !command.measure_chars.is_empty() {
            let mut results = Vec::with_capacity(command.measure_chars.len());
            for &char_pos in &command.measure_chars {
                let byte_pos = command
                    .text
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

        if let Some((cx, cy, cw, ch)) = command.clip {
            left = left.max(cx as i32);
            top = top.max(cy as i32);
            right = right.min((cx + cw) as i32);
            bottom = bottom.min((cy + ch) as i32);
        }

        if right <= left || bottom <= top {
            continue;
        }

        areas.push(TextArea {
            buffer,
            left: command.pos.0,
            top: command.pos.1,
            scale: 1.0,
            bounds: TextBounds {
                left,
                top,
                right,
                bottom,
            },
            default_color: Color::rgb(command.color[0], command.color[1], command.color[2]),
            custom_glyphs: &[],
        });
    }

    areas
}
