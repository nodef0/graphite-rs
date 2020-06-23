use crate::{
    camera::Camera,
    geometry::Geometry,
    pipelines,
    render::Render,
    render_types::{MvpUniforms, VertexDesc, VertexPlain},
    texture,
};

pub struct EquirectLayout {
    texture_layout: wgpu::BindGroupLayout,
    uniform_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
}

impl EquirectLayout {
    fn new(device: &wgpu::Device) -> Self {
        let texture_layout = pipelines::single_texture_layout(device);
        let uniform_layout = pipelines::single_uniform_buffer_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&uniform_layout, &texture_layout],
        });

        EquirectLayout {
            texture_layout,
            uniform_layout,
            pipeline_layout,
        }
    }

    fn create_render_pipeline(
        &self,
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        vs_module: &wgpu::ShaderModule,
        fs_module: &wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &self.pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[VertexPlain::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        })
    }
}

pub struct Equirect {
    pub pipeline: wgpu::RenderPipeline,
    pub layout: EquirectLayout,
}

impl Equirect {
    pub fn new(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        let vs_src = include_str!("../../shaders/equirect_vs.glsl");
        let fs_src = include_str!("../../shaders/equirect_fs.glsl");
        let (vs_module, fs_module) = pipelines::compile_modules(&device, (vs_src, fs_src), "pbr");

        let layout = EquirectLayout::new(&device);
        let pipeline = layout.create_render_pipeline(&device, &sc_desc, &vs_module, &fs_module);

        Equirect { layout, pipeline }
    }
}

pub struct SimpleRenderPass<'a> {
    pub clear_color: wgpu::Color,
    pub state: &'a EquirectState,
}

impl Render for SimpleRenderPass<'_> {
    fn render(
        &self,
        _device: &wgpu::Device,
        pipeline: &wgpu::RenderPipeline,
        frame: &wgpu::SwapChainOutput,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: self.clear_color,
            }],
            depth_stencil_attachment: None,
        });

        // let texture_bind_group = &self.state.texture_bind_group;

        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, &self.state.uniform_bind_group, &[]);
        // render_pass.set_bind_group(1, &texture_bind_group, &[]);
        // render_pass.set_vertex_buffer(0, &geometry.vertex_buffer, 0, 0);
        // render_pass.set_index_buffer(&geometry.index_buffer, 0, 0);
        // render_pass.draw_indexed(0..geometry.num_indices, 0, 0..1);
    }
}

pub struct EquirectState {
    pub uniforms: MvpUniforms,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    // pub texture: texture::Texture,
    // pub texture_bind_group: wgpu::BindGroup,
}

impl EquirectState {
    pub fn new(device: &wgpu::Device, pipeline: &Equirect, camera: &Camera) -> Self {
        let mut uniforms = MvpUniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[uniforms]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let uniform_bind_group = pipelines::single_uniform_bind_group(
            &pipeline.layout.uniform_layout,
            &device,
            &uniform_buffer,
            std::mem::size_of_val(&uniforms),
        );

        EquirectState {
            uniforms,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    pub fn update_uniforms(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &Camera,
        model_angle: f32,
    ) {
        self.uniforms.update_view_proj(&camera);
        self.uniforms.update_model_rotation(model_angle);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("update encoder"),
        });

        let staging_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[self.uniforms]),
            wgpu::BufferUsage::COPY_SRC,
        );

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.uniform_buffer,
            0,
            std::mem::size_of::<MvpUniforms>() as wgpu::BufferAddress,
        );

        queue.submit(&[encoder.finish()]);
    }
}
