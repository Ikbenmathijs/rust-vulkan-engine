use vulkanalia::prelude::v1_0::*;
use anyhow::Result;
use log::*;


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