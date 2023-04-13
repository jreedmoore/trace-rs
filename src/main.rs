mod bvh;
mod surface;

use std::collections::VecDeque;
use std::fs::File;
use std::sync::{atomic::AtomicUsize, Arc};
use std::time::{Duration, Instant};
use std::{io, thread};

use std::io::{BufRead, Write};

use bvh::BVH;
use glam::Vec3A;
use rand::Rng;
use rayon::prelude::*;
use surface::{CanHit, Geometry};

use crate::surface::{Material, Sphere, Triangle};

struct Image {
    width: usize,
    height: usize,
    pixels: Vec<Vec3A>,
}
impl Image {
    fn new(width: usize, height: usize) -> Image {
        Image {
            width,
            height,
            pixels: vec![Vec3A::ZERO; width * height],
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.height * self.width * 3);
        let n = (256 - 1) as f32;
        for pixel in &self.pixels {
            bytes.push((pixel.x.clamp(0.0, 1.0) * n + 0.5) as u8);
            bytes.push((pixel.y.clamp(0.0, 1.0) * n + 0.5) as u8);
            bytes.push((pixel.z.clamp(0.0, 1.0) * n + 0.5) as u8);
        }
        bytes
    }
}

pub struct Ray {
    origin: Vec3A,
    direction: Vec3A,
}
impl Ray {
    fn new(origin: Vec3A, direction: Vec3A) -> Ray {
        Ray { origin, direction }
    }
}

struct Light {
    origin: Vec3A,
    diffuse_color: Vec3A,
    specular_color: Vec3A,
}

