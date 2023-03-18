use lazy_static::lazy_static;
use vulkanalia::prelude::v1_0::*;
use nalgebra_glm::{vec3, Vec3, Vec2, vec2};
use std::mem::size_of;
use anyhow::Result;

use crate::{app::AppData, buffers::create_buffer};



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

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX).build()
    }

    pub fn attribute_description() -> [vk::VertexInputAttributeDescription; 2] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(0).build();

        let color = vk::VertexInputAttributeDescription::builder()
            .location(1)
            .binding(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<Vec2>() as u32).build();

        [pos, color]
    }
}


pub unsafe fn create_vertex_buffer(instance: &Instance, device: &Device, data: &mut AppData) -> Result<()> {

    let (buffer, buffer_memory) = create_buffer(
        (size_of::<Vertex>() * VERTICES.len()) as u64, 
        vk::BufferUsageFlags::VERTEX_BUFFER, 
        device, 
        instance, 
        data, 
        VERTICES.as_ptr(), 
        VERTICES.len())?;

    data.vertex_buffer = buffer;
    data.vertex_buffer_memory = buffer_memory;

    return Ok(());

}