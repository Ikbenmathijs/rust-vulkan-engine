use vulkanalia::vk::{Image, ImageCreateInfo, HasBuilder};
use vulkanalia::prelude::v1_0::*;
use anyhow::Result;


/*unsafe fn create_image(format: vk::Format, width: u32, height: u32) -> Result<vk::Image> {
    let info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .extent(vk::Extent2D{width, height});

}*/