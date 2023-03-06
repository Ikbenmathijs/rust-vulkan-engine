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
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue
}


#[derive(Clone, Debug)]
pub struct App {
    entry: Entry,
    instance: Instance,
    data: AppData,
    device: Device,
    frame: usize
}


impl App {
    pub unsafe fn Create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|e| anyhow!(e))?;
        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;
        data.surface = vkWindow::create_surface(&instance, window)?;
        data.physical_device = pick_physical_device(&instance, &data)?;
        let device = create_logical_device(&instance, &mut data)?;
        create_swapchain(&instance, &mut data, &device, window)?;
        create_swapchain_image_views(&mut data, &device)?;

        create_pipeline(&mut data, &device)?;
        create_framebuffers(&mut data, &device)?;
        let indicies = QueueFamilyIndices::get(&instance, &data.physical_device, &data)?;

        data.command_pool = create_command_pool(&device, indicies.graphics)?;

        create_command_buffers(&device, &mut data)?;
        

        for i in 0..data.swapchain_images.len() {
            data.render_finished_semaphores.push(create_semaphore(&device)?);
            data.image_available_semaphores.push(create_semaphore(&device)?);
        }
        


        return Ok(Self {entry, instance, data, device, frame: 0});
    }

    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {

       // let in_flight_fence = self.data.in_flight_fences[self.frame];

       /*self.device
            .wait_for_fences(&[in_flight_fence], true, u64::max_value())?;*/

        let image_index = self
            .device
            .acquire_next_image_khr(
                self.data.swapchain,
                u64::max_value(),
                self.data.image_available_semaphores[self.frame],
                vk::Fence::null(),
            )?
            .0 as usize;

        /*let image_in_flight = self.data.images_in_flight[image_index];
        if !image_in_flight.is_null() {
            self.device
                .wait_for_fences(&[image_in_flight], true, u64::max_value())?;
        }*/

        //self.data.images_in_flight[image_index] = in_flight_fence;

        let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index]];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        //self.device.reset_fences(&[in_flight_fence])?;

        self.device
            .queue_submit(self.data.graphics_queue, &[submit_info], vk::Fence::null())?;

        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        self.device.queue_present_khr(self.data.present_queue, &present_info)?;
        self.device.queue_wait_idle(self.data.present_queue)?;

        self.frame = (self.frame + 1) % 2;

        Ok(())
    }

    pub unsafe fn destroy(&mut self) {
        println!("Goodbye!");

        
        self.data.image_available_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));
        self.data.render_finished_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));

        self.data.framebuffers.iter().for_each(|f| self.device.destroy_framebuffer(*f, None));

        self.data.command_buffers.iter().for_each(|b| self.device.free_command_buffers(
            self.data.command_pool, &[*b]));
        
        self.device.destroy_command_pool(self.data.command_pool, None);

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


