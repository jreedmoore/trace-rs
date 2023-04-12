use std::cmp::min;
use std::fs::File;
use std::sync::{atomic::AtomicUsize, Arc};
use std::thread;
use std::time::{Duration, Instant};

use std::io::Write;

use glam::Vec3A;
use rand::Rng;
use rayon::prelude::*;

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

struct Ray {
    origin: Vec3A,
    direction: Vec3A,
}

struct Sphere {
    origin: Vec3A,
    radius: f32,
    k_ambient: Vec3A,
    k_diffuse: Vec3A,
    k_specular: Vec3A,
    shininess: f32,
}
impl Sphere {
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
}

struct Light {
    origin: Vec3A,
    diffuse_color: Vec3A,
    specular_color: Vec3A,
}

struct Scene {
    spheres: Vec<Sphere>,
    lights: Vec<Light>,
    global_light: Vec3A,
}
impl Scene {
    pub fn hits_any(&self, ray: &Ray) -> bool {
        for sphere in &self.spheres {
            if let Some(_) = sphere.ray_intersect(&ray) {
                return true
            }
        }
        false
    }

    pub fn best_hit(&self, ray: &Ray) -> Option<(f32, &Sphere)> {
        let mut best_hit: Option<(f32, &Sphere)> = None;
        for sphere in &self.spheres {
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
        spheres: vec![
            Sphere {
                origin: Vec3A::new(-4.0, -0.5, 14.0),
                radius: 1.0,
                k_ambient: Vec3A::new(1.0, 0.0, 0.0),
                k_diffuse: Vec3A::new(0.7, 0.5, 0.5),
                k_specular: Vec3A::splat(0.1),
                shininess: 20.0,
            },
            Sphere {
                origin: Vec3A::new(3.0, 0.0, 10.0),
                radius: 1.0,
                k_ambient: Vec3A::new(0.0, 1.0, 0.0),
                k_diffuse: Vec3A::new(0.5, 0.7, 0.5),
                k_specular: Vec3A::splat(0.1),
                shininess: 20.0,
            },
            Sphere {
                origin: Vec3A::new(3.5, 0.5, 8.0),
                radius: 1.0,
                k_ambient: Vec3A::new(0.0, 0.0, 1.0),
                k_diffuse: Vec3A::new(0.5, 0.5, 0.7),
                k_specular: Vec3A::splat(0.1),
                shininess: 20.0,
            },
            Sphere {
                origin: Vec3A::new(0.0, -102.0, 12.0),
                radius: 100.0,
                k_ambient: Vec3A::new(0.0, 0.0, 1.0),
                k_diffuse: Vec3A::new(0.5, 0.5, 0.7),
                k_specular: Vec3A::splat(0.1),
                shininess: 20.0,
            },
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

                let best_hit = scene.best_hit(&ray);

                if let Some((t, sphere)) = best_hit {
                    *pixel += scene.global_light * sphere.k_ambient;
                    let p = ray.origin + t * ray.direction;
                    let n = (p - sphere.origin).normalize();
                    for light in &scene.lights {
                        let l_v = (light.origin - p).normalize();

                        let d = l_v.dot(n);

                        if d > 0.0 && !scene.hits_any(&Ray { origin: p, direction: l_v }) {
                            let r = (2.0*(n.dot(l_v))*n) - l_v;
                            let v = (camera - p).normalize();
                            *pixel += sphere.k_diffuse * d * light.diffuse_color;
                            *pixel += sphere.k_specular * v.dot(r).powf(sphere.shininess) * light.specular_color;
                        }
                    }
                }
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
