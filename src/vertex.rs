use lazy_static::lazy_static;
use vulkanalia::prelude::v1_0::*;
use nalgebra_glm::{vec3, Vec3, Vec2, vec2};
use std::mem::size_of;
use anyhow::Result;
use log::*;

use crate::{app::AppData, buffers::{create_buffer, fill_buffer, copy_buffer}};



lazy_static!{
    pub static ref VERTICES: Vec<Vertex> = vec![
        Vertex::new(vec2(-0.5, -0.5), vec3(1.0, 0.0, 0.0), vec2(1.0, 0.0)),
        Vertex::new(vec2(0.5, -0.5), vec3(0.0, 1.0, 0.0), vec2(0.0, 0.0)),
        Vertex::new(vec2(0.5, 0.5), vec3(0.0, 0.0, 1.0), vec2(0.0, 1.0)),
        Vertex::new(vec2(-0.5, 0.5), vec3(0.0, 0.0, 1.0), vec2(1.0, 1.0))
    ];
}

lazy_static!{
    pub static ref INDICIES: Vec<u32> = vec![0, 1, 2, 2, 3, 0];
}




#[repr(C)]
pub struct Vertex {
    pub pos: Vec2,
    pub color: Vec3,
    pub tex_coord: Vec2
}


impl Vertex {
    pub fn new(pos: Vec2, color: Vec3, tex_coord: Vec2) -> Vertex {
        return Vertex {pos, color, tex_coord};
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX).build()
    }

    pub fn attribute_description() -> [vk::VertexInputAttributeDescription; 3] {
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

        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .location(2)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset((size_of::<Vec2>() + size_of::<Vec3>()) as u32).build();

        [pos, color, tex_coord]
    }
}


pub unsafe fn create_vertex_buffer(instance: &Instance, device: &Device, data: &mut AppData) -> Result<()> {

    let size = (size_of::<Vertex>() * VERTICES.len()) as u64;


    let (staging_buffer, staging_buffer_memory) = create_buffer(
        size, 
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        device, 
        instance, 
        data)?;
    

    fill_buffer(
        &staging_buffer, 
        &staging_buffer_memory, 
        &size, 
        VERTICES.as_ptr(), 
        VERTICES.len(), 
        device)?;
    


    let (buffer, buffer_memory) = create_buffer(
        size, 
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER, 
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        device, 
        instance, 
        data)?;
    
    device.bind_buffer_memory(buffer, buffer_memory, 0)?;



    copy_buffer(device, data, staging_buffer, buffer, size)?;



    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);



    data.vertex_buffer = buffer;
    data.vertex_buffer_memory = buffer_memory;




    info!("Created vertex buffer!");

    return Ok(());

}


pub unsafe fn create_index_buffer(instance: &Instance, device: &Device, data: &mut AppData) -> Result<()> {

    let size = (size_of::<u32>() * INDICIES.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        size, 
        vk::BufferUsageFlags::TRANSFER_SRC, 
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        device, 
        instance, 
        data)?;
    
    fill_buffer(&staging_buffer, &staging_buffer_memory, 
        &size, 
        INDICIES.as_ptr(), 
        INDICIES.len(), 
        device)?;
    

    let (buffer, buffer_memory) = create_buffer(
        size, 
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER, 
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        device, 
        instance, 
        data)?;
    

    device.bind_buffer_memory(buffer, buffer_memory, 0)?;
    
    copy_buffer(
        device, 
        data, 
        staging_buffer, 
        buffer, 
        size)?;
    

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);


    data.index_buffer = buffer;
    data.index_buffer_memory = buffer_memory;

    return Ok(());
}