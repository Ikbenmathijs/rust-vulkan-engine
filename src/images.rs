use vulkanalia::prelude::v1_0::*;
use anyhow::{anyhow, Result};
use log::*;
use std::fs::File;
use png::ColorType;

use crate::{app::AppData, buffers::{create_buffer, fill_buffer, get_memory_type_index, begin_single_time_commands, end_single_time_commands}};


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


    debug!("Color type is {:?}", reader.info().color_type);


    let mut buffer = vec![0; reader.info().raw_bytes()];
    reader.next_frame(&mut buffer)?;

    let pixels: Vec<u8>;


    if reader.info().color_type == ColorType::Rgb {
        // convert to RGBA
        pixels = buffer.chunks_exact(3)
        .flat_map(|rgb| {
            vec!(rgb[0], rgb[1], rgb[2], 255)
        })
        .collect::<Vec<_>>();
        debug!("Image converted to RGBA")
    } else {
        pixels = buffer;
    }


    let size = pixels.len() as u64;
    let (width, height) = reader.info().size();


    data.mip_levels = (width.max(height) as f32).log2().floor() as u32 + 1;


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


    let (image, image_memory) = create_image(
        instance, 
        device, 
        data, 
        width, 
        height, 
        vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST | 
        vk::ImageUsageFlags::TRANSFER_SRC,
    vk::Format::R8G8B8A8_SRGB,
        data.mip_levels)?;
    
    device.bind_image_memory(image, image_memory, 0)?;

    transition_image_layout(
        device, 
        data, 
        image, 
        vk::ImageLayout::UNDEFINED, 
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        data.mip_levels
    )?;


    copy_buffer_to_image(device, data, staging_buffer, image, width, height)?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);


    generate_mipmaps(
        instance, 
        device, 
        data, 
        image, 
        vk::Format::R8G8B8A8_SRGB,
        width, 
        height, 
        data.mip_levels)?;

    


    data.texture_image = image;
    data.texture_image_memory = image_memory;


    return Ok(());
}


pub unsafe fn generate_mipmaps(
    instance: &Instance, 
    device: &Device,
    data: &AppData,
    image: vk::Image,
    format: vk::Format,
    width: u32,
    height: u32,
    mip_levels: u32
) -> Result<()> {


    if !instance.get_physical_device_format_properties(data.physical_device, format)
        .optimal_tiling_features
        .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR) 
    {
        return Err(anyhow!("This device does not support linear blitting"));
    }


    let command_buffer = begin_single_time_commands(device, data)?;

    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_array_layer(0)
        .layer_count(1)
        .level_count(1);


    let mut barrier = vk::ImageMemoryBarrier::builder()
        .image(image)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(subresource);

    let mut mip_width = width;
    let mut mip_height = height;

    for i in 1..mip_levels {
        barrier.subresource_range.base_mip_level = i - 1;
        barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;


        device.cmd_pipeline_barrier(command_buffer, 
            vk::PipelineStageFlags::TRANSFER, 
            vk::PipelineStageFlags::TRANSFER, 
            vk::DependencyFlags::empty(), 
            &[] as &[vk::MemoryBarrier], 
            &[] as &[vk::BufferMemoryBarrier], 
            &[barrier]);

        
        let src_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(i - 1)
            .base_array_layer(0)
            .layer_count(1);
            
        
        let dst_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(i)
            .base_array_layer(0)
            .layer_count(1);



        let blit = vk::ImageBlit::builder()
            .src_offsets([
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: mip_width as i32,
                    y: mip_height as i32,
                    z: 1
                }
            ])
            .src_subresource(src_subresource)
            .dst_offsets([
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: (if mip_width > 1 { mip_width / 2 } else { 1 }) as i32,
                    y: (if mip_height > 1 { mip_height / 2 } else { 1 }) as i32,
                    z: 1
                }
            ])
            .dst_subresource(dst_subresource);


        device.cmd_blit_image(
            command_buffer, 
            image, 
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL, 
            image, 
            vk::ImageLayout::TRANSFER_DST_OPTIMAL, 
            &[blit], 
            vk::Filter::LINEAR);

        
        barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
        barrier.dst_access_mask =   vk::AccessFlags::SHADER_READ;


        device.cmd_pipeline_barrier(
            command_buffer, 
            vk::PipelineStageFlags::TRANSFER, 
            vk::PipelineStageFlags::FRAGMENT_SHADER, 
            vk::DependencyFlags::empty(), 
            &[] as &[vk::MemoryBarrier], 
            &[] as &[vk::BufferMemoryBarrier], 
            &[barrier]);

        
        if mip_width > 1 {
            mip_width /= 2;
        }
        
        if mip_height > 1 {
            mip_height /= 2;
        }
    }

    barrier.subresource_range.base_mip_level = mip_levels - 1;
    barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
    barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

    device.cmd_pipeline_barrier(
        command_buffer, 
        vk::PipelineStageFlags::TRANSFER, 
        vk::PipelineStageFlags::FRAGMENT_SHADER, 
        vk::DependencyFlags::empty(), 
        &[] as &[vk::MemoryBarrier], 
        &[] as &[vk::BufferMemoryBarrier], 
        &[barrier]);


    end_single_time_commands(device, data, command_buffer)?;


    return Ok(());
}


