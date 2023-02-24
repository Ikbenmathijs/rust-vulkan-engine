use vulkanalia::prelude::v1_0::*;
use log::*;
use anyhow::Result;

use crate::app::AppData;



pub unsafe fn create_semaphore(device: &Device) -> Result<vk::Semaphore> {

    let info = vk::SemaphoreCreateInfo::builder();


    return Ok(device.create_semaphore(&info, None)?);
}


pub unsafe fn create_fence(device: &Device) -> Result<vk::Fence> {
    let info = vk::FenceCreateInfo::builder();

    return Ok(device.create_fence(&info, None)?);
}