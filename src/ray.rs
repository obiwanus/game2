use glam::Vec3;

const EPSILON: f32 = 0.00001;

pub struct Ray {
    origin: Vec3,
    direction: Vec3,
    inv_direction: Vec3,
    sign_x: usize,
    sign_y: usize,
    sign_z: usize,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        let direction = direction.normalize();
        Ray {
            origin,
            direction,
            inv_direction: Vec3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z),
            sign_x: (direction.x < 0.0) as usize,
            sign_y: (direction.y < 0.0) as usize,
            sign_z: (direction.z < 0.0) as usize,
        }
    }

    #[allow(clippy::many_single_char_names)]
    pub fn hits_triangle(&self, a: &Vec3, b: &Vec3, c: &Vec3) -> TriangleHit {
        let ab = *b - *a;
        let ac = *c - *a;
        let u_vec = self.direction.cross(ac);
        let det = ab.dot(u_vec);
        if det < EPSILON {
            return TriangleHit::new(f32::INFINITY, 0.0, 0.0);
        }
        let inv_det = 1.0 / det;
        let a_to_origin = self.origin - *a;
        let u = a_to_origin.dot(u_vec) * inv_det;
        if !(0.0..=1.0).contains(&u) {
            return TriangleHit::new(f32::INFINITY, u, 0.0);
        }
        let v_vec = a_to_origin.cross(ab);
        let v = self.direction.dot(v_vec) * inv_det;
        if v < 0.0 || (u + v) > 1.0 {
            return TriangleHit::new(f32::INFINITY, u, v);
        }
        let t = ac.dot(v_vec) * inv_det;
        if t > EPSILON {
            TriangleHit::new(t, u, v)
        } else {
            TriangleHit::new(f32::INFINITY, u, v)
        }
    }

    // pub fn hits_aabb()

    pub fn get_point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

pub struct TriangleHit {
    pub t: f32,
    pub u: f32,
    pub v: f32,
}

impl TriangleHit {
    pub fn new(t: f32, u: f32, v: f32) -> Self {
        TriangleHit { t, u, v }
    }
}

#[derive(Debug, Copy, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> AABB {
        AABB { min, max }
    }

    pub fn empty() -> AABB {
        AABB {
            min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    pub fn contains(&self, p: &Vec3) -> bool {
        p.x >= self.min.x
            && p.x <= self.max.x
            && p.y >= self.min.y
            && p.y <= self.max.y
            && p.z >= self.min.z
            && p.z <= self.max.z
    }
}

impl Default for AABB {
    fn default() -> AABB {
        AABB::empty()
    }
}