pub unsafe fn create_texture_image_view(device: &Device, data: &mut AppData) -> Result<()> {


    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(data.mip_levels)
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
        .max_lod(data.mip_levels as f32);


    data.texture_image_sampler = device.create_sampler(&create_info, None)?;

    

    return Ok(());
}


pub unsafe fn transition_image_layout(
    device: &Device,
    data: &AppData,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    mip_levels: u32
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
        .level_count(mip_levels)
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
    height: u32
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

pub unsafe fn create_image(instance: &Instance, 
    device: &Device, 
    data: &mut AppData, 
    width: u32, 
    height: u32, 
    usage: vk::ImageUsageFlags, 
    format: vk::Format,
    mip_levels: u32) -> Result<(vk::Image, vk::DeviceMemory)> {



    let indicies = &[data.queue_family_indicies.graphics];

    let info = vk::ImageCreateInfo::builder()
    .image_type(vk::ImageType::_2D)
    .format(format)
    .extent(vk::Extent3D {width, height, depth: 1})
    .mip_levels(mip_levels)
    .array_layers(1)
    .samples(vk::SampleCountFlags::_1)
    .tiling(vk::ImageTiling::OPTIMAL)
    .usage(usage)
    .sharing_mode(vk::SharingMode::EXCLUSIVE)
    .queue_family_indices(indicies)
    .initial_layout(vk::ImageLayout::UNDEFINED);


    let image = device.create_image(&info, None)?;

    let requirements = device.get_image_memory_requirements(image);


    let allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
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




pub unsafe fn get_supported_format(
    instance: &Instance,
    data: &AppData,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags
) -> Result<vk::Format> {

    candidates.iter()
        .cloned()
        .find(|f| {
            let props = instance.get_physical_device_format_properties(
                data.physical_device, 
            *f);

            match tiling {
                vk::ImageTiling::LINEAR => props.linear_tiling_features.contains(features),
                vk::ImageTiling::OPTIMAL => props.optimal_tiling_features.contains(features),
                _ => false
            }
        }).ok_or_else(|| anyhow!("None of the formats in {:?} are supported.", candidates))
}


pub unsafe fn get_depth_format(instance: &Instance, data: &AppData) -> Result<vk::Format> {
    let candidates = &[
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];

    get_supported_format(
        instance,
        data,
        candidates,
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
}



pub unsafe fn create_depth_buffer(
    instance: &Instance,
    device: &Device, 
    data: &mut AppData
) -> Result<()> {


    let format = get_depth_format(instance, data)?;

    let (depth_image, depth_image_memory) = create_image(
        instance, 
        device, 
        data, 
        data.swapchain_extent.width, 
        data.swapchain_extent.height, 
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, 
        format,
        1)?;

    device.bind_image_memory(depth_image, depth_image_memory, 0)?;


    data.depth_image = depth_image;
    data.depth_image_memory = depth_image_memory;


    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::DEPTH)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1).build();

    data.depth_image_view = create_image_view(&data.depth_image, device, format, subresource)?;





    return Ok(());
}