use vulkanalia::{prelude::v1_0::*, vk::{PhysicalDevice, KhrSurfaceExtension}};
use anyhow::{Result, anyhow};
use log::*;

use crate::app::AppData;


const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];


pub unsafe fn pick_physical_device(instance: &Instance, data: &mut AppData) -> Result<()> {

    let devices = instance.enumerate_physical_devices().unwrap();

    for physical_device in devices {
        let props = instance.get_physical_device_properties(physical_device);
        debug!("Device found: {}", props.device_name);
        if let Err(_) = QueueFamilyIndices::get(instance, data, Some(&physical_device)) {
            warn!("Skipping {}: it doesn't support graphics queue family", props.device_name);
            continue;
        }
        if let Err(_) = check_device_extentions(instance, &physical_device) {
            warn!("Skipping {}: it doesn't support all required extensions", props.device_name);
            continue;
        }

        info!("Picked device: {}", props.device_name);
        data.physical_device = physical_device;
        data.msaa_samples = get_max_msaa_samples(instance, data);
        debug!("MSAA samples: {:?}", data.msaa_samples);
        return Ok(());
    }
    return Err(anyhow!("No suitable physical device could be found."));
}


pub unsafe fn create_logical_device(instance: &Instance, data: &mut AppData) -> Result<Device> {
    let physical_device = &data.physical_device;
    let queue_family_indices = QueueFamilyIndices::get(instance, data, Some(physical_device))?;

    let graphics_queue_info =  vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_indices.graphics)
        .queue_priorities(&[1.0]);

    let layers = [vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation").as_ptr()];
    let queue_infos = &[graphics_queue_info];

    let extensions = DEVICE_EXTENSIONS.iter().map(|n| n.as_ptr()).collect::<Vec<_>>();


    let features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true);



    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);

    let device = instance.create_device(*physical_device, &info, None)?;


    data.graphics_queue = device.get_device_queue(queue_family_indices.graphics, 0);
    data.present_queue = device.get_device_queue(queue_family_indices.present, 0);

    return Ok(device);

}


#[derive(Clone, Debug, Default)]
pub struct QueueFamilyIndices {
    pub graphics: u32,
    pub present: u32
}

impl QueueFamilyIndices {
    pub unsafe fn get(instance: &Instance, data: &AppData, physical_device: Option<&PhysicalDevice>) -> Result<Self> {

        let physical_device = if let Some(device) = physical_device {device} else {&data.physical_device};

        let props = instance.get_physical_device_queue_family_properties(*physical_device);

        let mut graphics: Option<u32> = None;
        let mut present: Option<u32> = None;
        for (i, queueFamilyProperty) in props.iter().enumerate() {
            if queueFamilyProperty.queue_flags.contains(vk::QueueFlags::GRAPHICS) && graphics == None {
                graphics = Some(i as u32);
                debug!("\tgraphics queue family index: {}", i);
            }
            if instance.get_physical_device_surface_support_khr(*physical_device, i as u32, data.surface)? && present == None {
                present = Some(i as u32);
                debug!("\tpresent queue family index: {}", i);
            }
        }

        if let Some(graphics) = graphics {
            if let Some(present) = present {
                return Ok(Self { graphics, present });
            } else {
                return Err(anyhow!("This GPU doesn't support a present queue."));
            }
        } else {
            return Err(anyhow!("This GPU doesn't support a graphics queue."));
        }

    }
}



unsafe fn get_max_msaa_samples(
    instance: &Instance,
    data: &AppData
) -> vk::SampleCountFlags {
    let properties = instance.get_physical_device_properties(data.physical_device);
    let counts = properties.limits.framebuffer_color_sample_counts
        & properties.limits.framebuffer_depth_sample_counts;
    [
        vk::SampleCountFlags::_64,
        vk::SampleCountFlags::_32,
        vk::SampleCountFlags::_16,
        vk::SampleCountFlags::_8,
        vk::SampleCountFlags::_4,
        vk::SampleCountFlags::_2,
    ]
    .iter()
    .cloned()
    .find(|c| counts.contains(*c))
    .unwrap_or(vk::SampleCountFlags::_1)
}



unsafe fn check_device_extentions(instance: &Instance, physical_device: &vk::PhysicalDevice) -> Result<()> {
    let required_extensions: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];
    let extensions = instance.enumerate_device_extension_properties(*physical_device, None)?.iter().map(|e| e.extension_name).collect::<Vec<_>>();
    if required_extensions.iter().all(|e| extensions.contains(e)) {
        return Ok(());
    } else {
        return Err(anyhow!("GPU doesn't support all extentions"));
    }
}