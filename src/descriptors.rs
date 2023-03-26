use vulkanalia::prelude::v1_0::*;
use anyhow::Result;
use crate::app::AppData;
use std::mem::size_of;
use crate::ubo::MVP_UBO;





pub unsafe fn create_descriptor_set_layout(device: &Device, data: &mut AppData) -> Result<()> {

    let mvp_ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);
    

    let sampler = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);


    let bindings = &[mvp_ubo_binding, sampler];

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
        let mvp_ubo_buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(data.uniform_buffers[i])
            .offset(0)
            .range(size_of::<MVP_UBO>() as u64).build();
        

        let buffer_infos = [mvp_ubo_buffer_info];
        
        let mvp_ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(&buffer_infos);


        let texture_image_info = vk::DescriptorImageInfo::builder()
            .sampler(data.texture_image_sampler)
            .image_view(data.texture_image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        
        let image_infos = &[texture_image_info];

        let texture_image_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(image_infos);
        

        
        device.update_descriptor_sets(&[mvp_ubo_write, texture_image_write], &[] as &[vk::CopyDescriptorSet]);

    }


    return Ok(());
}

pub unsafe fn create_descriptor_pool(device: &Device, data: &mut AppData) -> Result<()> {

    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(data.swapchain_images.len() as u32);

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(data.swapchain_images.len() as u32);

    let pool_sizes = &[ubo_size, sampler_size];

    let pool_create_info = vk::DescriptorPoolCreateInfo::builder()
        .max_sets(data.swapchain_images.len() as u32)
        .pool_sizes(pool_sizes);

    data.descriptor_pool = device.create_descriptor_pool(&pool_create_info, None)?;


    return Ok(());
}