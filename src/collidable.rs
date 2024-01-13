use glam::DVec3;
use rand::{Rng, SeedableRng};

use crate::{material::Material, ray::Ray};

pub struct Collision<'a> {
    pub ray: Ray,
    pub t: f64,
    pub normal: DVec3,
    pub material: &'a Material,
}

pub trait Collideable<R: Rng + SeedableRng> {
    fn trace(&self, ray: &Ray, rng: &mut R) -> Option<Collision>;
}

pub struct Plane<'a> {
    pub origin: DVec3,
    pub normal: DVec3,
    pub material: &'a Material,
}

impl<'a, R: Rng + SeedableRng> Collideable<R> for Plane<'a> {
    fn trace(&self, ray: &Ray, _rng: &mut R) -> Option<Collision> {
        let numerator = -(ray.origin.x - self.origin.x) * self.normal.x
            - (ray.origin.y - self.origin.y) * self.normal.y
            - (ray.origin.z - self.origin.z) * self.normal.z;

        let denominator =
            ray.dir.x * self.normal.x + ray.dir.y * self.normal.y + ray.dir.z * self.normal.z;

        let t = numerator / denominator;
        if t < 0.0 {
            return None;
        }

        Some(Collision {
            ray: ray.clone(),
            t,
            normal: self.normal.normalize(),
            material: &self.material,
        })
    }
}

pub struct Sphere<'a> {
    pub origin: DVec3,
    pub radius: f64,
    pub material: &'a Material,
}

impl<'a, R: Rng + SeedableRng> Collideable<R> for Sphere<'a> {
    fn trace(&self, ray: &Ray, _rng: &mut R) -> Option<Collision> {
        let off = DVec3::new(
            ray.origin.x - self.origin.x,
            ray.origin.y - self.origin.y,
            ray.origin.z - self.origin.z,
        );

        let a = ray.dir.length_squared();
        let b = 2.0 * (off.x * ray.dir.x + off.y * ray.dir.y + off.z * ray.dir.z);
        let c = off.length_squared() - self.radius * self.radius;

        let disc = b * b - 4.0 * a * c;

        if disc < 0.0 {
            return None;
        }

        let sqrt_disc = disc.sqrt();
        let t0 = (-b + sqrt_disc) / (2.0 * a);
        let t1 = (-b - sqrt_disc) / (2.0 * a);

        let mut t = None;

        if t0 > 0.0 {
            t = Some(t0);
        }

        if t1 > 0.0 {
            if let Some(t) = &mut t {
                *t = t.min(t1);
            }
        }

        if let Some(t) = t {
            Some(Collision {
                ray: ray.clone(),
                t,
                normal: (ray.at(t) - self.origin).normalize(),
                material: &self.material,
            })
        } else {
            None
        }
    }
}
