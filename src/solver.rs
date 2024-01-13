use std::f64::consts::PI;

use glam::{DQuat, DVec3, IVec2, UVec2};
use image::RgbImage;
use indicatif::ProgressBar;
use rand::{Rng, SeedableRng};

use crate::{
    camera::Camera,
    collidable::{Collideable, Collision},
    ray::Ray,
};

pub struct Solver<'a, C: Camera, R: Rng + SeedableRng> {
    pub camera: C,
    pub resolution: UVec2,
    pub max_bounces: u64,
    pub samples: u64,

    pub objects: Vec<&'a dyn Collideable<R>>,
    pub sky: fn(DVec3) -> DVec3,
}

impl<'a, C: Camera, R: Rng + SeedableRng> Solver<'a, C, R> {
    pub fn new(camera: C, resolution: UVec2) -> Self {
        Self {
            camera,
            resolution,
            max_bounces: 0,
            samples: 1,

            objects: Vec::new(),
            sky: |d| DVec3::new(0.7, 0.7, 1.0) * (d.y + 0.2),
        }
    }

    pub fn with_max_bounces(mut self, max_bounces: u64) -> Self {
        self.max_bounces = max_bounces;
        self
    }

    pub fn with_samples(mut self, samples: u64) -> Self {
        self.samples = samples;
        self
    }

    pub fn solve(&self, seed: u64) -> RgbImage {
        let mut img = RgbImage::new(self.resolution.x, self.resolution.y);

        let bar = ProgressBar::new(self.resolution.x as u64 * self.resolution.y as u64);

        let mut rng = R::seed_from_u64(seed);

        for x in 0..self.resolution.x {
            for y in 0..self.resolution.y {
                let mut sample = DVec3::ZERO;
                for _ in 0..self.samples {
                    let ray = self.camera.outgoing_ray(
                        self.resolution.clone(),
                        IVec2::new(x as i32, y as i32),
                        &mut rng,
                    );

                    sample += self.sample(ray, 0, &mut rng);
                }

                let avg_scale = 1.0 / self.samples as f64;
                let pixel = img.get_pixel_mut(x, self.resolution.y - y - 1);
                pixel.0[0] = ((sample.x * avg_scale).clamp(0.0, 1.0) * 255.0) as u8;
                pixel.0[1] = ((sample.y * avg_scale).clamp(0.0, 1.0) * 255.0) as u8;
                pixel.0[2] = ((sample.z * avg_scale).clamp(0.0, 1.0) * 255.0) as u8;
            }
            bar.inc(self.resolution.x as u64);
        }

        bar.finish();

        img
    }

    fn sample(&self, ray: Ray, bounce: u64, rng: &mut R) -> DVec3 {
        // Trace ray
        let collision: Option<Collision<'_>> = self
            .objects
            .iter()
            .filter_map(|o| o.trace(&ray, rng))
            .fold(None, |min, c| {
                if min
                    .as_ref()
                    .map(|c: &Collision<'_>| c.t)
                    .unwrap_or(f64::INFINITY)
                    > c.t
                {
                    Some(c)
                } else {
                    min
                }
            });

        // No collision
        if collision.is_none() {
            return (self.sky)(ray.dir);
        }
        let c = collision.expect("Just checked it was some...");

        // Out of bounces
        if bounce >= self.max_bounces {
            return DVec3::ZERO;
        }

        // Calculate reflection/refraction ray
        let transmission_ray;

        // Snell's law for refraction ray
        let n1;
        let n2;
        let directed_normal;

        if c.normal.dot(c.ray.dir) < 0.0 {
            // Incoming
            n1 = 1.0;
            n2 = c.material.refractive_index;
            directed_normal = -c.normal;
        } else {
            // Outgoing
            n1 = c.material.refractive_index;
            n2 = 1.0;
            directed_normal = c.normal;
        }

        let incidence_angle = c.ray.dir.angle_between(directed_normal);
        let sin_a2 = n1 / n2 * incidence_angle.sin();
        if sin_a2 > 1.0 {
            // Internal reflection
            transmission_ray = None;
        } else {
            // Fresnel equations for calculating amount of tranmission vs reflectance
            let transmission_angle = sin_a2.asin();

            let cosi = incidence_angle.cos();
            let cost = transmission_angle.cos();
            let n1_cosi = n1 * cosi;
            let n1_cost = n1 * cost;
            let n2_cosi = n2 * cosi;
            let n2_cost = n2 * cost;

            let rs = ((n1_cosi - n2_cost) / (n1_cosi + n2_cost)).abs().powi(2);
            let rp = ((n1_cost - n2_cosi) / (n1_cost + n2_cosi)).abs().powi(2);

            let r = (rs + rp) / 2.0;

            if rng.gen_range(0.0..1.0) < r {
                transmission_ray = None;
            } else {
                transmission_ray = Some(transmission_angle);
            }
        }

        let new_ray = if let Some(transmission_angle) = transmission_ray {
            // Transmit
            let hit_pos = c.ray.at(c.t * 1.0001);

            let outgoing_dir =
                DQuat::from_axis_angle(c.ray.dir.cross(directed_normal), transmission_angle)
                    * directed_normal;

            Ray {
                origin: hit_pos,
                dir: outgoing_dir,
            }
        } else {
            // Reflect
            let hit_pos = c.ray.at(c.t * 0.9999);
            let random_unit_vector = {
                let theta = rng.gen_range(0.0..2.0 * PI);
                let theta2 = rng.gen_range(0.0..2.0 * PI);
                let x = theta.cos() * theta2.cos();
                let y = theta.cos() * theta2.sin();
                let z = theta.sin();
                DVec3::new(x, y, z)
            };

            let reflect_target = ray.dir + c.normal * 2.0;
            let mut diffuse_target = random_unit_vector;
            if (hit_pos + c.normal).dot(c.ray.origin) > 0.0 {
                diffuse_target += c.normal;
            } else {
                diffuse_target -= c.normal;
            };

            let actual_target = reflect_target.lerp(diffuse_target, c.material.diffusion);

            Ray {
                origin: hit_pos,
                dir: actual_target,
            }
        };

        // Propagate
        let sample = self.sample(new_ray, bounce + 1, rng);
        c.material.colour * sample + c.material.colour * c.material.luminance
    }
}
