use std::mem::size_of;

use vulkanalia::prelude::v1_0::*;
use nalgebra_glm as glm;
use anyhow::Result;


use crate::{app::AppData, buffers::create_buffer};



#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MVP_UBO {
    pub view: glm::Mat4,
    pub proj: glm::Mat4
}




pub unsafe fn create_uniform_buffers(instance: &Instance, device: &Device, data: &mut AppData) -> Result<()> {

    data.uniform_buffers.clear();
    data.uniform_buffer_memory.clear();



    for _ in 0..data.swapchain_images.len() {
        let (buffer, buffer_memory) = create_buffer(
            size_of::<MVP_UBO>() as u64, 
            vk::BufferUsageFlags::UNIFORM_BUFFER, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, 
            device, 
            instance, 
            data)?;
        
            data.uniform_buffers.push(buffer);
            data.uniform_buffer_memory.push(buffer_memory);
            device.bind_buffer_memory(buffer, buffer_memory, 0)?;
        
    }


    return Ok(());
}