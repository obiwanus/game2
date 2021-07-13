use glam::Vec3;

pub struct Terrain {
    vertices: Vec<Vertex>,
}

impl Terrain {
    pub fn new(size: f32, cells: usize) -> Self {
        let mut vertices = vec![];
        let cell_size = size / cells as f32;
        let start_x = -size / 2.0;
        let start_z = -size / 2.0;

        for x in 0..cells {
            for z in 0..cells {
                let pos = Vec3::new(
                    start_x + x as f32 * cell_size,
                    0.0,
                    start_z + z as f32 * cell_size,
                );
                vertices.push(Vertex { pos });
            }
        }

        Terrain { vertices }
    }
}

#[repr(C)]
struct Vertex {
    pos: Vec3,
}
