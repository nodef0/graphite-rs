use crate::{
    camera::Camera,
    geometry::Geometry,
    pipelines,
    render::Render,
    render_types::{
        MaterialInfoRaw, MvpUniforms, PbrFragmentUniforms, TransformRaw, VertexDesc,
        VertexTexNormal,
    },
    texture::Texture,
};

pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    let mut x = value;
    if x < min {
        x = min;
    }
    if x > max {
        x = max;
    }
    x
}

pub struct Material {
    pub albedo: Texture,
    pub roughness: Texture,
    pub ambient_occlusion: Texture,
    pub normals: Texture,
    pub metallic: Texture,
}

pub struct PbrLayout {
    texture_layout: wgpu::BindGroupLayout,
    uniform_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
}

impl PbrLayout {
    fn texture_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                // albedo
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Uint,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                },
                // roughness
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Uint,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                },
                // ambient_occlusion
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Uint,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                },
                // normals
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Uint,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                },
            ],
            label: Some("texture_bind_group_layout"),
        })
    }

    fn uniform_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                // view_proj
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                // pbr params
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                // transform storage buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
                // material info storage buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
            ],
            label: Some("uniform_bind_group_layout"),
        })
    }

    fn new(device: &wgpu::Device) -> Self {
        let texture_layout = PbrLayout::texture_layout(device);
        let uniform_layout = PbrLayout::uniform_layout(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&uniform_layout, &texture_layout],
        });

        PbrLayout {
            texture_layout,
            uniform_layout,
            pipeline_layout,
        }
    }

    fn create_uniform_bind_group(
        &self,
        device: &wgpu::Device,
        mvp_buffer: &wgpu::Buffer,
        mvp_size: usize,
        pbr_fs_buffer: &wgpu::Buffer,
        pbr_fs_size: usize,
        transforms_buffer: &wgpu::Buffer,
        transforms_size: usize,
        material_info_buffer: &wgpu::Buffer,
        material_info_size: usize,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.uniform_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &mvp_buffer,
                        range: 0..mvp_size as wgpu::BufferAddress,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &pbr_fs_buffer,
                        range: 0..pbr_fs_size as wgpu::BufferAddress,
                    },
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &transforms_buffer,
                        range: 0..transforms_size as wgpu::BufferAddress,
                    },
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &material_info_buffer,
                        range: 0..material_info_size as wgpu::BufferAddress,
                    },
                },
            ],
            label: Some("pbr_uniform_bind_group"),
        })
    }

    fn create_texture_bind_group(
        &self,
        device: &wgpu::Device,
        material: &Material,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_layout,
            bindings: &[
                // albedo
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&material.albedo.view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&material.albedo.sampler),
                },
                // roughness
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&material.roughness.view),
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&material.roughness.sampler),
                },
                // ambient_occlusion
                wgpu::Binding {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&material.ambient_occlusion.view),
                },
                wgpu::Binding {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&material.ambient_occlusion.sampler),
                },
                // normals
                wgpu::Binding {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&material.normals.view),
                },
                wgpu::Binding {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&material.normals.sampler),
                },
            ],
            label: Some("bind_group"),
        })
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
                front_face: wgpu::FrontFace::Cw, // Ccw,
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
            // since we're rendering spheres by strips
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[VertexTexNormal::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        })
    }
}

pub struct Pbr {
    pub pipeline: wgpu::RenderPipeline,
    pub layout: PbrLayout,
}

impl Pbr {
    pub fn new(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        let vs_src = include_str!("../../shaders/pbr_vs.glsl");
        let fs_src = include_str!("../../shaders/pbr_fs.glsl");

        let (vs_module, fs_module) = pipelines::compile_modules(&device, (vs_src, fs_src), "pbr");

        let layout = PbrLayout::new(&device);
        let pipeline = layout.create_render_pipeline(&device, &sc_desc, &vs_module, &fs_module);

        Pbr { layout, pipeline }
    }
}

pub struct PbrRenderPass<'a> {
    pub clear_color: wgpu::Color,

    pub mvp: &'a mut MvpUniforms,
    pub mvp_buffer: &'a wgpu::Buffer,

    pub uniform_bind_group: &'a wgpu::BindGroup,
    pub geometry: &'a Geometry,

    pub depth_texture: &'a Texture,
    pub num_instances: usize,

    pub texture_bind_group: &'a wgpu::BindGroup,
}

impl<'a> PbrRenderPass<'a> {
    pub fn new(state: &'a mut PbrState) -> Self {
        PbrRenderPass {
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.1,
                b: 0.1,
                a: 1.0,
            },
            mvp: &mut state.mvp,
            mvp_buffer: &state.mvp_buffer,
            uniform_bind_group: &state.uniform_bind_group,
            geometry: &state.sphere,
            depth_texture: &state.depth_texture,
            num_instances: state.instances.0.len(),
            texture_bind_group: &state.material_bind_group,
        }
    }
}

impl Render for PbrRenderPass<'_> {
    fn render(
        &self,
        device: &wgpu::Device,
        pipeline: &wgpu::RenderPipeline,
        frame: &wgpu::SwapChainOutput,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        PbrState::stage_uniforms(device, encoder, &self.mvp, &self.mvp_buffer);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: self.clear_color,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture.view,
                depth_load_op: wgpu::LoadOp::Clear,
                depth_store_op: wgpu::StoreOp::Store,
                clear_depth: 1.0,
                stencil_load_op: wgpu::LoadOp::Clear,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_stencil: 0,
            }),
        });

        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.texture_bind_group, &[]);

        render_pass.set_vertex_buffer(0, &self.geometry.vertex_buffer, 0, 0);
        render_pass.set_index_buffer(&self.geometry.index_buffer, 0, 0);
        render_pass.draw_indexed(
            0..self.geometry.num_indices,
            0,
            0..self.num_instances as u32,
        );
    }
}

