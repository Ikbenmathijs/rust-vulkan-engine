use vulkanalia::{prelude::v1_0::*, vk::CommandPool};
use log::*;
use anyhow::Result;

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