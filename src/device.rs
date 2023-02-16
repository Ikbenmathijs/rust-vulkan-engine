use vulkanalia::{prelude::v1_0::*, vk::PhysicalDevice};
use anyhow::{Result, anyhow};
use log::*;


pub unsafe fn pick_physical_device(instance: &Instance) -> Result<vk::PhysicalDevice> {

    let devices = instance.enumerate_physical_devices().unwrap();

    for physical_device in devices {
        let props = instance.get_physical_device_properties(physical_device);
        debug!("Device found: {}", props.device_name);
        if let Err(e) = QueueFamilyIndices::get(instance, &physical_device) {
            warn!("Skipping {}: it doesn't support graphics queue family", props.device_name)
        } else {
            info!("Picked device: {}", props.device_name);
            return Ok(physical_device);
        }
    }
    return Err(anyhow!("No suitable physical device could be found."));
}


pub unsafe fn create_logical_device(instance: &Instance, physical_device: &vk::PhysicalDevice) -> Result<Device> {
    let queue_family_indices = QueueFamilyIndices::get(instance, physical_device)?;

    let graphics_queue_info =  vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_indices.graphics)
        .queue_priorities(&[1.0]);

    let layers = [vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation").as_ptr()];
    let queue_infos = &[graphics_queue_info];

    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(queue_infos)
        .enabled_layer_names(&layers);

    return Ok(instance.create_device(*physical_device, &info, None)?);

}


struct QueueFamilyIndices {
    graphics: u32
}

impl QueueFamilyIndices {
    pub unsafe fn get(instance: &Instance, physical_device: &vk::PhysicalDevice) -> Result<Self> {
        let props = instance.get_physical_device_queue_family_properties(*physical_device);

        let mut graphics: Option<u32> = None;
        for (i, queueFamilyProperty) in props.iter().enumerate() {
            if (queueFamilyProperty.queue_flags.contains(vk::QueueFlags::GRAPHICS)) {
                graphics = Some(i as u32);
                debug!("\tgraphics queue family index: {}", i);
            }
        }

        if let Some(graphics) = graphics {
            return Ok(Self { graphics });
        } else {
            return Err(anyhow!("This GPU doesn't support a graphics queue."));
        }

    }
}