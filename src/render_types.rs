use crate::camera::Camera;
use std::mem;
use wgpu::vertex_attr_array;

pub trait VertexDesc {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VertexPlain {
    pub position: [f32; 3],
}

impl VertexDesc for VertexPlain {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<VertexTex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &vertex_attr_array![0 => Float3],
        }
    }
}

unsafe impl bytemuck::Pod for VertexPlain {}
unsafe impl bytemuck::Zeroable for VertexPlain {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VertexTex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl VertexDesc for VertexTex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<VertexTex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &vertex_attr_array![0 => Float3, 1 => Float2],
        }
    }
}

unsafe impl bytemuck::Pod for VertexTex {}
unsafe impl bytemuck::Zeroable for VertexTex {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VertexTexNormal {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
    pub normal: [f32; 3],
}

impl VertexDesc for VertexTexNormal {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<VertexTexNormal>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &vertex_attr_array![
                0 => Float3, 1 => Float2, 2 => Float3
            ],
        }
    }
}

unsafe impl bytemuck::Pod for VertexTexNormal {}
unsafe impl bytemuck::Zeroable for VertexTexNormal {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MvpUniforms {
    view_position: cgmath::Vector4<f32>,
    view_proj: cgmath::Matrix4<f32>,
    model: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for MvpUniforms {}
unsafe impl bytemuck::Zeroable for MvpUniforms {}

impl MvpUniforms {
    #[rustfmt::skip]
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_position: cgmath::Zero::zero(),
            view_proj: cgmath::Matrix4::identity(),
            model: cgmath::Matrix4::identity(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_position = camera.eye.to_homogeneous();
        self.view_proj = camera.build_view_projection_matrix()
    }

    pub fn update_model(&mut self, model: &cgmath::Matrix4<f32>) {
        self.model = *model;
    }

    #[rustfmt::skip]
    pub fn update_model_rotation(&mut self, angle: f32) {
        self.model = cgmath::Matrix4::new(
            angle.cos(), -angle.sin(), 0.0, 0.0,
            angle.sin(),  angle.cos(), 0.0, 0.0,
                    0.0,          0.0, 1.0, 0.0,
                    0.0,          0.0, 0.0, 1.0,
        );
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct PbrFragmentUniforms {
    pub albedo: [f32; 4],
    pub light_positions: [[f32; 4]; 4],
    pub light_colors: [[f32; 4]; 4],
}

unsafe impl bytemuck::Pod for PbrFragmentUniforms {}
unsafe impl bytemuck::Zeroable for PbrFragmentUniforms {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MaterialInfoRaw {
    pub info: cgmath::Vector4<f32>, // metallic, roughness, ao, padding
}

unsafe impl bytemuck::Pod for MaterialInfoRaw {}
unsafe impl bytemuck::Zeroable for MaterialInfoRaw {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TransformRaw {
    pub model: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for TransformRaw {}
unsafe impl bytemuck::Zeroable for TransformRaw {}
