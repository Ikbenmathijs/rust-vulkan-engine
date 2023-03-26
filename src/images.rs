use vulkanalia::prelude::v1_0::*;
use anyhow::{anyhow, Result};
use log::*;
use std::fs::File;

use crate::{app::AppData, buffers::{create_buffer, fill_buffer, get_memory_type_index, begin_single_time_commands, end_single_time_commands}};


/*unsafe fn create_image(format: vk::Format, width: u32, height: u32) -> Result<vk::Image> {
    let info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .extent(vk::Extent2D{width, height});

}*/


pub unsafe fn create_image_view(image: &vk::Image,
    device: &Device,
    format: vk::Format, 
    subresource: vk::ImageSubresourceRange,
    ) -> Result<vk::ImageView> {

    let info = vk::ImageViewCreateInfo::builder()
        .image(*image)
        .subresource_range(subresource)
        .view_type(vk::ImageViewType::_2D)
        .format(format);
        
    debug!("Image view has been created");
    return Ok(device.create_image_view(&info, None)?);
}



pub unsafe fn create_texture_image(instance: &Instance, device: &Device, data: &mut AppData) -> Result<()> {

    let image = File::open("resources/texture.png")?;

    let decorder = png::Decoder::new(image);
    let mut reader = decorder.read_info()?;



    let mut pixels = vec![0; reader.info().raw_bytes()];
    reader.next_frame(&mut pixels)?;

    let size = reader.info().raw_bytes() as u64;
    let (width, height) = reader.info().size();


    let (staging_buffer, staging_buffer_memory) = create_buffer(
        size, 
        vk::BufferUsageFlags::TRANSFER_SRC, 
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, 
        device, 
        instance, 
        data)?;

    
    fill_buffer(&staging_buffer, 
        &staging_buffer_memory, 
        &size, 
        pixels.as_ptr(), 
        pixels.len(), 
        device)?;


    let (image, image_memory) = create_image(instance, device, data, size, width, height, vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)?;
    
    device.bind_image_memory(image, image_memory, 0)?;

    transition_image_layout(device, data, image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL)?;


    copy_buffer_to_image(device, data, staging_buffer, image, width, height)?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    transition_image_layout(device, data, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)?;


    data.texture_image = image;
    data.texture_image_memory = image_memory;


    return Ok(());
}


pub unsafe fn create_texture_image_view(device: &Device, data: &mut AppData) -> Result<()> {


    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1).build();


    data.texture_image_view = create_image_view(&data.texture_image, device, vk::Format::R8G8B8A8_SRGB, subresource)?;



    return Ok(());
}

pub unsafe fn create_texture_sampler(device: &Device, data: &mut AppData) -> Result<()> {

    let create_info = vk::SamplerCreateInfo::builder()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .anisotropy_enable(true)
        .max_anisotropy(16.0)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .compare_op(vk::CompareOp::ALWAYS)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .mip_lod_bias(0.0)
        .min_lod(0.0)
        .max_lod(0.0);


    data.texture_image_sampler = device.create_sampler(&create_info, None)?;

    

    return Ok(());
}


pub unsafe fn transition_image_layout(
    device: &Device,
    data: &AppData,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout
) -> Result<()> {
    
    let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) = match (old_layout, new_layout) {
        (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
            vk::AccessFlags::empty(),
            vk::AccessFlags::TRANSFER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER
        ),
        (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER
        ),
        _ => return Err(anyhow!("Not a supported image transition."))
    };


    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);


    let barrier = vk::ImageMemoryBarrier::builder()
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask)
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(data.queue_family_indicies.graphics)
        .dst_queue_family_index(data.queue_family_indicies.graphics)
        .image(image)
        .subresource_range(subresource);



    let command_buffer = begin_single_time_commands(device, data)?;

    device.cmd_pipeline_barrier(
        command_buffer, 
        src_stage_mask, 
        dst_stage_mask, 
        vk::DependencyFlags::empty(), 
        &[] as &[vk::MemoryBarrier], 
        &[] as &[vk::BufferMemoryBarrier], 
        &[barrier]);


    end_single_time_commands(device, data, command_buffer)?;


    debug!("Image layout transitioned from {:?} to {:?}", old_layout, new_layout);

    return Ok(());
}


pub unsafe fn copy_buffer_to_image(
    device: &Device,
    data: &AppData,
    src: vk::Buffer,
    dst: vk::Image,
    width: u32,
    height: u32,
 ) -> Result<()> {

    let command_buffer = begin_single_time_commands(device, data)?;

    let subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1);

    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(subresource)
        .image_offset(vk::Offset3D {x: 0, y: 0, z: 0})
        .image_extent(vk::Extent3D {width, height, depth: 1});

    let regions = &[region];

    device.cmd_copy_buffer_to_image(command_buffer, 
        src, 
        dst, 
        vk::ImageLayout::TRANSFER_DST_OPTIMAL, regions);

    
    end_single_time_commands(device, data, command_buffer)?;

    debug!("Buffer copied to image!");

    return Ok(());
}

pub unsafe fn create_image(instance: &Instance, device: &Device, data: &mut AppData, size: u64, width: u32, height: u32, usage: vk::ImageUsageFlags) -> Result<(vk::Image, vk::DeviceMemory)> {



    let indicies = &[data.queue_family_indicies.graphics];

    let info = vk::ImageCreateInfo::builder()
    .image_type(vk::ImageType::_2D)
    .format(vk::Format::R8G8B8A8_SRGB)
    .extent(vk::Extent3D {width, height, depth: 1})
    .mip_levels(1)
    .array_layers(1)
    .samples(vk::SampleCountFlags::_1)
    .tiling(vk::ImageTiling::OPTIMAL)
    .usage(usage)
    .sharing_mode(vk::SharingMode::EXCLUSIVE)
    .queue_family_indices(indicies)
    .initial_layout(vk::ImageLayout::UNDEFINED);


    let image = device.create_image(&info, None)?;

    let requirements = device.get_image_memory_requirements(image);

    debug!("Pixels size: {}, requirements size: {}", size, requirements.size);

    let allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(size)
        .memory_type_index(
            get_memory_type_index(
            instance, 
            data, 
            vk::MemoryPropertyFlags::DEVICE_LOCAL, 
            requirements)?);
    

    let memory = device.allocate_memory(&allocate_info, None)?;

    debug!("Image has been created!");

    return Ok((image, memory));
}

