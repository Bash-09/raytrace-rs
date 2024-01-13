use glam::DVec3;

pub struct Material {
    pub colour: DVec3,
    pub diffusion: f64,
    pub refractive_index: f64,
    pub luminance: f64,
}
