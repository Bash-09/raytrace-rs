use glam::{DQuat, DVec2, DVec3, IVec2, UVec2};
use rand::{Rng, SeedableRng};

use crate::ray::Ray;

pub trait Camera {
    fn outgoing_ray<R: Rng + SeedableRng>(&self, res: UVec2, pixel: IVec2, rng: &mut R) -> Ray;
}

pub struct OrthCamera {
    pub origin: DVec3,
    pub rotation: DQuat,
    pub size: DVec2,
}

impl Camera for OrthCamera {
    fn outgoing_ray<R: Rng>(&self, res: UVec2, pixel: IVec2, rng: &mut R) -> Ray {
        let scale_x = self.size.x / res.x as f64;
        let scale_y = self.size.y / res.y as f64;

        let off_x = rng.gen_range(-scale_x / 2.0..scale_x / 2.0);
        let off_y = rng.gen_range(-scale_y / 2.0..scale_y / 2.0);

        let mut out = Ray {
            origin: DVec3::new(
                pixel.x as f64 * scale_x + scale_x / 2.0 - self.size.x / 2.0 + off_x,
                pixel.y as f64 * scale_y + scale_y / 2.0 - self.size.y / 2.0 + off_y,
                0.0,
            ),
            dir: DVec3::Z,
        };

        out.origin += self.origin;
        out.dir = self.rotation * out.dir;

        out
    }
}

pub struct PerspectiveCamera {
    pub origin: DVec3,
    pub rotation: DQuat,
    pub horizontal_fov: f64,
}

impl Camera for PerspectiveCamera {
    fn outgoing_ray<R: Rng + SeedableRng>(&self, res: UVec2, pixel: IVec2, rng: &mut R) -> Ray {
        let scale_x = 1.0 / res.x as f64;
        let scale_y = 1.0 / res.y as f64;

        let off_x = rng.gen_range(-scale_x / 2.0..scale_x / 2.0);
        let off_y = rng.gen_range(-scale_y / 2.0..scale_y / 2.0);

        let target = DVec3::new(
            pixel.x as f64 * scale_x + scale_x / 2.0 - 0.5 + off_x,
            pixel.y as f64 * scale_y + scale_y / 2.0 - 0.5 + off_y,
            0.5 / (self.horizontal_fov.to_radians() / 2.0).tan(),
        )
        .normalize();

        Ray {
            origin: self.origin.clone(),
            dir: self.rotation * target,
        }
    }
}
