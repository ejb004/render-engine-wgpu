use crate::{
    camera::camera::CameraUniform,
    wgpu_utils::{
        binding_builder::{BindGroupBuilder, BindGroupLayoutBuilder, BindGroupLayoutWithDesc},
        binding_types,
        uniform_buffer::UniformBuffer,
    },
};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GlobalUBOContent {
    camera: CameraUniform,
}

unsafe impl bytemuck::Pod for GlobalUBOContent {}
unsafe impl bytemuck::Zeroable for GlobalUBOContent {}

pub type GlobalUBO = UniformBuffer<GlobalUBOContent>;

pub fn update_global_ubo(ubo: &mut GlobalUBO, queue: &wgpu::Queue, camera: CameraUniform) {
    ubo.update_content(queue, GlobalUBOContent { camera });
}

pub struct GlobalBindings {
    bind_group_layout: BindGroupLayoutWithDesc,
    bind_group: Option<wgpu::BindGroup>,
}

impl GlobalBindings {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = BindGroupLayoutBuilder::new()
            .next_binding_rendering(binding_types::uniform())
            .create(&device, "Globals Bind Group");

        GlobalBindings {
            bind_group_layout,
            bind_group: None,
        }
    }

    pub fn create_bind_group(&mut self, device: &wgpu::Device, ubo: &GlobalUBO) {
        self.bind_group = Some(
            BindGroupBuilder::new(&self.bind_group_layout)
                .resource(ubo.binding_resource())
                .create(&device, "Global Bind Group"),
        );
    }

    pub fn bind_group_layouts(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout.layout
    }

    pub fn bind_groups(&self) -> &wgpu::BindGroup {
        &self
            .bind_group
            .as_ref()
            .expect("Bind group has not been created yet!")
    }
}
