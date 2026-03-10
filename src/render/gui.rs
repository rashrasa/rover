use std::sync::Arc;

use egui::Context;
use egui_wgpu::{RendererOptions, ScreenDescriptor};
use wgpu::{
    CommandEncoder, Device, Operations, Queue, RenderPass, RenderPassColorAttachment,
    RenderPassDescriptor, TextureView,
};
use winit::window::Window;

pub struct EguiRenderer {
    state: egui_winit::State,
    window: Arc<Window>,
    builder: Box<dyn Fn() -> Box<dyn FnOnce(&mut egui::Ui) + 'static>>,
    renderer: egui_wgpu::Renderer,
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
        builder: impl FnOnce(&mut egui::Ui) + 'static + Clone,
    ) -> Self {
        let builder = Box::new(builder);
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
            builder: Box::new(move || {
                let builder = builder.clone();

                Box::new(|ui| (builder)(ui))
            }),
            renderer,
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
            .show(self.state.egui_ctx(), (self.builder)());

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
}