fn make_instances() -> (Vec<TransformRaw>, Vec<MaterialInfoRaw>) {
    const ROWS: i16 = 7;
    const COLS: i16 = 7;
    const SPACE: f32 = 2.5;

    (0..ROWS)
        .map(|row| {
            let metallic = (row as f32) / (ROWS as f32);
            (row, metallic)
        })
        .flat_map(|(row, metallic)| {
            (0..COLS).map(move |col| {
                let roughness = (col as f32) / (COLS as f32);
                let roughness = clamp(roughness, 0.05, 1.);
                let translation = cgmath::Vector3::new(
                    (col - (COLS / 2)) as f32 * SPACE,
                    (row - (ROWS / 2)) as f32 * SPACE,
                    0.,
                );
                (
                    TransformRaw {
                        model: cgmath::Matrix4::from_translation(translation),
                    },
                    MaterialInfoRaw {
                        info: cgmath::Vector4::new(metallic, roughness, 1.0, 0.0),
                    },
                )
            })
        })
        .unzip()
}

pub struct PbrState {
    pub mvp: MvpUniforms,
    pub pbr_fs: PbrFragmentUniforms,

    pub depth_texture: Texture,

    pub mvp_buffer: wgpu::Buffer,
    pub pbr_fs_buffer: wgpu::Buffer,

    pub uniform_bind_group: wgpu::BindGroup,

    pub sphere: Geometry,

    pub instances: (Vec<TransformRaw>, Vec<MaterialInfoRaw>),

    // pub albedo_tex: (Texture, wgpu::BindGroup),
    pub material: Material,
    pub material_bind_group: wgpu::BindGroup,
}

impl PbrState {
    pub fn new(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        queue: &wgpu::Queue,
        pipeline: &Pbr,
        camera: &Camera,
    ) -> Self {
        let hdr_bytes = std::fs::read("res/Subway_Lights/20_Subway_Lights_3k.hdr")
            .expect("could not hdr bytes");
        let hdr_vec =
            Texture::from_bytes(&device, &hdr_bytes, false).expect("could not read hdr env");

        // textures
        let load_texture = |name, is_normal| {
            let image_bytes = std::fs::read(format!("res/steelplate1/steelplate1_{}.png", name))
                .expect("could not load texture");
            let (texture, cmd_buffer) = Texture::from_bytes(&device, &image_bytes, is_normal)
                .expect("Failed to create texture");
            queue.submit(&[cmd_buffer]);
            texture
        };

        let albedo = load_texture("albedo", false);
        let roughness = load_texture("roughness", false);
        let metallic = load_texture("metallic", false);
        let normals = load_texture("normal-dx", true);
        let ambient_occlusion = load_texture("ao", false);
        let material = Material {
            albedo,
            roughness,
            metallic,
            normals,
            ambient_occlusion,
        };

        let material_bind_group = pipeline
            .layout
            .create_texture_bind_group(&device, &material);

        let depth_texture = Texture::create_depth_texture(&device, &sc_desc, "pbr_depth_texture");

        // instances
        let instances = make_instances();

        // uniforms
        let mut mvp = MvpUniforms::new();
        mvp.update_view_proj(&camera);

        let pbr_fs = PbrFragmentUniforms {
            albedo: [0.5, 0.0, 0.0, 0.0],
            light_positions: [
                [-10.0, 10.0, 10.0, 0.],
                [10.0, 10.0, 10.0, 0.],
                [-10.0, -10.0, 10.0, 0.],
                [10.0, -10.0, 10.0, 0.],
            ],
            light_colors: [
                [300.0, 300.0, 300.0, 0.],
                [300.0, 300.0, 300.0, 0.],
                [300.0, 300.0, 300.0, 0.],
                [300.0, 300.0, 300.0, 0.],
            ],
        };

        let mvp_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[mvp]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let pbr_fs_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[pbr_fs]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let (transforms_buffer, transforms_size) = (
            device.create_buffer_with_data(
                bytemuck::cast_slice(&instances.0),
                wgpu::BufferUsage::STORAGE_READ,
            ),
            instances.0.len() * std::mem::size_of::<TransformRaw>(),
        );

        let (material_info_buffer, material_info_size) = (
            device.create_buffer_with_data(
                bytemuck::cast_slice(&instances.1),
                wgpu::BufferUsage::STORAGE_READ,
            ),
            instances.0.len() * std::mem::size_of::<MaterialInfoRaw>(),
        );

        let uniform_bind_group = pipeline.layout.create_uniform_bind_group(
            &device,
            &mvp_buffer,
            std::mem::size_of_val(&mvp),
            &pbr_fs_buffer,
            std::mem::size_of_val(&pbr_fs),
            &transforms_buffer,
            transforms_size,
            &material_info_buffer,
            material_info_size,
        );

        let sphere = Geometry::create_sphere_pbr(&device);

        PbrState {
            mvp,
            pbr_fs,
            mvp_buffer,
            pbr_fs_buffer,
            uniform_bind_group,
            sphere,
            depth_texture,
            instances,
            material,
            material_bind_group,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) {
        self.depth_texture = Texture::create_depth_texture(&device, &sc_desc, "pbr_depth_texture");
    }

    pub fn stage_uniforms(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        mvp: &MvpUniforms,
        mvp_buffer: &wgpu::Buffer,
    ) {
        let staging_buffer = device
            .create_buffer_with_data(bytemuck::cast_slice(&[*mvp]), wgpu::BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &mvp_buffer,
            0,
            std::mem::size_of::<MvpUniforms>() as wgpu::BufferAddress,
        );
    }
}