struct SceneBuilder<'m> {
    surfaces: Vec<CanHit<'m>>,
    lights: Vec<Light>,
    global_light: Vec3A,
    camera: Vec3A,
}
impl<'m> SceneBuilder<'m> {
    pub fn build<'a>(&'a mut self) -> Scene<'a, 'm> {
        Scene {
            bvh: bvh::BVH::new(&mut self.surfaces),
            lights: &self.lights,
            global_light: self.global_light,
            camera: self.camera,
        }
    }
    pub fn add_quad(&mut self, v0: Vec3A, v1: Vec3A, v2: Vec3A, v3: Vec3A, material: &'m Material) {
        self.surfaces
            .push(CanHit::Triangle(Triangle::new(v0, v1, v2, material)));
        self.surfaces
            .push(CanHit::Triangle(Triangle::new(v0, v2, v3, material)));
    }
}
struct Scene<'a, 'm> {
    bvh: BVH<'a, 'm>,
    lights: &'a [Light],
    global_light: Vec3A,
    camera: Vec3A,
}
impl<'a, 'm> Scene<'a, 'm>
where
    'a: 'm,
{
    pub fn hits_any(&'a self, ray: &Ray) -> bool {
        self.bvh.hits_any(ray)
    }

    pub fn best_hit(&'a self, ray: &Ray) -> Option<(f32, &dyn Geometry)> {
        self.bvh.ray_intersect(ray)
    }

    pub fn ray_color(&'a self, ray: &Ray, depth: usize) -> Vec3A {
        let mut color = Vec3A::ZERO;
        if depth <= 0 {
            return color;
        }
        if let Some((t, surface)) = self.best_hit(ray) {
            color += self.global_light * surface.material().k_ambient;
            let hit = surface.hit(ray, t);
            let p = hit.at;
            let n = hit.surface_normal;
            for light in self.lights.iter() {
                let l_v = (light.origin - p).normalize();
                let v = (self.camera - p).normalize();
                let view_reflection = (2.0 * (n.dot(v) * n)) - v;

                color += surface.material().k_reflective
                    * self.ray_color(&Ray::new(p, view_reflection), depth - 1);

                let d = l_v.dot(n);

                if d > 0.0 && !self.hits_any(&Ray::new(p, l_v)) {
                    let lr = (2.0 * (n.dot(l_v)) * n) - l_v;
                    color += surface.material().k_diffuse * d * light.diffuse_color;
                    color += surface.material().k_specular
                        * v.dot(lr).powf(surface.material().shininess)
                        * light.specular_color;
                }
            }
        }
        color
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let width = 960;
    let height = width * 9 / 16;
    let mut image = Image::new(width, height);

    let h = image.height;
    let w = image.width;
    let fh = image.height as f32;
    let fw = image.width as f32;
    let aspect_ratio = fh / fw;

    let top_left = Vec3A::new(-1.0, aspect_ratio, 0.0);
    let top_right = Vec3A::new(1.0, aspect_ratio, 0.0);
    let bottom_left = Vec3A::new(-1.0, -aspect_ratio, 0.0);
    let bottom_right = Vec3A::new(1.0, -aspect_ratio, 0.0);

    let camera = Vec3A::new(0.0, 0.0, -1.0);

    let red = Material {
        k_ambient: Vec3A::new(1.0, 0.0, 0.0),
        k_diffuse: Vec3A::new(0.7, 0.5, 0.5),
        k_reflective: Vec3A::splat(0.2),
        k_specular: Vec3A::splat(0.1),
        shininess: 20.0,
    };

    let green = Material {
        k_ambient: Vec3A::new(0.0, 1.0, 0.0),
        k_diffuse: Vec3A::new(0.5, 0.7, 0.5),
        k_reflective: Vec3A::splat(0.2),
        k_specular: Vec3A::splat(0.1),
        shininess: 20.0,
    };

    let blue = Material {
        k_ambient: Vec3A::new(0.0, 0.0, 1.0),
        k_diffuse: Vec3A::new(0.5, 0.5, 0.7),
        k_reflective: Vec3A::splat(0.2),
        k_specular: Vec3A::splat(0.1),
        shininess: 20.0,
    };
    let mut builder = SceneBuilder {
        surfaces: vec![
            CanHit::Sphere(Sphere {
                origin: Vec3A::new(-4.0, -0.5, 14.0),
                radius: 1.0,
                material: &red,
            }),
            CanHit::Sphere(Sphere {
                origin: Vec3A::new(3.0, 0.0, 10.0),
                radius: 1.0,
                material: &green,
            }),
            CanHit::Sphere(Sphere {
                origin: Vec3A::new(3.5, 0.5, 8.0),
                radius: 1.0,
                material: &blue,
            }),
        ],
        lights: vec![
            Light {
                origin: Vec3A::new(-1.0, 8.0, 11.0),
                diffuse_color: 0.5 * Vec3A::new(1.0, 0.2, 1.0),
                specular_color: Vec3A::splat(0.8),
            },
            Light {
                origin: Vec3A::new(9.0, 8.0, 5.0),
                diffuse_color: 0.5 * Vec3A::new(0.0, 1.0, 0.0),
                specular_color: Vec3A::splat(0.8),
            },
        ],
        global_light: Vec3A::new(0.5, 0.5, 0.5),
        camera,
    };

    let purple = Material {
        k_ambient: Vec3A::new(0.7, 0.0, 0.7),
        k_diffuse: Vec3A::new(0.5, 0.5, 0.7),
        k_reflective: Vec3A::splat(0.1),
        k_specular: Vec3A::splat(0.1),
        shininess: 20.0,
    };

    builder.add_quad(
        Vec3A::new(10.0, -1.5, 3.0),
        Vec3A::new(-10.0, -1.5, 3.0),
        Vec3A::new(-10.0, -1.5, 20.0),
        Vec3A::new(10.0, -1.5, 20.0),
        &purple,
    );

    let load = Instant::now();
    let teapot = File::open("teapot.obj")?;
    let mut vertices: Vec<Vec3A> = vec![];
    let material = Material {
        k_ambient: Vec3A::new(0.7, 0.3, 0.7),
        k_diffuse: Vec3A::new(0.5, 0.5, 0.7),
        k_reflective: Vec3A::splat(0.1),
        k_specular: Vec3A::splat(0.1),
        shininess: 20.0,
    };
    let offset = Vec3A::new(0.0, 0.0, 10.0);
    for line in io::BufReader::new(teapot).lines() {
        let line = line?;
        match line.chars().next() {
            Some('v') => {
                let nums = line
                    .split(' ')
                    .skip(1)
                    .map(|s| s.parse::<f32>())
                    .flatten()
                    .collect::<Vec<f32>>();
                vertices.push(Vec3A::from_slice(&nums) + offset);
            }
            Some('f') => {
                let nums = line
                    .split(' ')
                    .skip(1)
                    .map(|s| s.parse::<usize>())
                    .flatten()
                    .collect::<Vec<usize>>();
                /*
                builder.surfaces.push(CanHit::Triangle(Triangle::new(
                    vertices[nums[0] - 1],
                    vertices[nums[1] - 1],
                    vertices[nums[2] - 1],
                    &material,
                )));*/
            }
            _ => (),
        }
    }
    println!("Finished load in: {} ms", load.elapsed().as_millis());

    let scene = builder.build();
    let pixels_rendered = Arc::new(AtomicUsize::new(0));
    let rayon_counter = Arc::clone(&pixels_rendered);
    let reporter_counter = Arc::clone(&pixels_rendered);

    let render = Instant::now();
    let progress_report_interval = 500;
    let _handle = thread::spawn(move || {
        let mut estimator = VecDeque::new();
        let mut prev_progress = 0.0;
        loop {
            let rendered = reporter_counter.load(std::sync::atomic::Ordering::SeqCst);
            let progress = rendered as f32 / (fh * fw);
            if estimator.len() >= 10 {
                estimator.pop_front();
            }
            estimator.push_back(progress);
            let mut progress_avg = 0.0;
            for i in 1..estimator.len() {
                progress_avg += estimator[i] - estimator[i - 1];
            }
            progress_avg /= estimator.len() as f32;
            let estimated_remaining = (progress_report_interval as f32 / progress_avg) * (1.0 - progress);
            prev_progress = progress;
            println!("{:.2}% est remaining: {:.2} s", progress * 100.0, estimated_remaining / 1000.0);
            if rendered == h * w {
                break;
            }
            thread::sleep(Duration::from_millis(progress_report_interval));
        }
    });

    let samples = 10;
    let ray_depth = 10;
    image
        .pixels
        .par_iter_mut()
        .enumerate()
        .for_each(|(idx, pixel)| {
            let x = idx % w;
            let y = idx / w;

            let mut rng = rand::thread_rng();
            for _ in 0..samples {
                let xt = (x as f32 + rng.gen::<f32>()) / (fw - 1.0);
                let yt = (y as f32 + rng.gen::<f32>()) / (fh - 1.0);

                let t = top_left.lerp(top_right, xt);
                let b = bottom_left.lerp(bottom_right, xt);
                let p = t.lerp(b, yt);

                let ray = Ray::new(p, p - camera);

                *pixel += scene.ray_color(&ray, ray_depth);
            }
            *pixel /= samples as f32;
            rayon_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
    println!("Finished render in: {} ms", render.elapsed().as_millis());

    let output = Instant::now();
    let mut f = File::create("output.ppm")?;
    writeln!(f, "P6")?;
    writeln!(f, "{} {}", image.width, image.height)?;
    writeln!(f, "255")?;
    f.write(&image.to_bytes())?;
    println!("Finished output in: {} ms", output.elapsed().as_millis());
    Ok(())
}
