use glm::translate;
use vulkanalia::{loader::{LibloadingLoader, LIBRARY}, vk::{DebugUtilsMessengerEXT, ExtDebugUtilsExtension, KhrSurfaceExtension, KhrSwapchainExtension}};
use winit::window::{Window};
use anyhow::{Result, anyhow};
use vulkanalia::prelude::v1_0::*;
use crate::{instance::create_instance, device::{pick_physical_device, create_logical_device, QueueFamilyIndices}, swapchain::{create_swapchain, create_swapchain_image_views}, pipeline::create_pipeline, buffers::{create_framebuffers, create_command_pools, create_command_buffers}, sync::{create_semaphore, create_fence}, render_pass::create_render_pass, vertex::{create_vertex_buffer, create_index_buffer, Vertex, load_model}, ubo::{ create_uniform_buffers, MVP_UBO}, images::{create_texture_image, create_texture_image_view, create_texture_sampler, create_depth_buffer, create_color_buffer}};
use log::*;
use vulkanalia::window as vkWindow;
use std::time::Instant;
use std::mem::size_of;
use nalgebra_glm as glm;
use std::ptr::copy_nonoverlapping as memcpy;
use crate::descriptors::{create_descriptor_pool, create_descriptor_sets, create_descriptor_set_layout};



#[derive(Debug, Default)]
pub struct AppData {
    pub messenger: DebugUtilsMessengerEXT,
    pub physical_device: vk::PhysicalDevice,
    pub msaa_samples: vk::SampleCountFlags,
    pub surface: vk::SurfaceKHR,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_format: vk::Format,
    pub swapchain_image_views: Vec<vk::ImageView>,
    pub swapchain_extent: vk::Extent2D,
    pub pipeline_layout: vk::PipelineLayout,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub render_pass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub color_image: vk::Image,
    pub color_image_memory: vk::DeviceMemory,
    pub color_image_view: vk::ImageView,
    pub command_pools: Vec<vk::CommandPool>,
    pub transient_command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub secondary_command_buffers: Vec<Vec<vk::CommandBuffer>>,
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffer_memory: Vec<vk::DeviceMemory>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub queue_family_indicies: QueueFamilyIndices,
    pub mip_levels: u32,
    pub texture_image: vk::Image,
    pub texture_image_memory: vk::DeviceMemory,
    pub texture_image_view: vk::ImageView,
    pub texture_image_sampler: vk::Sampler,
    pub depth_image: vk::Image,
    pub depth_image_memory: vk::DeviceMemory,
    pub depth_image_view: vk::ImageView,
    pub vertices: Vec<Vertex>,
    pub indicies: Vec<u32>
}


#[derive(Debug)]
pub struct App {
    entry: Entry,
    instance: Instance,
    data: AppData,
    device: Device,
    frame: usize,
    start: Instant,
    pub models: usize
}


