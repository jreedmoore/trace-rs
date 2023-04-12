use std::sync::{Arc, atomic::AtomicUsize};
use std::time::{Instant, Duration};
use std::thread; 
use std::fs::File;

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
            pixels: vec![Vec3A::ZERO; width*height],
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
    direction: Vec3A
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

    let camera = Vec3A::new(0.0,0.0,1.0);
    
    let pixels_rendered = Arc::new(AtomicUsize::new(0));
    let rayon_counter = Arc::clone(&pixels_rendered);
    let reporter_counter = Arc::clone(&pixels_rendered);
        
    let render = Instant::now();
    let samples = 100;
    let _handle = thread::spawn(move || {
        loop {
            let rendered = reporter_counter.load(std::sync::atomic::Ordering::SeqCst);
            let percent = rendered as f32 * 100.0 / (fh * fw);
            println!("{:.2}%", percent);
            if rendered == h * w {
                break;
            }
            thread::sleep(Duration::from_millis(500));
        }
    });
    image.pixels.par_iter_mut().enumerate().for_each(|(idx, pixel)| {
        let x = idx % w;
        let y = idx / w;

        let mut rng = rand::thread_rng();
        for _ in 0..samples {
            let xt = (x as f32 + rng.gen::<f32>()) / (fw - 1.0);
            let yt = (y as f32 + rng.gen::<f32>()) / (fh - 1.0);

            let t = top_left.lerp(top_right, xt);
            let b = bottom_left.lerp(bottom_right, xt);
            let p = t.lerp(b, yt);

            let ray = Ray { origin: p, direction: p - camera };

            *pixel = ray.direction;
            pixel.x += aspect_ratio;
            pixel.x /= 2.0;

            pixel.y += 1.0;
            pixel.y /= 2.0;

            pixel.z = 0.3;
        }
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