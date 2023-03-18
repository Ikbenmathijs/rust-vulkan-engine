use vulkanalia::{prelude::v1_0::*, vk::CommandPool};
use log::*;
use anyhow::{Result, anyhow};
use std::ptr::copy_nonoverlapping as memcpy;

use crate::app::AppData;


pub unsafe fn create_framebuffers(data: &mut AppData, device: &Device) -> Result<()> {
    
    data.framebuffers = data.swapchain_image_views.iter().map(|i| {
        let attachments = &[*i];

        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(data.render_pass)
            .attachments(attachments)
            .width(data.swapchain_extent.width)
            .height(data.swapchain_extent.height)
            .layers(1);

        device.create_framebuffer(&framebuffer_info, None)
    }).collect::<Result<Vec<_>, _>>()?;

    debug!("Created {} framebuffers", data.framebuffers.len());

    return Ok(());
} 



pub unsafe fn create_command_pool(device: &Device, queue_family_index: u32) -> Result<CommandPool> {
    let command_pool_info = vk::CommandPoolCreateInfo::builder()
        .queue_family_index(queue_family_index);

    debug!("Creating command pool!");

    return Ok(device.create_command_pool(&command_pool_info, None)?);
}


pub unsafe fn create_command_buffers(device: &Device, data: &mut AppData) -> Result<()> {


    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(data.framebuffers.len() as u32);

    data.command_buffers = device.allocate_command_buffers(&allocate_info)?;

    for (i, command_buffer) in data.command_buffers.iter().enumerate() {
        
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();

        device.begin_command_buffer(*command_buffer, &command_buffer_begin_info)?;

        let render_area = vk::Rect2D {
            offset: vk::Offset2D {x: 0, y: 0}, 
            extent: vk::Extent2D {width: data.swapchain_extent.width, height: data.swapchain_extent.height}};
        
        let clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0]
            }
        };

        let clear_values = &[clear_value];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(data.render_pass)
            .framebuffer(data.framebuffers[i])
            .render_area(render_area)
            .clear_values(clear_values);

        device.cmd_begin_render_pass(*command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
        device.cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, data.pipeline);
        device.cmd_draw(*command_buffer, 3, 1, 0, 0);
        device.cmd_end_render_pass(*command_buffer);

        device.end_command_buffer(*command_buffer)?;
        debug!("Created command buffer with index: {}", i);

    }

    return Ok(());
}





pub unsafe fn create_buffer<T>(size: vk::DeviceSize, usage: vk::BufferUsageFlags, device: &Device, instance: &Instance, data: &AppData, cpy_src: *const T, cpy_count: usize) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = device.create_buffer(&buffer_info, None)?;

    let requirements = device.get_buffer_memory_requirements(buffer);

    let allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance, 
            data, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, 
            requirements)?);
    
    let device_memory = device.allocate_memory(&allocate_info, None)?;


    let memory_ptr = device.map_memory(device_memory, 0, buffer_info.size, vk::MemoryMapFlags::empty())?;

    memcpy(cpy_src, memory_ptr.cast(), cpy_count);

    device.unmap_memory(device_memory);

    Ok((buffer, device_memory))
}


unsafe fn get_memory_type_index(
    instance: &Instance,
    data: &AppData,
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements
) -> Result<u32> {
    let memory_props = instance.get_physical_device_memory_properties(data.physical_device);
    let mut index: Option<u32> = None;
    for (i, memtype) in memory_props.memory_types.iter().enumerate() {
        if memtype.property_flags.contains(properties) && (requirements.memory_type_bits & (1 << i) != 0) {
            index = Some(i as u32);
            debug!("\tFound memory type index: {}", i);
        }
    }

    if let Some(i) = index {
        return Ok(i);
    } else {
        return Err(anyhow!("Couldn't find valid memory type."));
    }
}