use vulkanalia::prelude::v1_0::*;
use log::*;
use anyhow::Result;

use crate::app::AppData;

pub unsafe fn create_render_pass(device: &Device, data: &mut AppData) -> Result<()> {

    let color_attachment = vk::AttachmentDescription::builder()
        .format(data.swapchain_image_format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);



    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_attachments = &[color_attachment_ref];


    let subpass = vk::SubpassDescription::builder()
        .color_attachments(color_attachments)
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

    

    return Ok(());
}