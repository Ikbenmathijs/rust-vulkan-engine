use vulkanalia::prelude::v1_0::*;
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

    return Ok(());
} 