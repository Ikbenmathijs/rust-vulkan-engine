use vulkanalia::{prelude::v1_0::{*, vk::KhrSurfaceExtension}, vk::{KhrSwapchainExtension}};
use anyhow::Result;
use log::*;
use winit::window::Window;

use crate::{app::AppData, images::create_image_view};
use crate::device::QueueFamilyIndices;


pub unsafe fn create_swapchain(instance: &Instance, data: &mut AppData, device: &Device, window: &Window) -> Result<()> {

    let indices = QueueFamilyIndices::get(instance, &data.physical_device, data)?;
    
    let support = SwapchainSupport::get(instance, data, &data.physical_device)?;
    let image_count = support.capabilities.min_image_count + 1;

    let format = get_swapchain_format(&support.formats);

    let mut queue_family_indicies = vec![];
    let image_sharing_mode = if indices.graphics != indices.present {
        queue_family_indicies.push(indices.graphics);
        queue_family_indicies.push(indices.present);
        warn!("Graphics and present queue family indices are not the same, which *should* be supported but no promises!");
        vk::SharingMode::CONCURRENT
    } else {
        queue_family_indicies.push(indices.graphics);
        debug!("Graphics and present queue family are the same!");
        vk::SharingMode::EXCLUSIVE
    };


    let extent = get_swapchain_extent(window, support.capabilities);

    data.swapchain_extent = extent;

    let present_modes = instance.get_physical_device_surface_present_modes_khr(data.physical_device, data.surface)?;

    
    let present_mode = *present_modes.iter()
    .find(|p| **p == vk::PresentModeKHR::MAILBOX)
    .unwrap_or_else(|| {
        debug!("Mailbox isn't supported, so using FIFO");
        &vk::PresentModeKHR::FIFO});


    let info = vk::SwapchainCreateInfoKHR::builder()
        .surface(data.surface)
        .min_image_count(image_count)
        .image_format(format.format)
        .image_color_space(format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(image_sharing_mode)
        .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true);

    data.swapchain = device.create_swapchain_khr(&info, None)?;

    data.swapchain_images = device.get_swapchain_images_khr(data.swapchain)?;

    data.swapchain_image_format = format.format;

    info!("Created swapchain!");

    return Ok(());
}




fn get_swapchain_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    return formats.iter()
    .cloned()
    .find(|f| f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
    .unwrap_or_else(|| formats[0]);
}


fn get_swapchain_extent(window: &Window, capabilities: vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::max_value() {
        capabilities.current_extent
    } else {
        let size = window.inner_size();
        let clamp = |min: u32, max: u32, v: u32| min.max(max.min(v));
        vk::Extent2D::builder()
            .width(clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
                size.width,
            ))
            .height(clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
                size.height,
            ))
            .build()
    }
}


struct SwapchainSupport {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>
}


impl  SwapchainSupport {
    unsafe fn get(instance: &Instance, data: &AppData, physical_device: &vk::PhysicalDevice) -> Result<Self> {
        let formats = instance.get_physical_device_surface_formats_khr(*physical_device, data.surface)?;
        let capabilities = instance.get_physical_device_surface_capabilities_khr(*physical_device, data.surface)?;
        let present_modes = instance.get_physical_device_surface_present_modes_khr(*physical_device, data.surface)?;
        return Ok(Self {capabilities, formats, present_modes});
    }
}


pub unsafe fn create_swapchain_image_views(data: &mut AppData, device: &Device) -> Result<()> {

    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1).build();

    let mut image_views: Vec<vk::ImageView> = vec![];


    for image in &data.swapchain_images {
        image_views.push(create_image_view(image, device, data.swapchain_image_format, subresource)?);
        debug!("Created swapchain image view");
    }


    data.swapchain_image_views = image_views;
    info!("Created swapchain image views");

    return Ok(());
}