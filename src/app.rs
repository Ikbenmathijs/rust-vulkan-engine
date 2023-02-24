use vulkanalia::{loader::{LibloadingLoader, LIBRARY}, vk::{DebugUtilsMessengerEXT, ExtDebugUtilsExtension, KhrSurfaceExtension, KhrSwapchainExtension}};
use winit::window::{Window};
use anyhow::{Result, anyhow};
use vulkanalia::prelude::v1_0::*;
use crate::{instance::create_instance, device::{pick_physical_device, create_logical_device, QueueFamilyIndices}, swapchain::{create_swapchain, create_swapchain_image_views}, pipeline::create_pipeline, buffers::{create_framebuffers, create_command_pool, create_command_buffers}, sync::create_semaphore};
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
    pub pipeline_layout: vk::PipelineLayout,
    pub render_pass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub image_availible_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue
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
        create_framebuffers(&mut data, &device)?;
        create_command_buffers(&device, &mut data)?;
        
        let indicies = QueueFamilyIndices::get(&instance, &data.physical_device, &data)?;

        data.command_pool = create_command_pool(&device, indicies.graphics)?;

        data.render_finished_semaphore = create_semaphore(&device)?;
        data.image_availible_semaphore = create_semaphore(&device)?;


        return Ok(Self {entry, instance, data, device});
    }

    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {

        let image_index = self.device.acquire_next_image_khr(
            self.data.swapchain, 
            u64::max_value(), 
            self.data.image_availible_semaphore, 
            vk::Fence::null())?.0 as usize;
        
        
        let wait_semaphores = &[self.data.image_availible_semaphore];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index]];
        let signal_semaphores = &[self.data.render_finished_semaphore];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        self.device.queue_submit(self.data.graphics_queue, &[submit_info], vk::Fence::null())?;




        return Ok(());
    }

    pub unsafe fn destroy(&mut self) {
        println!("Goodbye!");

        


        self.device.destroy_pipeline(self.data.pipeline, None);
        debug!("Destroyed pipeline");

        self.device.destroy_render_pass(self.data.render_pass, None);
        debug!("Destroyed render pass");

        self.device.destroy_pipeline_layout(self.data.pipeline_layout, None);
        debug!("Destroyed pipeline layout");

 
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


