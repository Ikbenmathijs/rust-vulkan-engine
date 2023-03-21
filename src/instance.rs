use vulkanalia::{prelude::v1_0::*, vk::ExtDebugUtilsExtension};
use anyhow::Result;
use winit::window::{Window};
use std::{ffi::CStr, str::from_utf8};
use log::*;
use std::ffi::c_void;

use crate::app::AppData;

/*
.iter().for_each(|l| {
        let a = *l.layer_name;
        println!("{}", from_utf8( unsafe {std::slice::from_raw_parts(a.as_ptr() as *const u8, a.len())}).unwrap());
    });


 */


pub unsafe fn create_instance(window: &Window, entry: &Entry, data: &mut AppData) -> Result<Instance> {

    let app_info = vk::ApplicationInfo::builder()
        .application_version(0)
        .api_version(vk::make_version(1, 0, 0));


    
    let mut extentions = vulkanalia::window::get_required_instance_extensions(window).iter().map(|e| e.as_ptr()).collect::<Vec<_>>();

    extentions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());

    let layers = [vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation").as_ptr()];

    let info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extentions);

    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
        .user_callback(Some(debug_callback));

    info.push_next(&mut debug_info);

    let instance = entry.create_instance(&info, None)?;

    data.messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
    
    info!("Instance created!");
    return Ok(instance);
}



extern "system" fn debug_callback(

    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void
) -> vk::Bool32 {
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        error!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        debug!("({:?}) {}", type_, message);
    } else {
        trace!("({:?}) {}", type_, message);
    }

    

    return vk::FALSE;
}