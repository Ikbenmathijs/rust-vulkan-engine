use vulkanalia::{prelude::v1_0::*, vk::{SwapchainKHR, SwapchainCreateInfoKHR}};
use anyhow::Result;

use crate::app::AppData;


pub unsafe fn create_swapchain(data: &AppData) -> Result<SwapchainKHR> {




    let info = SwapchainCreateInfoKHR::builder()
        .surface(data.surface)
        ; 

}



struct SwapchainSupports {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>
}


impl  SwapchainSupports {
    unsafe fn get(instance: &Instance, data: &AppData, physical_device: &vk::PhysicalDevice) -> Result<Self> {
        
    }
}