use crate::render_types::VertexTex;

#[allow(clippy::approx_constant, clippy::eq_op)]

pub const CIRCLE_VERTICES: &[VertexTex] = &[
    VertexTex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.5, 1. - 0.5],
    }, // A
    VertexTex {
        position: [0.0, 1.0, 0.0],
        tex_coords: [0.5, 1. - 1.0],
    }, // B
    VertexTex {
        position: [-0.7071, 0.7071, 0.0],
        tex_coords: [0.1465, 1. - 0.8535],
    }, // C
    VertexTex {
        position: [-1.0, 0.0, 0.0],
        tex_coords: [0.0, 1. - 0.5],
    }, // D
    VertexTex {
        position: [-0.7071, -0.7071, 0.0],
        tex_coords: [0.1465, 1. - 0.1465],
    }, // E
    VertexTex {
        position: [0.0, -1.0, 0.0],
        tex_coords: [0.5, 1. - 0.0],
    }, // F
    VertexTex {
        position: [0.7071, -0.7071, 0.0],
        tex_coords: [0.8535, 1. - 0.1465],
    }, // G
    VertexTex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 1. - 0.5],
    }, // H
    VertexTex {
        position: [0.7071, 0.7071, 0.0],
        tex_coords: [0.8535, 1. - 0.8535],
    }, // I
];

pub const CIRCLE_INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 6, 7, 0, 7, 8, 0, 8, 1,
];

pub const PENTAGON_VERTICES: &[VertexTex] = &[
    VertexTex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 1.0 - 0.99240386],
    }, // A
    VertexTex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 1.0 - 0.56958646],
    }, // B
    VertexTex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 1.0 - 0.050602943],
    }, // C
    VertexTex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 1.0 - 0.15267089],
    }, // D
    VertexTex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 1.0 - 0.7347359],
    }, // E
];

pub const PENTAGON_INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
