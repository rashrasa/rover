use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use egui::Context;
use egui_wgpu::{RendererOptions, ScreenDescriptor};
use serde_json::Value;
use wgpu::{
    CommandEncoder, Device, Operations, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    TextureView,
};
use winit::window::Window;

pub struct EguiRenderer {
    state: egui_winit::State,
    window: Arc<Window>,
    builder: fn(&mut egui::Ui, Arc<RwLock<HashMap<String, serde_json::Value>>>),
    renderer: egui_wgpu::Renderer,

    data: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl EguiRenderer {
    /// Provided [builder] will be cloned each time the UI needs to rebuild.
    /// Each variable captured in that closure will also be cloned each time
    /// and therefore every variable captured needs to implement Clone.
    pub fn new(
        device: &Device,
        texture_format: wgpu::TextureFormat,
        renderer_options: RendererOptions,
        window: Arc<Window>,
        builder: fn(&mut egui::Ui, Arc<RwLock<HashMap<String, serde_json::Value>>>),
    ) -> Self {
        let ctx = Context::default();

        let egui_state = egui_winit::State::new(
            ctx,
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(1024 * 2),
        );

        let renderer = egui_wgpu::Renderer::new(device, texture_format, renderer_options);

        Self {
            state: egui_state,
            window,
            builder,
            renderer,

            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        screen_descriptor: &ScreenDescriptor,
        encoder: &mut CommandEncoder,
        window_surface_view: &TextureView,
    ) {
        // Begin frame
        let input = self.state.take_egui_input(&self.window);
        self.state.egui_ctx().begin_pass(input);

        // Ui
        egui::containers::TopBottomPanel::bottom("root")
            .max_height(window_surface_view.texture().height() as f32 / 4.0)
            .frame(egui::Frame::NONE)
            .show(self.state.egui_ctx(), |ui| {
                (self.builder)(ui, self.data.clone())
            });

        // Draw
        let output = self.state.egui_ctx().end_pass();

        self.state
            .handle_platform_output(&self.window, output.platform_output);

        let tris = self
            .state
            .egui_ctx()
            .tessellate(output.shapes, self.state.egui_ctx().pixels_per_point());

        for (id, image_delta) in output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, id, &image_delta);
        }
        self.renderer
            .update_buffers(device, queue, encoder, &tris, screen_descriptor);

        let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("UI Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: window_surface_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.renderer
            .render(&mut render_pass.forget_lifetime(), &tris, screen_descriptor);

        for id in output.textures_delta.free {
            self.renderer.free_texture(&id);
        }
    }

    pub fn data(&self) -> Arc<RwLock<HashMap<String, Value>>> {
        self.data.clone()
    }
}
