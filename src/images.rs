use vulkanalia::prelude::v1_0::*;
use anyhow::Result;
use log::*;
use std::fs::File;

use crate::{app::AppData, buffers::{create_buffer, fill_buffer}};


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
    


    let indicies = &[data.queue_family_indicies.graphics];


    let info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .format(vk::Format::R8G8B8A8_SRGB)
        .extent(vk::Extent3D {width, height, depth: 1})
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .queue_family_indices(indicies)
        .initial_layout(vk::ImageLayout::UNDEFINED);
    

    data.texture_image = device.create_image(&info, None)?;


    let requirements = device.get_image_memory_requirements(data.texture_image);


    return Ok(());
}


