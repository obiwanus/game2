use std::ops::Index;

use glam::Vec3;

const EPSILON: f32 = 0.00001;

#[derive(Debug)]
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

    pub fn hits_aabb(&self, aabb: &AABB) -> Option<AabbHit> {
        let mut ray_min = (aabb[self.sign_x].x - self.origin.x) * self.inv_direction.x;
        let mut ray_max = (aabb[1 - self.sign_x].x - self.origin.x) * self.inv_direction.x;

        let y_min = (aabb[self.sign_y].y - self.origin.y) * self.inv_direction.y;
        let y_max = (aabb[1 - self.sign_y].y - self.origin.y) * self.inv_direction.y;

        if (ray_min > y_max) || (y_min > ray_max) {
            return None;
        }
        if y_min > ray_min {
            ray_min = y_min;
        }
        if y_max < ray_max {
            ray_max = y_max;
        }
        let z_min = (aabb[self.sign_z].z - self.origin.z) * self.inv_direction.z;
        let z_max = (aabb[1 - self.sign_z].z - self.origin.z) * self.inv_direction.z;

        if (ray_min > z_max) || (z_min > ray_max) {
            return None;
        }
        if z_max < ray_max {
            ray_max = z_max;
        }
        if ray_max <= 0.0 {
            return None;
        }
        if z_min > ray_min {
            ray_min = z_min;
        }
        if ray_min < 0.0 {
            ray_min = 0.0;
        }

        Some(AabbHit {
            t_min: ray_min,
            t_max: ray_max,
        })
    }

    pub fn get_point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

#[derive(Debug)]
pub struct AabbHit {
    pub t_min: f32,
    pub t_max: f32,
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
        debug_assert!(min.x < max.x && min.y < max.y && min.z < max.z);
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

impl Index<usize> for AABB {
    type Output = Vec3;

    fn index(&self, index: usize) -> &Vec3 {
        if index == 0 {
            &self.min
        } else {
            &self.max
        }
    }
}

impl Default for AABB {
    fn default() -> AABB {
        AABB::empty()
    }
}
