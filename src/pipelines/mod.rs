pub mod equirect;
pub mod pbr;
pub mod simple;

use std::io::Cursor;
use wgpu::ShaderModule;

fn single_uniform_buffer_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStage::VERTEX,
            ty: wgpu::BindingType::UniformBuffer { dynamic: false },
        }],
        label: Some("uniform_bind_group_layout"),
    })
}

fn single_texture_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
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
        ],
        label: Some("single_texture_bg_layout"),
    })
}

fn single_texture_bind_group(
    layout: &wgpu::BindGroupLayout,
    device: &wgpu::Device,
    texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        bindings: &[
            wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::Binding {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: Some("bind_group"),
    })
}

fn single_uniform_bind_group(
    layout: &wgpu::BindGroupLayout,
    device: &wgpu::Device,
    uniform_buffer: &wgpu::Buffer,
    size: usize,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        bindings: &[wgpu::Binding {
            binding: 0,
            resource: wgpu::BindingResource::Buffer {
                buffer: &uniform_buffer,
                range: 0..size as wgpu::BufferAddress,
            },
        }],
        label: Some("uniform_bind_group"),
    })
}

fn compile_modules(
    device: &wgpu::Device,
    (vs, fs): (&str, &str),
    tag: &str,
) -> (ShaderModule, ShaderModule) {
    use shaderc::{Compiler, ShaderKind};

    let mut compiler = Compiler::new().expect("failed to create shaderc");
    let mut create_mod = |source, kind, tag| {
        let spirv = compiler
            .compile_into_spirv(source, kind, tag, "main", None)
            .expect("failed to compile vs");

        let data =
            wgpu::read_spirv(Cursor::new(spirv.as_binary_u8())).expect("Failed to read vs spirv");
        device.create_shader_module(&data)
    };

    let vs_tag = format!("{}_vs.glsl", tag);
    let fs_tag = format!("{}_fs.glsl", tag);
    let vs_module = create_mod(vs, ShaderKind::Vertex, &vs_tag);
    let fs_module = create_mod(fs, ShaderKind::Fragment, &fs_tag);

    (vs_module, fs_module)
}
