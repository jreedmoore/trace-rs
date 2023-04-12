use glam::Vec3A;

use crate::Ray;

pub trait Surface {
    fn material(&self) -> &Material;

    fn ray_intersect(&self, ray: &Ray) -> Option<f32>;
    fn hit(&self, ray: &Ray, t: f32) -> Hit;
}
pub struct Sphere {
    pub origin: Vec3A,
    pub radius: f32,

    pub material: Material,
}
impl Surface for Sphere {
    fn ray_intersect(&self, ray: &Ray) -> Option<f32> {
        let a = ray.direction.length_squared();
        let oc = ray.origin - self.origin;
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            None
        } else {
            let root_one = (-half_b - discriminant.sqrt()) / a;
            let root_two = (-half_b + discriminant.sqrt()) / a;
            if root_one > 1.0 {
                Some(root_one)
            } else if root_two > 1.0 {
                Some(root_two)
            } else {
                None
            }
        }
    }
    fn hit(&self, ray: &Ray, t: f32) -> Hit {
        let p = ray.origin + t*ray.direction;
        Hit { 
            at: p,
            surface_normal: (p - self.origin).normalize()
        }
    }

    fn material(&self) -> &Material {
        &self.material
    } 
}

pub struct Triangle {
    v0: Vec3A,
    v1: Vec3A,
    v2: Vec3A,

    normal: Vec3A,

    pub material: Material
}
impl Triangle {
    pub fn new(v0: Vec3A, v1: Vec3A, v2: Vec3A, material: Material) -> Triangle {
        let a = v1-v0;
        let b = v2-v0;
        let normal = a.cross(b).normalize();

        Triangle {
            v0,
            v1,
            v2,
            normal,
            material
        }
    }
}

pub const EPSILON: f32 = 1e-8;
impl Surface for Triangle {
    fn ray_intersect(&self, ray: &Ray) -> Option<f32> {
        // if ray and plane of triangle are parallel, no intersection
        // Moller-Tromboe
        let v0v1 = self.v1 - self.v0;
        let v0v2 = self.v2 - self.v0;
        let det = -ray.direction.dot(v0v1.cross(v0v2));
        //println!("det: {}", det);

        if det < EPSILON {
            return None
        }

        let inv_det = 1.0 / det;

        let b = ray.origin - self.v0;
        let det_u = -ray.direction.dot(b.cross(v0v2));
        let u = inv_det * det_u;
        //println!("u: {}", u);
        if u < 0.0 || u > 1.0 {
            return None
        }

        let det_v = -ray.direction.dot(v0v1.cross(b));
        let v = inv_det * det_v;
        //println!("v: {}", v);
        if v < 0.0 || v > 1.0 {
            return None
        }

        let det_t = b.dot(v0v1.cross(v0v2));
        //println!("det_t = {} dot ({} cross {})", b, v0v1, v0v2);
        let t = inv_det * det_t;
        //println!("t: {}, det_t: {}", t, det_t);
        if t < EPSILON {
            None
        } else {
            Some(t)
        }
    }

    fn hit(&self, ray: &Ray, t: f32) -> Hit {
        Hit {
            at: ray.origin + t*ray.direction,
            surface_normal: self.normal,
        }
    }

    // material props
    fn material(&self) -> &Material {
        &self.material
    }
}

pub struct Hit {
    pub at: Vec3A,
    pub surface_normal: Vec3A
}

pub struct Material {
    pub k_ambient: Vec3A,
    pub k_diffuse: Vec3A,
    pub k_specular: Vec3A,
    pub k_reflective: Vec3A,
    pub shininess: f32,
}
impl Material {
    fn default() -> Material {
        Material {
            k_ambient: Vec3A::splat(0.7),
            k_diffuse: Vec3A::splat(0.7),
            k_specular: Vec3A::splat(0.7),
            k_reflective: Vec3A::splat(0.7),
            shininess: 20.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3A;

    use crate::Ray;

    use super::{Triangle, Material, Surface};

    fn assert_approx_ex(a: f32, b: f32, msg: &'static str) {
        println!("{} ~= {}? a - b: {}", a, b, (a-b).abs());
        assert!((a - b).abs() < 1e-8, "{}: {} !~= {}", msg, a, b);
    }

    #[test]
    fn test_triangle_intersection() {
        let triangle = Triangle::new(
            Vec3A::new(1.0, -1.0, 1.0),
            Vec3A::new(-1.0, -1.0, 1.0),
            Vec3A::new(0.0, 1.0, 1.0),
            Material::default()
        );

        let ray = Ray { origin: Vec3A::ZERO, direction: Vec3A::new(0.0, 1.0, 1.0) };

        let t = triangle.ray_intersect(&ray).unwrap();

        assert_approx_ex(t, 1.0, "t");
    }
}