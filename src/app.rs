use vulkanalia::{loader::{LibloadingLoader, LIBRARY}, vk::{DebugUtilsMessengerEXT, ExtDebugUtilsExtension}};
use winit::window::{Window};
use anyhow::{Result, anyhow};
use vulkanalia::prelude::v1_0::*;
use crate::{instance::create_instance, device::{pick_physical_device, create_logical_device}};
use log::*;


#[derive(Clone, Debug, Default)]
pub struct AppData {
    pub messenger: DebugUtilsMessengerEXT,
    pub physical_device: vk::PhysicalDevice
}


#[derive(Clone, Debug)]
pub struct App {
    entry: Entry,
    instance: Instance,
    data: AppData,
    device: Device
}


impl App {
    pub unsafe fn Create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|e| anyhow!(e))?;
        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;
        data.physical_device = pick_physical_device(&instance)?;
        let device = create_logical_device(&instance, &data.physical_device)?;
        return Ok(Self {entry, instance, data, device});
    }

    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {

        return Ok(());
    }

    pub unsafe fn destroy(&mut self) {
        println!("Goodbye!");

        self.device.destroy_device(None);

        self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None);
        self.instance.destroy_instance(None);
    }
}


