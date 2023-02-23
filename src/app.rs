use vulkanalia::{loader::{LibloadingLoader, LIBRARY}, vk::{DebugUtilsMessengerEXT, ExtDebugUtilsExtension, KhrSurfaceExtension, KhrSwapchainExtension}};
use winit::window::{Window};
use anyhow::{Result, anyhow};
use vulkanalia::prelude::v1_0::*;
use crate::{instance::create_instance, device::{pick_physical_device, create_logical_device}, swapchain::{create_swapchain, create_swapchain_image_views}, pipeline::create_pipeline, render_pass::create_render_pass};
use log::*;
use vulkanalia::window as vkWindow;


#[derive(Clone, Debug, Default)]
pub struct AppData {
    pub messenger: DebugUtilsMessengerEXT,
    pub physical_device: vk::PhysicalDevice,
    pub surface: vk::SurfaceKHR,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_format: vk::Format,
    pub swapchain_image_views: Vec<vk::ImageView>,
    pub swapchain_extent: vk::Extent2D,
    pub pipeline_layout: vk::PipelineLayout
}


#[derive(Clone, Debug)]
pub struct App {
    entry: Entry,
    instance: Instance,
    data: AppData,
    device: Device
}


impl App {
    pub unsafe fn Create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|e| anyhow!(e))?;
        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;
        data.surface = vkWindow::create_surface(&instance, window)?;
        data.physical_device = pick_physical_device(&instance, &data)?;
        let device = create_logical_device(&instance, &data.physical_device, &data)?;
        create_swapchain(&instance, &mut data, &device, window)?;
        create_swapchain_image_views(&mut data, &device)?;

        create_pipeline(&mut data, &device)?;
        create_render_pass(&device, &mut data)?;

        return Ok(Self {entry, instance, data, device});
    }

    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {

        return Ok(());
    }

    pub unsafe fn destroy(&mut self) {
        println!("Goodbye!");

        for view in &self.data.swapchain_image_views {
            self.device.destroy_image_view(*view, None);
        }
        debug!("Destroyed image views");
        self.device.destroy_swapchain_khr(self.data.swapchain, None);
        debug!("Destroyed swapchain");


        self.device.destroy_device(None);
        debug!("Destroyed device");


        self.instance.destroy_surface_khr(self.data.surface, None);
        debug!("Destroyed surface");


        self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None);
        debug!("Destroyed debug messenger");

        self.instance.destroy_instance(None);
        debug!("destroyed instance");
    }
}


