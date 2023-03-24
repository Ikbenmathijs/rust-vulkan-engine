use vulkanalia::{prelude::v1_0::*, vk::CommandPool};
use log::*;
use anyhow::{Result, anyhow};
use std::ptr::copy_nonoverlapping as memcpy;

use crate::app::AppData;
use crate::device::QueueFamilyIndices;
use crate::vertex::{VERTICES, INDICIES};


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



pub unsafe fn create_command_pools(device: &Device, instance: &Instance, data: &mut AppData) -> Result<()> {
    let indicies = QueueFamilyIndices::get(instance, data, Some(&data.physical_device))?;

    let command_pool_info = vk::CommandPoolCreateInfo::builder()
        .queue_family_index(indicies.graphics);

    let transient_command_pool_info = vk::CommandPoolCreateInfo::builder()
        .queue_family_index(indicies.graphics);

    data.command_pool = device.create_command_pool(&command_pool_info, None)?;
    data.transient_command_pool = device.create_command_pool(&transient_command_pool_info, None)?;

    debug!("Created command pools!");

    return Ok(());
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

        device.cmd_bind_vertex_buffers(*command_buffer, 0, &[data.vertex_buffer], &[0]);
        device.cmd_bind_index_buffer(*command_buffer, data.index_buffer, 0, vk::IndexType::UINT32);
        

        device.cmd_bind_descriptor_sets(*command_buffer, vk::PipelineBindPoint::GRAPHICS, data.pipeline_layout, 0, &[data.descriptor_sets[i]], &[]);

        device.cmd_draw_indexed(*command_buffer, INDICIES.len() as u32, 1, 0, 0, 0);
        device.cmd_end_render_pass(*command_buffer);

        device.end_command_buffer(*command_buffer)?;
        debug!("Created command buffer with index: {}", i);

    }

    return Ok(());
}





pub unsafe fn create_buffer(size: vk::DeviceSize, usage: vk::BufferUsageFlags, mem_props: vk::MemoryPropertyFlags, device: &Device, instance: &Instance, data: &AppData) -> Result<(vk::Buffer, vk::DeviceMemory)> {
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
            mem_props, 
            requirements)?);
    
    let device_memory = device.allocate_memory(&allocate_info, None)?;


    

    debug!("Created buffer");

    Ok((buffer, device_memory))
}




pub unsafe fn fill_buffer<T>(
    buffer: &vk::Buffer, 
    device_memory: &vk::DeviceMemory,
    size: &vk::DeviceSize,
    cpy_src: *const T,
    cpy_count: usize,
    device: &Device
) -> Result<()> {

    let memory_ptr = device.map_memory(*device_memory, 0, *size, vk::MemoryMapFlags::empty())?;

    memcpy(cpy_src, memory_ptr.cast(), cpy_count);

    device.unmap_memory(*device_memory);

    device.bind_buffer_memory(*buffer, *device_memory, 0)?;

    debug!("Filled buffer!");
    
    return Ok(());
}


pub unsafe fn copy_buffer(
    device: &Device,
    data: &AppData,
    src: vk::Buffer,
    dst: vk::Buffer,
    size: u64
) -> Result<()> {
    debug!("Copying buffer");
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.transient_command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);


    let command_buffer = device.allocate_command_buffers(&allocate_info)?[0];

    let begin_info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    device.begin_command_buffer(command_buffer, &begin_info)?;

    let region = vk::BufferCopy::builder().size(size);

    device.cmd_copy_buffer(command_buffer, src, dst, &[region]);

    device.end_command_buffer(command_buffer)?;

    let command_buffers = &[command_buffer];

    let submit_info = vk::SubmitInfo::builder()
        .command_buffers(command_buffers);

    device.queue_submit(data.graphics_queue, &[submit_info], vk::Fence::null())?;
    device.queue_wait_idle(data.graphics_queue)?;
    
    device.free_command_buffers(data.transient_command_pool, command_buffers);


    Ok(())
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
            break;
        }
    }

    if let Some(i) = index {
        return Ok(i);
    } else {
        return Err(anyhow!("Couldn't find valid memory type."));
    }
}