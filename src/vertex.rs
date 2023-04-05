use lazy_static::lazy_static;
use vulkanalia::prelude::v1_0::*;
use nalgebra_glm as glm;
use nalgebra_glm::{Vec3, Vec2};
use std::mem::size_of;
use anyhow::Result;
use log::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::fs::File;

use crate::{app::AppData, buffers::{create_buffer, fill_buffer, copy_buffer}};



#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: Vec3,
    pub color: Vec3,
    pub tex_coord: Vec2
}


impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
            && self.color == other.color
            && self.tex_coord == other.tex_coord
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos[0].to_bits().hash(state);
        self.pos[1].to_bits().hash(state);
        self.pos[2].to_bits().hash(state);
        self.color[0].to_bits().hash(state);
        self.color[1].to_bits().hash(state);
        self.color[2].to_bits().hash(state);
        self.tex_coord[0].to_bits().hash(state);
        self.tex_coord[1].to_bits().hash(state);
    }
}



pub unsafe fn load_model(data: &mut AppData) -> Result<()> {

    let mut reader = BufReader::new(File::open("resources/viking_room.obj")?);

    let (models, _) = tobj::load_obj_buf(
        &mut reader, 
        &tobj::LoadOptions { triangulate: true, ..Default::default() }, 
        |_| Ok(Default::default()))?;

    let mut unique_verticies = HashMap::new();

        
    
    for model in &models {
        for index in &model.mesh.indices {

            let pos_offset = (3 * index) as usize;
            let tex_coord_offset = (2 * index) as usize;

            let vertex = Vertex {
                pos: glm::vec3(
                    model.mesh.positions[pos_offset],
                    model.mesh.positions[pos_offset + 1],
                    model.mesh.positions[pos_offset + 2]
                ),
                color: glm::vec3(1.0, 1.0, 1.0),
                tex_coord: glm::vec2(
                    model.mesh.texcoords[tex_coord_offset],
                    1.0 - model.mesh.texcoords[tex_coord_offset + 1]
                )
            };


            if let Some(index) = unique_verticies.get(&vertex) {
                data.indicies.push(*index);
            } else {
                let index = data.vertices.len();
                unique_verticies.insert(vertex, index as u32);
                data.vertices.push(vertex);
                data.indicies.push(index as u32);
            }
        }
    }

    return Ok(());
}


impl Vertex {
    pub fn new(pos: Vec3, color: Vec3, tex_coord: Vec2) -> Vertex {
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
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0).build();

        let color = vk::VertexInputAttributeDescription::builder()
            .location(1)
            .binding(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<Vec3>() as u32).build();

        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .location(2)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset((size_of::<Vec3>() + size_of::<Vec3>()) as u32).build();

        [pos, color, tex_coord]
    }
}


pub unsafe fn create_vertex_buffer(instance: &Instance, device: &Device, data: &mut AppData) -> Result<()> {

    let size = (size_of::<Vertex>() * data.vertices.len()) as u64;


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
        data.vertices.as_ptr(), 
        data.vertices.len(), 
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

    let size = (size_of::<u32>() * data.indicies.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        size, 
        vk::BufferUsageFlags::TRANSFER_SRC, 
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        device, 
        instance, 
        data)?;
    
    fill_buffer(&staging_buffer, &staging_buffer_memory, 
        &size, 
        data.indicies.as_ptr(), 
        data.indicies.len(), 
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