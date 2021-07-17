use glam::Vec3;

const EPSILON: f32 = 0.00001;

pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Ray {
            origin,
            direction: direction.normalize(),
        }
    }

    #[allow(clippy::many_single_char_names)]
    pub fn hits_triangle(&self, a: &Vec3, b: &Vec3, c: &Vec3) -> Hit {
        let ab = *b - *a;
        let ac = *c - *a;
        let u_vec = self.direction.cross(ac);
        let det = ab.dot(u_vec);
        if det < EPSILON {
            return Hit::new(f32::INFINITY, 0.0, 0.0);
        }
        let inv_det = 1.0 / det;
        let a_to_origin = self.origin - *a;
        let u = a_to_origin.dot(u_vec) * inv_det;
        if !(0.0..=1.0).contains(&u) {
            return Hit::new(f32::INFINITY, u, 0.0);
        }
        let v_vec = a_to_origin.cross(ab);
        let v = self.direction.dot(v_vec) * inv_det;
        if v < 0.0 || (u + v) > 1.0 {
            return Hit::new(f32::INFINITY, u, v);
        }
        let t = ac.dot(v_vec) * inv_det;
        if t > EPSILON {
            Hit::new(t, u, v)
        } else {
            Hit::new(f32::INFINITY, u, v)
        }
    }

    pub fn get_point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

pub struct Hit {
    pub t: f32,
    pub u: f32,
    pub v: f32,
}

impl Hit {
    pub fn new(t: f32, u: f32, v: f32) -> Self {
        Hit { t, u, v }
    }
}
