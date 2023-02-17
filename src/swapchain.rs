use vulkanalia::{prelude::v1_0::*, vk::{SwapchainKHR, SwapchainCreateInfoKHR, KhrSurfaceExtension}};
use anyhow::Result;

use crate::app::AppData;


pub unsafe fn create_swapchain(data: &AppData) -> Result<SwapchainKHR> {




    let info = SwapchainCreateInfoKHR::builder()
        .surface(data.surface)
        ; 

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