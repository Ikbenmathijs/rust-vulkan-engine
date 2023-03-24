use std::mem::size_of;

use vulkanalia::prelude::v1_0::*;
use nalgebra_glm as glm;
use anyhow::Result;


use crate::{app::AppData, buffers::create_buffer};



#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UBO {
    pub model: glm::Mat4,
    pub view: glm::Mat4,
    pub proj: glm::Mat4
}



pub unsafe fn create_descriptor_set_layout(device: &Device, data: &mut AppData) -> Result<()> {

    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);
    

    let bindings = &[ubo_binding];

    let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(bindings);

    data.descriptor_set_layout = device.create_descriptor_set_layout(&create_info, None)?;

    return Ok(());
}


pub unsafe fn create_descriptor_sets(device: &Device, data: &mut AppData) -> Result<()> {

    let descriptor_set_layouts = vec![data.descriptor_set_layout; data.swapchain_images.len()];


    let allocate_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.descriptor_pool)
        .set_layouts(&descriptor_set_layouts);


    data.descriptor_sets = device.allocate_descriptor_sets(&allocate_info)?;


    for i in 0..data.swapchain_images.len() {
        // Descriptors that refer to buffers, like our uniform buffer descriptor, are configured with a vk::DescriptorBufferInfo struct.
        // This structure specifies the buffer and the region within it that contains the data for the descriptor.
        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(data.uniform_buffers[i])
            .offset(0)
            .range(size_of::<UBO>() as u64).build();
        

        let buffer_infos = [buffer_info];
        
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(&buffer_infos);
        
        device.update_descriptor_sets(&[ubo_write], &[] as &[vk::CopyDescriptorSet]);

    }


    return Ok(());
}

pub unsafe fn create_descriptor_pool(device: &Device, data: &mut AppData) -> Result<()> {

    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(data.swapchain_images.len() as u32);

    let pool_sizes = &[ubo_size];

    let pool_create_info = vk::DescriptorPoolCreateInfo::builder()
        .max_sets(data.swapchain_images.len() as u32)
        .pool_sizes(pool_sizes);

    data.descriptor_pool = device.create_descriptor_pool(&pool_create_info, None)?;


    return Ok(());
}





pub unsafe fn create_uniform_buffers(instance: &Instance, device: &Device, data: &mut AppData) -> Result<()> {

    data.uniform_buffers.clear();
    data.uniform_buffer_memory.clear();



    for _ in 0..data.swapchain_images.len() {
        let (buffer, buffer_memory) = create_buffer(
            size_of::<UBO>() as u64, 
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