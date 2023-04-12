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

struct Triangle {
    v0: Vec3A,
    v1: Vec3A,
    v2: Vec3A,

    pub material: Material
}
impl Triangle {
    fn new(v0: Vec3A, v1: Vec3A, v2: Vec3A, material: Material) -> Triangle {
        todo!()
    }
}

impl Surface for Triangle {
    fn ray_intersect(&self, ray: &Ray) -> Option<f32> {
        todo!()
    }

    fn hit(&self, ray: &Ray, t: f32) -> Hit {
        todo!()
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