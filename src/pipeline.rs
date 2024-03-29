use vulkanalia::{prelude::v1_0::*};
use anyhow::{Result, anyhow};
use log::*;
use crate::vertex::Vertex;

use crate::{app::AppData, render_pass::create_render_pass};



pub unsafe fn create_pipeline(instance: &Instance, data: &mut AppData, device: &Device) -> Result<()> {


    let binding_descriptions = [Vertex::binding_description()];
    let attribute_descriptions = Vertex::attribute_description();

    let vertex_input_stage = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    let input_assembly_stage =  vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    let vertex_shader_bytecode = include_bytes!("shaders/vertex.spv");
    let fragment_shader_bytecode = include_bytes!("shaders/fragment.spv");

    let vertex_shader_module = create_shader_module(device, vertex_shader_bytecode)?;
    let fragment_shader_module = create_shader_module(device, fragment_shader_bytecode)?;


    let vertex_stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vertex_shader_module)
        .name(b"main\0");

    let fragment_stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(fragment_shader_module)
        .name(b"main\0");


    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(data.swapchain_extent.width as f32)
        .height(data.swapchain_extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);


    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D {x: 0, y: 0})
        .extent(data.swapchain_extent);

    let viewports = &[viewport];
    let scissors = &[scissor];

    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissors(scissors);


    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false)
        .line_width(1.0);


    let multi_sample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(data.msaa_samples);

    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD);


    let attachments = &[attachment];

    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .attachments(attachments)
        .logic_op_enable(false)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let vert_push_constant_range = vk::PushConstantRange::builder()
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .offset(0)
        .size(64);

    let frag_push_constant_range = vk::PushConstantRange::builder()
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .offset(64)
        .size(16);

    


    let set_layouts = &[data.descriptor_set_layout];
    let push_constant_ranges = &[vert_push_constant_range, frag_push_constant_range];

    let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(set_layouts)
        .push_constant_ranges(push_constant_ranges);

    data.pipeline_layout = device.create_pipeline_layout(&pipeline_layout_info, None)?;


    let render_pass = create_render_pass(instance, device, data)?;

    data.render_pass = render_pass;

    let stages = &[vertex_stage_info, fragment_stage_info];

    let depth_stencil_stage = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS)
        .stencil_test_enable(false);





    let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_stage)
        .input_assembly_state(&input_assembly_stage)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multi_sample_state)
        .color_blend_state(&color_blend_state)
        .depth_stencil_state(&depth_stencil_stage)
        .layout(data.pipeline_layout)
        .render_pass(render_pass)
        .subpass(0);

    data.pipeline = device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)?.0;


    device.destroy_shader_module(vertex_shader_module, None);
    device.destroy_shader_module(fragment_shader_module, None);

    info!("Created pipeline!");

    return Ok(());
}




unsafe fn create_shader_module(device: &Device, bytecode: &[u8]) -> Result<vk::ShaderModule> {

    let (prefix, aligned_bytes, suffix) = bytecode.align_to::<u32>();

    if !prefix.is_empty() || !suffix.is_empty() {
        return Err(anyhow!("Bytecode isn't aligned properly!"));
    }


    let info = vk::ShaderModuleCreateInfo::builder()
        .code(aligned_bytes)
        .code_size(bytecode.len());

    debug!("Shader module created with bytecode size {}", bytecode.len());

    return Ok(device.create_shader_module(&info, None)?);    
}