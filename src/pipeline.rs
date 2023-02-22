use vulkanalia::{prelude::v1_0::*, vk::PipelineInputAssemblyStateCreateInfo};
use anyhow::Result;

use crate::app::AppData;



pub unsafe fn create_pipeline(data: &mut AppData, device: &Device) -> Result<()> {



    let vertex_input_stage = vk::PipelineVertexInputStateCreateInfo::builder();

    let input_assembly_stage =  PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);



    return Ok(());
}



