use lazy_static::lazy_static;
use vulkanalia::prelude::v1_0::*;
use nalgebra_glm::{vec3, Vec3, Vec2, vec2};



lazy_static!{
    static ref VERTICES: Vec<Vertex> = vec![
        Vertex::new(vec2(0.0, -0.5), vec3(1.0, 0.0, 0.0)),
        Vertex::new(vec2(0.5, 0.5), vec3(0.0, 1.0, 0.0)),
        Vertex::new(vec2(-0.5, 0.0), vec3(0.0, 0.0, 1.0))
    ];
}



#[repr(C)]
struct Vertex {
    pos: Vec2,
    color: Vec3
}


impl Vertex {
    pub fn new(pos: Vec2, color: Vec3) -> Vertex {
        return Vertex {pos, color};
    }
}