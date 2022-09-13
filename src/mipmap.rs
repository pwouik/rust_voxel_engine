use crate::texture::Texture;
use std::borrow::Cow;
use std::default::Default;
use wgpu::ShaderModule;

const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/*pub fn generate_mipmaps(
    encoder: &mut wgpu::CommandEncoder,
    device: &wgpu::Device,
    texture: &wgpu::Texture,
    levels: u32,
    mip_count: u32,
) {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("mipmap.wgsl"))),
    });
    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                    },
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&texture_bind_group_layout],
        push_constant_ranges: &[wgpu::PushConstantRange {
            stages: wgpu::ShaderStages::COMPUTE,
            range: 0..4,
        }],
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "main",
    });
    for target_mip in 1u32..mip_count {
        let view_in = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            base_mip_level: target_mip - 1,
            ..wgpu::TextureViewDescriptor::default()
        });
        let view_out = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            base_mip_level: target_mip,
            ..wgpu::TextureViewDescriptor::default()
        });
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view_in),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view_out),
                },
            ],
            label: Some("diffuse_bind_group"),
        });
        let mut compute_pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_push_constants(0, &target_mip.to_ne_bytes());
        compute_pass.set_bind_group(0, &texture_bind_group, &[]);
        compute_pass.dispatch_workgroups(1, 1, levels);
    }
}*/
