use anyhow::{anyhow, Result};
use std::path::Path;

use gltf::{buffer::Buffer, buffer::Source, scene::Node};

fn visit_node(node: &Node) {
    let name = node.name().unwrap_or("--");
    println!("Node {} with index {}", name, node.index());

    for node in node.children() {
        println!("With children");
        visit_node(&node);
    }
}

const DATA_BASE64: &str = "data:application/gltf-buffer;base64,";

struct GltfBuffers(Vec<u8>, Vec<(usize, usize)>); // buffer, (start, end)

impl GltfBuffers {
    fn buffer(&self, buffer: &gltf::Buffer) -> Option<&[u8]> {
        self.1
            .get(buffer.index())
            .map(|(start, end)| &self.0[*start..*end])
    }
}

fn sum_buffer_sizes(gltf: &gltf::Gltf) -> usize {
    gltf.buffers().map(|buffer| buffer.length()).sum()
}

fn collect_buffers(gltf: &gltf::Gltf) -> GltfBuffers {
    let buffers = gltf.buffers();
    let buffer_cap = sum_buffer_sizes(gltf);
    let buffer_count = buffers.len();
    let (buffer, slices) = buffers
        .filter_map(|buffer| match buffer.source() {
            Source::Uri(data) if data.starts_with(DATA_BASE64) => Some(&data[DATA_BASE64.len()..]),
            _ => unimplemented!(),
        })
        .fold(
            (
                Vec::with_capacity(buffer_cap),
                Vec::with_capacity(buffer_count),
            ),
            |(mut v, mut slices), data| {
                let start = v.len();
                base64::decode_config_buf(data, base64::STANDARD, &mut v)
                    .expect("base 64 decode error");
                slices.push((start, v.len()));
                (v, slices)
            },
        );
    GltfBuffers(buffer, slices)
}

fn load_geometry(mesh: &gltf::Mesh, buffers: &GltfBuffers) -> Result<()> {
    for (prim_index, primitive) in mesh.primitives().enumerate() {
        let reader = primitive.reader(|index| buffers.buffer(&index));
        if let Some(indices) = reader.read_indices() {
            // we don't really need this, just to visualize
            let indices = indices.into_u32();
            let len = indices.len();
            let (_, indices) =
                indices.fold((0, Vec::with_capacity(len / 3)), |(mut i, mut v), index| {
                    if i == 0 {
                        v.push([0, 0, 0]);
                    }
                    v.last_mut().expect("at least one element")[i] = index;
                    i += 1;
                    i %= 3;
                    (i, v)
                });
            for (i, index) in indices.iter().enumerate() {
                println!("index {}: {:?}", i, index);
            }
        }
        if let Some(positions) = reader.read_positions() {
            for (i, pos) in positions.enumerate() {
                println!("prim {}, pos {:?}", i, pos);
            }
        }
        if let Some(normals) = reader.read_normals() {
            println!("{:?}", normals);
        }
        if let Some(tangents) = reader.read_tangents() {
            println!("{:?}", tangents);
        }
        if let Some(weights) = reader.read_weights(0) {
            for (i, weight) in weights.into_f32().enumerate() {
                println!("weight {}: {:?}", i, weight);
            }
        }
        if let Some(joints) = reader.read_joints(0) {
            for (i, joint) in joints.into_u16().enumerate() {
                println!("joint {}: {:?}", i, joint);
            }
        }
    }
    Ok(())
}

// struct JointRoot {
//     node: JointNode,
//     mesh_index: usize,
// }

// transform: gltf::scene::Transform,

#[derive(Debug)]
struct JointNode {
    index: usize,
    children: Vec<JointNode>,
}

impl From<&gltf::Node<'_>> for JointNode {
    fn from(node: &gltf::Node) -> Self {
        JointNode {
            index: node.index(),
            children: gather_children(&node, node.transform()),
        }
    }
}

fn gather_children(node: &gltf::Node, parent: gltf::scene::Transform) -> Vec<JointNode> {
    node.children().map(|node| JointNode::from(&node)).collect()
}

fn transform_forest(scene: gltf::scene::Scene) -> Vec<JointNode> {
    scene.nodes().map(|node| JointNode::from(&node)).collect()
}

fn to_global_transform(parent: gltf::scene::Transform) -> [[f32; 4]; 4] {
    parent.matrix()
}

use gltf::animation::{util::ReadOutputs, Animation};

fn load_animation(animation: &Animation<'_>, buffers: &GltfBuffers) {
    for channel in animation.channels() {
        let reader = channel.reader(|id| buffers.buffer(&id));
        if let Some(inputs) = reader.read_inputs() {
            println!("anim inputs");
            for (i, input) in inputs.enumerate() {
                println!("\tindex {} {:?}", i, input);
            }
        }
        if let Some(outputs) = reader.read_outputs() {
            match outputs {
                ReadOutputs::Translations(trans) => {}
                ReadOutputs::Rotations(rots) => {
                    for (i, rot) in rots.into_f32().enumerate() {
                        println!("Rotations {}, {:?}", i, rot);
                    }
                }
                ReadOutputs::Scales(scales) => {}
                ReadOutputs::MorphTargetWeights(morphs) => {}
            }
        }
    }
    for sampler in animation.samplers() {}
}

fn load_skins(skin: &gltf::Skin, buffers: &GltfBuffers) {
    let reader = skin.reader(|id| buffers.buffer(&id));
    if let Some(inv_bind_matrices) = reader.read_inverse_bind_matrices() {
        for mat in inv_bind_matrices {
            println!("inv_bind {:#?}", mat);
        }
    }
}

pub fn load_gltf<P: AsRef<Path>>(path: P) -> Result<()> {
    let gltf = gltf::Gltf::open(path)?;

    for scene in gltf.scenes() {
        let name = scene.name().unwrap_or("--");
        println!("Scene {} with index {}", name, scene.index());
        for node in scene.nodes() {
            visit_node(&node);
        }
    }

    let gltf_buffers = collect_buffers(&gltf);
    println!("Buffer total {}", gltf_buffers.0.len());
    for (index, (start, end)) in gltf_buffers.1.iter().cloned().enumerate() {
        println!("Buffer {} {:?}", index, &gltf_buffers.0[start..end]);
    }

    for mesh in gltf.meshes() {
        let _ = load_geometry(&mesh, &gltf_buffers);
    }
    for skin in gltf.skins() {
        load_skins(&skin, &gltf_buffers);
    }
    for scene in gltf.scenes() {
        let v = transform_forest(scene);
        println!("{:#?}", v);
    }

    for animation in gltf.animations() {
        load_animation(&animation, &gltf_buffers);
    }
    Ok(())
}