impl App {
    pub unsafe fn Create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|e| anyhow!(e))?;
        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;
        data.surface = vkWindow::create_surface(&instance, window)?;
        pick_physical_device(&instance, &mut data)?;
        let device = create_logical_device(&instance, &mut data)?;


        data.queue_family_indicies = QueueFamilyIndices::get(&instance, &mut data, None)?;


        create_swapchain(&instance, &mut data, &device, window)?;
        create_swapchain_image_views(&mut data, &device)?;


        create_command_pools(&device, &mut data)?;


        create_texture_image(&instance, &device, &mut data)?;
        create_texture_image_view(&device, &mut data)?;
        create_texture_sampler(&device, &mut data)?;


        


        load_model(&mut data)?;

        create_vertex_buffer(&instance, &device, &mut data)?;
        create_index_buffer(&instance, &device, &mut data)?;


        
        create_color_buffer(&instance, &device, &mut data)?;
        create_depth_buffer(&instance, &device, &mut data)?;

        create_descriptor_set_layout(&device, &mut data)?;
        create_uniform_buffers(&instance, &device, &mut data)?;
        create_descriptor_pool(&device, &mut data)?;

        create_descriptor_sets(&device, &mut data)?;

        create_pipeline(&instance, &mut data, &device)?;
        create_framebuffers(&mut data, &device)?;



        create_command_buffers(&device, &mut data)?;

        

    
        

        for _ in 0..data.swapchain_images.len() {
            data.render_finished_semaphores.push(create_semaphore(&device)?);
            data.image_available_semaphores.push(create_semaphore(&device)?);
            data.in_flight_fences.push(create_fence(&device, true)?);
        }
        
        data.images_in_flight = data.swapchain_images.iter().map(|_| vk::Fence::null()).collect();



        return Ok(Self {entry, instance, data, device, frame: 0, start: Instant::now(), models: 4});
    }

    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {



        let in_flight_fence = self.data.in_flight_fences[self.frame];



        self.device
            .wait_for_fences(&[in_flight_fence], true, u64::max_value())?;




        let image_index = self
            .device
            .acquire_next_image_khr(
                self.data.swapchain,
                u64::max_value(),
                self.data.image_available_semaphores[self.frame],
                vk::Fence::null(),
            )?
            .0 as usize;

        

        let image_in_flight = self.data.images_in_flight[image_index];
        if !image_in_flight.is_null() {
            self.device
                .wait_for_fences(&[image_in_flight], true, u64::max_value())?;
        }

        self.data.images_in_flight[image_index] = in_flight_fence;


        self.update_uniform_buffers(image_index)?;
        self.update_command_buffer(image_index)?;

        let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index]];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        self.device.reset_fences(&[in_flight_fence])?;

        self.device
            .queue_submit(self.data.graphics_queue, &[submit_info], in_flight_fence)?;

        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self.device.queue_present_khr(self.data.present_queue, &present_info);


        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
    || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);


        if changed {
            self.recreate_swapchain(window)?;
        } else if let Err(e) = result {
            return Err(anyhow!(e));
        }



        self.frame = (self.frame + 1) % 2;

        Ok(())
    }


    unsafe fn update_command_buffer(&mut self, image_index: usize) -> Result<()> {

        let command_pool = self.data.command_pools[image_index];
        self.device.reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty())?;

        let command_buffer = self.data.command_buffers[image_index];
            
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();

        self.device.begin_command_buffer(command_buffer, &command_buffer_begin_info)?;

        let render_area = vk::Rect2D {
            offset: vk::Offset2D {x: 0, y: 0}, 
            extent: vk::Extent2D {width: self.data.swapchain_extent.width, height: self.data.swapchain_extent.height}};
        
        let clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0]
            }
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0
            }
        };

        let clear_values = &[clear_value, depth_clear_value];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.data.render_pass)
            .framebuffer(self.data.framebuffers[image_index])
            .render_area(render_area)
            .clear_values(clear_values);



        self.device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::SECONDARY_COMMAND_BUFFERS);
       
        let secondary_command_buffers = (0..self.models)
            .map(|i| self.update_secondary_command_buffer(image_index, i))
            .collect::<Result<Vec<_>, _>>()?;

        self.device.cmd_execute_commands(command_buffer, &secondary_command_buffers);

        self.device.cmd_end_render_pass(command_buffer);
        self.device.end_command_buffer(command_buffer)?;



        return Ok(());
    }



    unsafe fn update_secondary_command_buffer(&mut self, image_index: usize, model_index: usize) -> Result<vk::CommandBuffer> {

        self.data.secondary_command_buffers.resize_with(image_index + 1, Vec::new);
        let command_buffers = &mut self.data.secondary_command_buffers[image_index];
        while model_index >= command_buffers.len() {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.data.command_pools[image_index])
                .level(vk::CommandBufferLevel::SECONDARY)
                .command_buffer_count(1);

            command_buffers.push(self.device.allocate_command_buffers(&allocate_info)?[0]);
        }

        let command_buffer = command_buffers[model_index];

        let inhenritance_info = vk::CommandBufferInheritanceInfo::builder()
            .render_pass(self.data.render_pass)
            .subpass(0)
            .framebuffer(self.data.framebuffers[image_index]);

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE)
            .inheritance_info(&inhenritance_info);

        self.device.begin_command_buffer(command_buffer, &begin_info)?;


        self.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.data.pipeline);

        self.device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.data.vertex_buffer], &[0]);
        self.device.cmd_bind_index_buffer(command_buffer, self.data.index_buffer, 0, vk::IndexType::UINT32);
        

        self.device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.data.pipeline_layout, 0, &[self.data.descriptor_sets[image_index]], &[]);

       let y = (((model_index % 2) as f32) * 2.5) - 1.25;
        let z = (((model_index / 2) as f32) * -2.0) + 1.0;

        let model = glm::translate(
            &glm::identity(),
            &glm::vec3(0.0, y, z),
        );

        let time = self.start.elapsed().as_secs_f32();

        let model = glm::rotate(
            &model,
            time * glm::radians(&glm::vec1(90.0))[0],
            &glm::vec3(0.0, 0.0, 1.0),
        );

        /*let model = glm::rotate(
            &model,
            glm::radians(&glm::vec1(90.0))[0],
            &glm::vec3(1.0, 0.0, 0.0)
        );

        let model = glm::translate(
            &model, 
            &glm::vec3(0.0, -1.0, 0.0));*/
        
        let (_, model_bytes, _) = model.as_slice().align_to::<u8>();


        self.device.cmd_push_constants(
            command_buffer, 
            self.data.pipeline_layout, 
            vk::ShaderStageFlags::VERTEX, 
            0, 
            model_bytes
        );

        let light_dir = glm::normalize(&glm::vec3::<f32>(1.0, -3.0, -1.0));

        let (_, light_dir_bytes, _) = light_dir.as_slice().align_to::<u8>();

        self.device.cmd_push_constants(
            command_buffer, 
            self.data.pipeline_layout, 
            vk::ShaderStageFlags::FRAGMENT, 
            64, 
            light_dir_bytes);



        let opacity = (model_index + 1) as f32 * 0.25;

        self.device.cmd_push_constants(
            command_buffer, 
            self.data.pipeline_layout, 
            vk::ShaderStageFlags::FRAGMENT, 
            76, 
            &opacity.to_ne_bytes()[..]
        );


        self.device.cmd_draw_indexed(command_buffer, self.data.indicies.len() as u32, 1, 0, 0, 0);


        self.device.end_command_buffer(command_buffer)?;

        return Ok(command_buffer);
    }


    unsafe fn update_uniform_buffers(&self, image_index: usize) -> Result<()> {


        let view = glm::look_at(
            &glm::vec3(6.0, 0.0, 2.0),
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 0.0, 1.0),
        );

        let mut proj = glm::perspective_rh_zo(
            self.data.swapchain_extent.width as f32 / self.data.swapchain_extent.height as f32,
            glm::radians(&glm::vec1(45.0))[0],
            0.1,
            10.0,
        );

        proj[(1, 1)] *= -1.0;

        let ubo = MVP_UBO { view, proj };

        // Copy

        let memory = self.device.map_memory(
            self.data.uniform_buffer_memory[image_index],
            0,
            size_of::<MVP_UBO>() as u64,
            vk::MemoryMapFlags::empty(),
        )?;

        memcpy(&ubo, memory.cast(), 1);

        self.device.unmap_memory(self.data.uniform_buffer_memory[image_index]);


        Ok(())

    }

    pub unsafe fn destroy_swapchain(&mut self) {


        self.device.destroy_image(self.data.color_image, None);
        self.device.free_memory(self.data.color_image_memory, None);
        self.device.destroy_image_view(self.data.color_image_view, None);
        debug!("Destroyed color buffer!");

        self.device.destroy_image(self.data.depth_image, None);
        self.device.free_memory(self.data.depth_image_memory, None);
        self.device.destroy_image_view(self.data.depth_image_view, None);
        debug!("Destroyed depth buffer");






        self.data.framebuffers.iter().for_each(|f| self.device.destroy_framebuffer(*f, None));
        debug!("Destroyed frame buffers");


   
        self.data.command_pools.iter().for_each(|p| 
            self.device.destroy_command_pool(*p, None)
        );

        self.data.command_pools = vec![] as Vec<vk::CommandPool>;
        self.data.command_buffers = vec![] as Vec<vk::CommandBuffer>;

        
        
        self.device.destroy_command_pool(self.data.transient_command_pool, None);
        debug!("Destroyed command pools");

        self.device.destroy_descriptor_pool(self.data.descriptor_pool, None);
        debug!("Destroyed descriptor pool!");

        self.device.destroy_render_pass(self.data.render_pass, None);
        debug!("Destroyed render pass");


        self.data.uniform_buffers.iter().for_each(|b| self.device.destroy_buffer(*b, None));
        self.data.uniform_buffer_memory.iter().for_each(|m| self.device.free_memory(*m, None));



        self.device.destroy_pipeline(self.data.pipeline, None);
        debug!("Destroyed pipeline");



        self.device.destroy_pipeline_layout(self.data.pipeline_layout, None);
        debug!("Destroyed pipeline layout");


 
        for view in &self.data.swapchain_image_views {
            self.device.destroy_image_view(*view, None);
        }
        debug!("Destroyed image views");
        self.device.destroy_swapchain_khr(self.data.swapchain, None);
        debug!("Destroyed swapchain");
    }

    pub unsafe fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        self.device.device_wait_idle()?;

        self.destroy_swapchain();

        create_swapchain(&self.instance, &mut self.data, &self.device, window)?;
        create_swapchain_image_views(&mut self.data, &self.device)?;

        create_command_pools(&self.device, &mut self.data)?;
        create_command_buffers(&self.device, &mut self.data)?;

        create_color_buffer(&self.instance, &self.device, &mut self.data)?;
        create_depth_buffer(&self.instance, &self.device, &mut self.data)?;

        create_uniform_buffers(&self.instance, &self.device, &mut self.data)?;

        create_pipeline(&self.instance, &mut self.data, &self.device)?;


        create_descriptor_pool(&self.device, &mut self.data)?;

        create_descriptor_sets(&self.device, &mut self.data)?;


        create_framebuffers(&mut self.data, &self.device)?;
        

        
        info!("Swapchain & related objects have been re-created!");
        
        return Ok(());
    }

    pub unsafe fn destroy(&mut self) {
        println!("Goodbye!");
        self.device.device_wait_idle().unwrap();

        
        self.data.image_available_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));
        self.data.render_finished_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));
        self.data.in_flight_fences.iter().for_each(|f| self.device.destroy_fence(*f, None));
        debug!("Destroyed fences & semaphores");
        
        self.destroy_swapchain();

        self.device.destroy_sampler(self.data.texture_image_sampler, None);
        self.device.destroy_image_view(self.data.texture_image_view, None);
        self.device.destroy_image(self.data.texture_image, None);
        self.device.free_memory(self.data.texture_image_memory, None);
        debug!("Destroyed textre");

        
        

        self.device.destroy_buffer(self.data.vertex_buffer, None);
        self.device.free_memory(self.data.vertex_buffer_memory, None);
        self.device.destroy_buffer(self.data.index_buffer, None);
        self.device.free_memory(self.data.index_buffer_memory, None);
        debug!("Destroyed vertex & index buffers");

        self.device.destroy_descriptor_set_layout(self.data.descriptor_set_layout, None);


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


