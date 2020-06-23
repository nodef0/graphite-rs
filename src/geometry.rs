use crate::render_types::VertexTexNormal;

pub struct Geometry {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub num_indices: u32,
}

impl Geometry {
    pub fn new<T>(device: &wgpu::Device, vertices: &[T], indices: &[u16]) -> Self
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        let vertex_buffer = device
            .create_buffer_with_data(bytemuck::cast_slice(&vertices), wgpu::BufferUsage::VERTEX);
        let num_vertices = vertices.len() as u32;

        let index_buffer = device
            .create_buffer_with_data(bytemuck::cast_slice(&indices), wgpu::BufferUsage::INDEX);
        let num_indices = indices.len() as u32;

        Geometry {
            vertex_buffer,
            index_buffer,
            num_vertices,
            num_indices,
        }
    }

    pub fn create_sphere_pbr(device: &wgpu::Device) -> Self {
        use std::f32::consts::PI;

        let mut vertices = vec![];
        let mut indices = vec![];

        const XS: u16 = 64;
        const YS: u16 = 64;

        for y in 0..=YS {
            for x in 0..=XS {
                let xs = (x as f32) / (XS as f32);
                let ys = (y as f32) / (YS as f32);
                let xp = (xs * 2. * PI).cos() * (ys * PI).sin();
                let yp = (ys * PI).cos();
                let zp = (xs * 2. * PI).sin() * (ys * PI).sin();
                vertices.push(VertexTexNormal {
                    position: [xp, yp, zp],
                    tex_coord: [xs, ys],
                    normal: [xp, yp, zp],
                })
            }
        }

        for y in 0..YS {
            if y & 1 == 0 {
                for x in 0..=XS {
                    indices.push(y * (XS + 1) + x);
                    indices.push((y + 1) * (XS + 1) + x);
                }
            } else {
                for x in (0..=XS).rev() {
                    indices.push((y + 1) * (XS + 1) + x);
                    indices.push(y * (XS + 1) + x);
                }
            }
        }
        Geometry::new(&device, &vertices, &indices)
    }
}
