use std::mem::size_of;

use gl::types::*;
use glam::{Mat4, Vec2, Vec3, Vec4};
use gltf::accessor::DataType;
use gltf::image::Format;
use gltf::Document;
use memoffset::offset_of;

use crate::texture::calculate_mip_levels;
use crate::utils::size_of_slice;
use crate::Result;

#[derive(Debug)]
pub struct Model {
    pub vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    texture_ids: Vec<GLuint>,

    pub drawable_nodes: Vec<DrawableNode>,
    pub materials: Vec<Material>,
}

impl Model {
    pub fn load(path: &str) -> Result<Model> {
        let (gltf, buffers, images) = gltf::import(path)?;

        // Get drawable nodes and primitives
        // Fill in the derivative vertex and index buffers manually
        let mut drawable_nodes = vec![];
        let mut vertices = vec![];
        let mut indices = vec![];
        for (node, transform) in
            NodesWithTransforms::from(&gltf).filter(|(node, _)| node.mesh().is_some())
        {
            let mut primitives = vec![];
            for primitive in node.mesh().unwrap().primitives() {
                let first_index = indices.len();
                let vertex_start = vertices.len();

                // Add vertices to our buffer
                {
                    // Gather all attributes in separate vecs (yes, inefficient)
                    let mut positions = vec![];
                    let mut normals = vec![];
                    let mut uvs = vec![];

                    for (attr, accessor) in primitive.attributes() {
                        let buffer_view = accessor.view().unwrap();
                        let offset = buffer_view.offset() + accessor.offset();
                        let stride =
                            buffer_view
                                .stride()
                                .unwrap_or_else(|| match accessor.data_type() {
                                    DataType::I8 => 1,
                                    DataType::U8 => 1,
                                    DataType::I16 => 2,
                                    DataType::U16 => 2,
                                    DataType::U32 => 4,
                                    DataType::F32 => 4,
                                });
                        let count = accessor.count();

                        let src_buffer = &buffers[buffer_view.buffer().index()];

                        unsafe {
                            // Get a pointer to the first element of attribute
                            let mut cursor = src_buffer.as_ptr().add(offset);

                            use gltf::accessor::Dimensions;
                            use gltf::Semantic::*;
                            match attr {
                                Positions => {
                                    assert_eq!(accessor.dimensions(), Dimensions::Vec3);
                                    for _ in 0..count {
                                        let position = cursor as *const Vec3;
                                        positions.push(*position);
                                        cursor = cursor.add(stride);
                                    }
                                }
                                Normals => {
                                    assert_eq!(accessor.dimensions(), Dimensions::Vec3);
                                    for _ in 0..count {
                                        let normal = cursor as *const Vec3;
                                        normals.push(*normal);
                                        cursor = cursor.add(stride);
                                    }
                                }
                                TexCoords(0) => {
                                    assert_eq!(accessor.dimensions(), Dimensions::Vec2);
                                    for _ in 0..count {
                                        let uv = cursor as *const Vec2;
                                        uvs.push(*uv);
                                        cursor = cursor.add(stride);
                                    }
                                }
                                TexCoords(_) => unimplemented!("Only one texture is supported"),
                                _ => {}
                            }
                        }
                    }
                    assert_eq!(positions.len(), normals.len());
                    assert_eq!(positions.len(), uvs.len());

                    for i in 0..positions.len() {
                        vertices.push(Vertex {
                            pos: positions[i],
                            normal: normals[i],
                            uv: uvs[i],
                        })
                    }
                }

                // Add indices to our buffer
                let accessor = primitive.indices().unwrap();
                {
                    let buffer_view = accessor.view().unwrap();
                    let offset = buffer_view.offset() + accessor.offset();
                    let stride = buffer_view.stride().unwrap_or(0);
                    assert_eq!(stride, 0); // support only tightly packed indices

                    let src_buffer = &buffers[buffer_view.buffer().index()];

                    match accessor.data_type() {
                        // NOTE: some duplication
                        DataType::U32 => unsafe {
                            let src_buffer = src_buffer.as_ptr().add(offset) as *const u32;
                            for i in 0..accessor.count() {
                                let index = {
                                    let index_ptr = src_buffer.add(i);

                                    *index_ptr
                                };
                                indices.push(vertex_start as u32 + index);
                            }
                        },
                        DataType::U16 => unsafe {
                            let src_buffer = src_buffer.as_ptr().add(offset) as *const u16;
                            for i in 0..accessor.count() {
                                let index = {
                                    let index_ptr = src_buffer.add(i);

                                    *index_ptr
                                };
                                indices.push(vertex_start as u32 + index as u32);
                            }
                        },
                        _ => unimplemented!("We only support indices of types U16 and U32"),
                    }
                }
                assert_eq!(indices.len(), first_index + accessor.count());

                primitives.push(Primitive {
                    first_index,
                    index_count: accessor.count(),
                    material_index: primitive.material().index().unwrap(),
                });
            }

            drawable_nodes.push(DrawableNode {
                primitives,
                transform,
            });
        }

        // Send the vertex and index buffers to GPU
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        let mut ebo: GLuint = 0;
        unsafe {
            gl::CreateVertexArrays(1, &mut vao);
            gl::CreateBuffers(1, &mut vbo);
            gl::CreateBuffers(1, &mut ebo);

            // Attach buffers to vao
            gl::VertexArrayVertexBuffer(vao, 0, vbo, 0, size_of::<Vertex>() as i32);
            gl::VertexArrayElementBuffer(vao, ebo);

            // Position
            gl::VertexArrayAttribFormat(
                vao,
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                offset_of!(Vertex, pos) as u32,
            );

            // Normal
            gl::VertexArrayAttribFormat(
                vao,
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                offset_of!(Vertex, normal) as u32,
            );

            // UV
            gl::VertexArrayAttribFormat(
                vao,
                2,
                2,
                gl::UNSIGNED_BYTE,
                gl::FALSE,
                offset_of!(Vertex, uv) as u32,
            );

            gl::EnableVertexArrayAttrib(vao, 0);
            gl::EnableVertexArrayAttrib(vao, 1);
            gl::EnableVertexArrayAttrib(vao, 2);

            gl::VertexArrayAttribBinding(vao, 0, 0);
            gl::VertexArrayAttribBinding(vao, 1, 0);
            gl::VertexArrayAttribBinding(vao, 2, 0);

            // Vertex data
            gl::NamedBufferStorage(
                vbo,
                size_of_slice(&vertices) as isize,
                vertices.as_ptr() as *const _,
                0,
            );

            // Index data
            gl::NamedBufferStorage(
                ebo,
                size_of_slice(&indices) as isize,
                indices.as_ptr() as *const _,
                0,
            );
        }

        // Load textures
        let num_textures = images.len();
        let mut texture_ids = Vec::with_capacity(num_textures);
        unsafe {
            texture_ids.set_len(num_textures);
            gl::CreateTextures(
                gl::TEXTURE_2D,
                num_textures as i32,
                texture_ids.as_mut_ptr(),
            );
            for &texture in &texture_ids {
                // Default sampler parameters
                gl::TextureParameteri(texture, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
                gl::TextureParameteri(texture, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
                gl::TextureParameteri(
                    texture,
                    gl::TEXTURE_MIN_FILTER,
                    gl::LINEAR_MIPMAP_LINEAR as GLint,
                );
                gl::TextureParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            }
        }
        for (image, texture_id) in images.into_iter().zip(&texture_ids) {
            let texture = *texture_id;
            unsafe {
                gl::TextureStorage2D(
                    texture,
                    calculate_mip_levels(image.width as usize, image.height as usize),
                    gl::SRGB8,
                    image.width as i32,
                    image.height as i32,
                );
                let format = match image.format {
                    Format::B8G8R8A8 => gl::BGRA,
                    Format::R8G8B8A8 => gl::RGBA,
                    Format::R8G8B8 => gl::RGB,
                    Format::B8G8R8 => gl::BGR,
                    _ => panic!("Unsupported texture format"),
                };
                if format == gl::RGB || format == gl::BGR {
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                }
                gl::TextureSubImage2D(
                    texture,
                    0,
                    0,
                    0,
                    image.width as i32,
                    image.height as i32,
                    format,
                    gl::UNSIGNED_BYTE,
                    image.pixels.as_ptr() as *const _,
                );
                gl::GenerateTextureMipmap(texture);
                if format == gl::RGB || format == gl::BGR {
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
                }
            }
        }

        // Fill in textures
        let textures = gltf
            .textures()
            .map(|texture| texture_ids[texture.source().index()])
            .collect::<Vec<_>>();

        // Load materials
        let materials = gltf
            .materials()
            .map(|material| {
                let pbr = material.pbr_metallic_roughness();
                let texture_index = pbr.base_color_texture().unwrap().texture().index();
                Material {
                    base_color_factor: Vec4::from(pbr.base_color_factor()),
                    base_color_texture: textures[texture_index],
                }
            })
            .collect::<Vec<_>>();

        Ok(Model {
            vao,
            vbo,
            ebo,
            texture_ids,

            drawable_nodes,
            materials,
        })
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteTextures(self.texture_ids.len() as i32, self.texture_ids.as_ptr());
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Vertex {
    pos: Vec3,
    normal: Vec3,
    uv: Vec2,
}

#[derive(Debug)]
pub struct Material {
    pub base_color_factor: Vec4,
    pub base_color_texture: GLuint,
}

#[derive(Debug)]
pub struct Texture {
    pub image_index: usize,
}

#[derive(Debug)]
pub struct Primitive {
    pub first_index: usize,
    pub index_count: usize,
    pub material_index: usize,
}

#[derive(Debug)]
pub struct DrawableNode {
    pub primitives: Vec<Primitive>,
    pub transform: Mat4,
}

impl DrawableNode {
    /// TODO: do we need it?
    pub fn transform_as_bytes(&self) -> &[u8] {
        let data_ptr = self.transform.as_ref().as_ptr() as *const u8;
        let byte_len = std::mem::size_of::<Mat4>();

        unsafe { std::slice::from_raw_parts(data_ptr, byte_len) }
    }
}

/// Iterator over all the nodes in the default scene which have a mesh.
/// Returns nodes together with their final transforms.
struct NodesWithTransforms<'a> {
    root_nodes: gltf::scene::iter::Nodes<'a>, // sadly, can't put it into the vec below
    children_iters: Vec<gltf::scene::iter::Children<'a>>,
    transforms: Vec<Mat4>,
    current_transform: Mat4,
}

impl<'a> NodesWithTransforms<'a> {
    fn from(gltf: &'a Document) -> Self {
        NodesWithTransforms {
            root_nodes: gltf.default_scene().unwrap().nodes(),
            children_iters: vec![],
            transforms: vec![],
            current_transform: Mat4::IDENTITY,
        }
    }
}

impl<'a> Iterator for NodesWithTransforms<'a> {
    type Item = (gltf::Node<'a>, Mat4);

    fn next(&mut self) -> Option<Self::Item> {
        // Get rid of all exhausted children iterators
        while let Some(children) = self.children_iters.last_mut() {
            if children.len() > 0 {
                break; // have some children left
            } else {
                self.children_iters.pop(); // exhausted
                self.current_transform = self.transforms.pop().unwrap();
            }
        }

        let node = if let Some(children) = self.children_iters.last_mut() {
            // At this point we either have a child node or no children iterators at all
            children.next()
        } else {
            // No more active children iterators left
            self.root_nodes.next()
        };
        if let Some(node) = node {
            let node_transform = Mat4::from_cols_array_2d(&node.transform().matrix());
            self.current_transform *= node_transform;
            self.transforms.push(self.current_transform);
            self.children_iters.push(node.children());

            Some((node, self.current_transform))
        } else {
            None
        }
    }
}
