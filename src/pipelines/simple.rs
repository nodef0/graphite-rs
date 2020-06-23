use crate::{
    camera::Camera,
    geometry::Geometry,
    pipelines,
    render::Render,
    render_types::{MvpUniforms, VertexDesc, VertexTex},
    texture,
};

pub struct SimpleLayout {
    texture_layout: wgpu::BindGroupLayout,
    uniform_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
}

impl SimpleLayout {
    fn new(device: &wgpu::Device) -> Self {
        let texture_layout = pipelines::single_texture_layout(device);
        let uniform_layout = pipelines::single_uniform_buffer_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&uniform_layout, &texture_layout],
        });

        SimpleLayout {
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
                vertex_buffers: &[VertexTex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        })
    }
}

pub struct Simple {
    pub pipeline: wgpu::RenderPipeline,
    pub layout: SimpleLayout,
}

impl Simple {
    pub fn new(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        let vs_src = include_str!("../../shaders/tex2D_vs.glsl");
        let fs_src = include_str!("../../shaders/tex2D_fs.glsl");

        let (vs_module, fs_module) =
            pipelines::compile_modules(&device, (vs_src, fs_src), "simple");

        let layout = SimpleLayout::new(&device);
        let pipeline = layout.create_render_pipeline(&device, &sc_desc, &vs_module, &fs_module);

        Simple { layout, pipeline }
    }
}

pub struct SimpleRenderPass<'a> {
    pub clear_color: wgpu::Color,
    pub state: &'a SimpleState,
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

        let texture_bind_group = &self.state.textures[self.state.texture_index].1;
        let geometry = &self.state.geometries[self.state.geometry_index];

        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, &self.state.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, &geometry.vertex_buffer, 0, 0);
        render_pass.set_index_buffer(&geometry.index_buffer, 0, 0);
        render_pass.draw_indexed(0..geometry.num_indices, 0, 0..1);
    }
}

pub struct SimpleState {
    pub uniforms: MvpUniforms,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,

    pub geometries: Vec<Geometry>,
    pub textures: Vec<(texture::Texture, wgpu::BindGroup)>,

    pub geometry_index: usize,
    pub texture_index: usize,
}

impl SimpleState {
    pub fn inc_geometry_index(&mut self) {
        self.geometry_index += 1;
        self.geometry_index %= self.geometries.len();
    }

    pub fn inc_texture_index(&mut self) {
        self.texture_index += 1;
        self.texture_index %= self.textures.len();
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

    pub fn new(device: &wgpu::Device, pipeline: &Simple, camera: &Camera) -> Self {
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

        SimpleState {
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            geometries: vec![],
            textures: vec![],
            geometry_index: 0,
            texture_index: 0,
        }
    }

    pub fn add_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pipeline: &Simple,
        image_bytes: &[u8],
    ) -> usize {
        let (texture, cmd_buffer) = texture::Texture::from_bytes(&device, image_bytes, false)
            .expect("Failed to create texture");
        queue.submit(&[cmd_buffer]);
        let bind_group = pipelines::single_texture_bind_group(
            &pipeline.layout.texture_layout,
            &device,
            &texture.view,
            &texture.sampler,
        );
        let index = self.textures.len();
        self.textures.push((texture, bind_group));
        index
    }

    pub fn add_geometry(
        &mut self,
        device: &wgpu::Device,
        vertices: &[VertexTex],
        indices: &[u16],
    ) -> usize {
        let index = self.geometries.len();
        self.geometries
            .push(Geometry::new(&device, &vertices, &indices));
        index
    }
}
