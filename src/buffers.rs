use vulkanalia::prelude::v1_0::*;
use log::*;
use anyhow::{Result, anyhow};
use std::ptr::copy_nonoverlapping as memcpy;
use nalgebra_glm as glm;


use crate::app::AppData;
use crate::device::QueueFamilyIndices;


pub unsafe fn create_framebuffers(data: &mut AppData, device: &Device) -> Result<()> {
    
    data.framebuffers = data.swapchain_image_views.iter().map(|i| {
        let attachments = &[data.color_image_view, data.depth_image_view, *i];



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



pub unsafe fn create_command_pools(device: &Device, data: &mut AppData) -> Result<()> {

    let transient_command_pool_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(data.queue_family_indicies.graphics);

        data.transient_command_pool = device.create_command_pool(&transient_command_pool_info, None)?;



    for _ in 0..data.swapchain_images.len() {
        data.command_pools.push(create_command_pool(device, data)?);
    }

    debug!("Created command pools!");

    return Ok(());
}


unsafe fn create_command_pool(device: &Device, data: &AppData) -> Result<vk::CommandPool> {

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(data.queue_family_indicies.graphics);

    return Ok(device.create_command_pool(&info, None)?);
}


pub unsafe fn create_command_buffers(device: &Device, data: &mut AppData) -> Result<()> {


    for i in 0..data.swapchain_images.len() {
        println!("{:?}", data.command_pools);

        let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.command_pools[i])
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(data.framebuffers.len() as u32);

        data.command_buffers.push(device.allocate_command_buffers(&allocate_info)?[0]);
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
    let command_buffer = begin_single_time_commands(device, data)?;

    let region = vk::BufferCopy::builder().size(size);

    device.cmd_copy_buffer(command_buffer, src, dst, &[region]);

    end_single_time_commands(device, data, command_buffer)?;


    Ok(())
}



pub unsafe fn begin_single_time_commands(device: &Device, data: &AppData) -> Result<vk::CommandBuffer> {

    let command_buffer_info = vk::CommandBufferAllocateInfo::builder()
    .command_pool(data.transient_command_pool)
    .level(vk::CommandBufferLevel::PRIMARY)
    .command_buffer_count(1);

    let command_buffer = device.allocate_command_buffers(&command_buffer_info)?[0];

    let begin_info = vk::CommandBufferBeginInfo::builder()
    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    device.begin_command_buffer(command_buffer, &begin_info)?;

    Ok(command_buffer)
}



pub unsafe fn end_single_time_commands(device: &Device, data: &AppData, command_buffer: vk::CommandBuffer) -> Result<()> {

    device.end_command_buffer(command_buffer)?;

    let command_buffers = &[command_buffer];

    let submit_info = vk::SubmitInfo::builder()
        .command_buffers(command_buffers);

    device.queue_submit(data.graphics_queue, &[submit_info], vk::Fence::null())?;

    device.queue_wait_idle(data.graphics_queue)?;

    device.free_command_buffers(data.transient_command_pool, command_buffers);

    Ok(())
}


pub unsafe fn get_memory_type_index(
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