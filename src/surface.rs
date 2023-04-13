use std::f32::{INFINITY, NEG_INFINITY};

use glam::Vec3A;

use crate::{bvh::BVH, Ray};

#[derive(Debug)]
pub enum CanHit<'m> {
    Sphere(Sphere<'m>),
    Triangle(Triangle<'m>),
}
impl<'m> CanHit<'m> {
    pub fn ray_intersect(&self, ray: &Ray) -> Option<(f32, &dyn Geometry)> {
        match self {
            CanHit::Sphere(s) => s.ray_intersect(ray),
            CanHit::Triangle(t) => t.ray_intersect(ray),
        }
    }
    pub fn hits_any(&self, ray: &Ray) -> bool {
        self.ray_intersect(ray).is_some()
    }
    pub fn aabb(&self) -> AABB {
        match self {
            CanHit::Sphere(s) => s.aabb(),
            CanHit::Triangle(t) => t.aabb(),
        }
    }
}
pub trait Geometry<'m> {
    fn material(&self) -> &'m Material;
    fn hit(&self, ray: &Ray, t: f32) -> Hit;
}
#[derive(Debug, Clone)]
pub struct AABB {
    min: Vec3A,
    max: Vec3A,
}
impl AABB {
    pub fn ray_hit(&self, ray: &Ray) -> bool {
        let ta = (self.min - ray.origin) / ray.direction;
        let tb = (self.max - ray.origin) / ray.direction;
        let min_t = ta.min(tb);
        let max_t = ta.max(tb);

        min_t.max_element() < max_t.min_element()
    }

    pub fn midpoint(&self) -> Vec3A {
        (self.min + self.max) * 0.5
    }

    pub fn zero() -> AABB {
        AABB {
            min: Vec3A::ZERO,
            max: Vec3A::ZERO,
        }
    }

    pub fn union_mut(&mut self, aabb: &AABB) {
        self.min = self.min.min(aabb.min);
        self.max = self.max.max(aabb.max);
    }

    pub fn max_axis(&self) -> usize {
        let diff = self.max - self.min;
        let mut max = -NEG_INFINITY;
        let mut max_i = 0;
        for i in 0..=2 {
            if diff[i] > max {
                max = diff[i];
                max_i = i;
            }
        }
        return max_i;
    }
}

#[derive(Debug, Clone)]
pub struct Sphere<'a> {
    pub origin: Vec3A,
    pub radius: f32,

    pub material: &'a Material,
}
impl<'a> Sphere<'a> {
    fn ray_intersect(&self, ray: &Ray) -> Option<(f32, &dyn Geometry)> {
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
                Some((root_one, self))
            } else if root_two > 1.0 {
                Some((root_two, self))
            } else {
                None
            }
        }
    }

    fn aabb(&self) -> AABB {
        AABB {
            min: self.origin - Vec3A::splat(self.radius),
            max: self.origin + Vec3A::splat(self.radius),
        }
    }
}
impl<'m> Geometry<'m> for Sphere<'m> {
    fn hit(&self, ray: &Ray, t: f32) -> Hit {
        let p = ray.origin + t * ray.direction;
        Hit {
            at: p,
            surface_normal: (p - self.origin).normalize(),
        }
    }

    fn material(&self) -> &'m Material {
        &self.material
    }
}

#[derive(Debug, Clone)]
pub struct Triangle<'m> {
    v0: Vec3A,
    v1: Vec3A,
    v2: Vec3A,

    normal: Vec3A,

    pub material: &'m Material,
}
impl<'m> Triangle<'m> {
    pub fn new(v0: Vec3A, v1: Vec3A, v2: Vec3A, material: &'m Material) -> Triangle<'m> {
        let a = v1 - v0;
        let b = v2 - v0;
        let normal = a.cross(b).normalize();

        Triangle {
            v0,
            v1,
            v2,
            normal,
            material,
        }
    }
}

pub const EPSILON: f32 = 1e-6;
impl<'m> Triangle<'m> {
    fn ray_intersect(&self, ray: &Ray) -> Option<(f32, &dyn Geometry<'m>)> {
        // if ray and plane of triangle are parallel, no intersection
        // Moller-Trumbore
        let v0v1 = self.v1 - self.v0;
        let v0v2 = self.v2 - self.v0;
        let plane_vec = v0v1.cross(v0v2);
        let det = -ray.direction.dot(plane_vec);

        if det < EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;

        let b = ray.origin - self.v0;
        let det_u = -ray.direction.dot(b.cross(v0v2));
        let u = inv_det * det_u;
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let det_v = -ray.direction.dot(v0v1.cross(b));
        let v = inv_det * det_v;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let det_t = b.dot(plane_vec);
        let t = inv_det * det_t;

        if t < EPSILON {
            None
        } else {
            Some((t, self))
        }
    }

    fn aabb(&self) -> AABB {
        AABB {
            min: self.v0.min(self.v1.min(self.v2)),
            max: self.v0.max(self.v1.max(self.v2)),
        }
    }
}
impl<'m> Geometry<'m> for Triangle<'m> {
    fn hit(&self, ray: &Ray, t: f32) -> Hit {
        Hit {
            at: ray.origin + t * ray.direction,
            surface_normal: self.normal,
        }
    }

    // material props
    fn material(&self) -> &'m Material {
        &self.material
    }
}

pub struct Hit {
    pub at: Vec3A,
    pub surface_normal: Vec3A,
}

#[derive(Debug, Clone)]
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
    use std::default;

    use glam::Vec3A;

    use crate::{surface::AABB, Ray};

    use super::{CanHit, Material, Triangle};

    fn assert_approx_ex(a: f32, b: f32, msg: &'static str) {
        println!("{} ~= {}? a - b: {}", a, b, (a - b).abs());
        assert!((a - b).abs() < 1e-8, "{}: {} !~= {}", msg, a, b);
    }

    #[test]
    fn test_triangle_intersection() {
        let mat = Material::default();
        let triangle = Triangle::new(
            Vec3A::new(6.0, -2.0, 16.0),
            Vec3A::new(-6.0, -2.0, 16.0),
            Vec3A::new(0.0, 5.0, 16.0),
            &mat,
        );

        assert!(triangle
            .ray_intersect(&Ray::new(
                Vec3A::ZERO,
                Vec3A::new(-1.0, 5.0, 16.0).normalize()
            ))
            .is_none());
        assert!(triangle
            .ray_intersect(&Ray::new(
                Vec3A::ZERO,
                Vec3A::new(0.0, 5.0, 16.0).normalize()
            ))
            .is_some());
    }

    #[test]
    fn test_aabb_intersection() {
        let aabb = AABB {
            min: Vec3A::splat(-1.0),
            max: Vec3A::splat(1.0),
        };

        assert!(aabb.ray_hit(&Ray::new(Vec3A::new(0.0, 0.0, -2.0), Vec3A::Z)));
        assert!(!aabb.ray_hit(&Ray::new(Vec3A::new(2.0, 0.0, -2.0), Vec3A::Z)));
    }
}
