use rand::rngs::SmallRng;
use std::time::Instant;

use glam::{DQuat, DVec3, EulerRot, UVec2};
use image::ImageOutputFormat;

use crate::{
    camera::PerspectiveCamera,
    collidable::{Plane, Sphere},
    material::Material,
    solver::Solver,
};

pub mod camera;
pub mod collidable;
pub mod material;
pub mod ray;
pub mod solver;

fn main() {
    let cam = PerspectiveCamera {
        origin: DVec3::new(0.0, 1.0, 0.0),
        rotation: DQuat::from_euler(EulerRot::YXZ, 0.0, 0.0, 0.0),
        horizontal_fov: 60.0,
    };

    let mut solver: Solver<'_, _, SmallRng> = Solver::new(cam, UVec2::new(1000, 1000))
        .with_samples(500)
        .with_max_bounces(10);

    let left_sphere = Sphere {
        origin: DVec3::new(-1.0, 0.7, 3.0),
        radius: 0.7,
        material: &Material {
            colour: DVec3::new(0.55, 0.55, 0.95),
            diffusion: 1.0,
            refractive_index: 0.0,
            luminance: 0.0,
        },
    };
    let middle_sphere = Sphere {
        origin: DVec3::new(0.0, 1.7, 3.0),
        radius: 0.7,
        material: &Material {
            colour: DVec3::new(0.95, 0.95, 0.95),
            diffusion: 0.0,
            refractive_index: 0.0,
            luminance: 0.0,
        },
    };
    let right_sphere = Sphere {
        origin: DVec3::new(1.0, 0.8, 3.0),
        radius: 0.7,
        material: &Material {
            colour: DVec3::new(0.95, 0.55, 0.55),
            diffusion: 0.5,
            refractive_index: 0.0,
            luminance: 0.0,
        },
    };

    let front_sphere = Sphere {
        origin: DVec3::new(0.0, 0.8, 2.5),
        radius: 0.5,
        material: &Material {
            colour: DVec3::ONE,
            diffusion: 0.0,
            refractive_index: 3.0,
            luminance: 0.0,
        },
    };

    let light_sphere = Sphere {
        origin: DVec3::new(-0.5, 0.3, 2.5),
        radius: 0.3,
        material: &Material {
            colour: DVec3::new(1.0, 1.0, 1.0),
            diffusion: 0.0,
            refractive_index: 0.0,
            luminance: 3.0,
        },
    };

    let plane = Plane {
        origin: DVec3::ZERO,
        normal: DVec3::Y,
        material: &Material {
            colour: DVec3::new(0.3, 0.75, 0.3),
            diffusion: 1.0,
            refractive_index: 0.0,
            luminance: 0.0,
        },
    };

    solver.objects.push(&left_sphere);
    solver.objects.push(&middle_sphere);
    solver.objects.push(&right_sphere);
    solver.objects.push(&front_sphere);
    solver.objects.push(&light_sphere);
    solver.objects.push(&plane);

    println!("Beginning render...");
    let start = Instant::now();
    let img = solver.solve(0);
    let fin = Instant::now();
    println!(
        "Render complete in {} secs.",
        fin.duration_since(start).as_secs_f32()
    );

    let dest = "img.png";
    println!("Writing to {}...", dest);
    let mut out_file = std::fs::File::create(dest).unwrap();
    img.write_to(&mut out_file, ImageOutputFormat::Png).unwrap();
    println!("File written to '{}'", dest);
}
