mod surface;

use std::fs::File;
use std::sync::{atomic::AtomicUsize, Arc};
use std::thread;
use std::time::{Duration, Instant};

use std::io::Write;

use glam::Vec3A;
use rand::Rng;
use rayon::prelude::*;
use surface::Surface;

use crate::surface::{Sphere, Material, Triangle};

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


struct Light {
    origin: Vec3A,
    diffuse_color: Vec3A,
    specular_color: Vec3A,
}

struct Scene {
    surfaces: Vec<Box<dyn Surface + Sync>>,
    lights: Vec<Light>,
    global_light: Vec3A,
    camera: Vec3A,
}
impl Scene {
    pub fn hits_any(&self, ray: &Ray) -> bool {
        for sphere in &self.surfaces {
            if let Some(_) = sphere.ray_intersect(&ray) {
                return true
            }
        }
        false
    }

    pub fn best_hit(&self, ray: &Ray) -> Option<(f32, &Box<dyn Surface + Sync>)> {
        let mut best_hit: Option<(f32, &Box<dyn Surface + Sync>)> = None;
        for sphere in &self.surfaces {
            if let Some(t) = sphere.ray_intersect(&ray) {
                if let Some((prior_t, _)) = best_hit {
                    if t < prior_t {
                        best_hit = Some((t, sphere));
                    }
                } else {
                    best_hit = Some((t, sphere));
                }
            }
        }
        best_hit
    }
    
    pub fn ray_color(&self, ray: &Ray, depth: usize) -> Vec3A {
        let mut color = Vec3A::ZERO;
        if depth <= 0 {
            return color;
        }
        if let Some((t, surface)) = self.best_hit(ray) {
            color += self.global_light * surface.material().k_ambient;
            let hit = surface.hit(ray, t);
            let p = hit.at;
            let n = hit.surface_normal;
            for light in &self.lights {
                let l_v = (light.origin - p).normalize();
                let v = (self.camera - p).normalize();
                let view_reflection = (2.0*(n.dot(v)*n)) - v;

                color += surface.material().k_reflective * self.ray_color(&Ray { origin: p, direction: view_reflection}, depth - 1);

                let d = l_v.dot(n);

                if d > 0.0 && !self.hits_any(&Ray { origin: p, direction: l_v }) {
                    let lr = (2.0*(n.dot(l_v))*n) - l_v;
                    color += surface.material().k_diffuse * d * light.diffuse_color;
                    color += surface.material().k_specular * v.dot(lr).powf(surface.material().shininess) * light.specular_color;
                }
            }
        }
        color
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut image = Image::new(960, 540);

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

    let scene = Scene {
        surfaces: vec![
            Box::new(Sphere {
                origin: Vec3A::new(-4.0, -0.5, 14.0),
                radius: 1.0,
                material: Material {
                    k_ambient: Vec3A::new(1.0, 0.0, 0.0),
                    k_diffuse: Vec3A::new(0.7, 0.5, 0.5),
                    k_reflective: Vec3A::splat(0.2),
                    k_specular: Vec3A::splat(0.1),
                    shininess: 20.0,
                }
            }),
            Box::new(Sphere {
                origin: Vec3A::new(3.0, 0.0, 10.0),
                radius: 1.0,
                material: Material {
                    k_ambient: Vec3A::new(0.0, 1.0, 0.0),
                    k_diffuse: Vec3A::new(0.5, 0.7, 0.5),
                    k_reflective: Vec3A::splat(0.2),
                    k_specular: Vec3A::splat(0.1),
                    shininess: 20.0,
                }
            }),
            Box::new(Sphere {
                origin: Vec3A::new(3.5, 0.5, 8.0),
                radius: 1.0,
                material: Material {
                    k_ambient: Vec3A::new(0.0, 0.0, 1.0),
                    k_diffuse: Vec3A::new(0.5, 0.5, 0.7),
                    k_reflective: Vec3A::splat(0.2),
                    k_specular: Vec3A::splat(0.1),
                    shininess: 20.0,
                }
            }),
            Box::new(Triangle::new(
                Vec3A::new(10.0, -1.5, 3.0),
                Vec3A::new(-10.0, -1.5, 3.0),
                Vec3A::new(0.0, -1.0, 14.0),
                Material {
                    k_ambient: Vec3A::new(0.0, 0.0, 1.0),
                    k_diffuse: Vec3A::new(0.5, 0.5, 0.7),
                    k_reflective: Vec3A::splat(0.01),
                    k_specular: Vec3A::splat(0.2),
                    shininess: 20.0,
                }
            )),
        ],
        lights: vec![
            Light {
                origin: Vec3A::new(-1.0, 8.0, 11.0),
                diffuse_color: 0.5*Vec3A::new(1.0, 0.2, 1.0),
                specular_color: Vec3A::splat(0.8),
            },
            Light {
                origin: Vec3A::new(9.0, 8.0, 5.0),
                diffuse_color: 0.5*Vec3A::new(0.0, 1.0, 0.0),
                specular_color: Vec3A::splat(0.8),
            }
        ],
        global_light: Vec3A::new(0.5, 0.5, 0.5),
        camera
    };

    let pixels_rendered = Arc::new(AtomicUsize::new(0));
    let rayon_counter = Arc::clone(&pixels_rendered);
    let reporter_counter = Arc::clone(&pixels_rendered);

    let render = Instant::now();
    let samples = 100;
    let _handle = thread::spawn(move || loop {
        let rendered = reporter_counter.load(std::sync::atomic::Ordering::SeqCst);
        let percent = rendered as f32 * 100.0 / (fh * fw);
        println!("{:.2}%", percent);
        if rendered == h * w {
            break;
        }
        thread::sleep(Duration::from_millis(500));
    });

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

                let ray = Ray {
                    origin: p,
                    direction: p - camera,
                };

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
