use std::marker::PhantomData;

pub struct UniformBuffer<Content> {
    buffer: wgpu::Buffer,
    content_type: PhantomData<Content>, // basically stops the buffer outliving its content type i think
    previous_content: Vec<u8>,
}

impl<Content: bytemuck::Pod> UniformBuffer<Content> {
    // literally just pick the name out of <Content> but is likely something::Content so split by :
    fn name() -> &'static str {
        let type_name = std::any::type_name::<Content>();
        let pos = type_name.rfind(':').unwrap();
        &type_name[(pos + 1)..]
    }

    /// Create a new wgpu buffer that is a uniform, using the type Content we passed in so can be anything really
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("UniformBuffer: {}", Self::name())),
            size: std::mem::size_of::<Content>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        UniformBuffer {
            buffer,
            content_type: PhantomData,
            previous_content: Vec::new(),
        }
    }

    /// Similar to new, but we instantiate the buffer with data
    pub fn new_with_data(device: &wgpu::Device, initial_content: &Content) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("UniformBuffer: {}", Self::name())),
            size: std::mem::size_of::<Content>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });

        let mapped_memory = buffer.slice(..);
        mapped_memory
            .get_mapped_range_mut()
            .clone_from_slice(bytemuck::bytes_of(initial_content));
        buffer.unmap();

        UniformBuffer {
            buffer,
            content_type: PhantomData,
            previous_content: bytemuck::bytes_of(initial_content).to_vec(),
        }
    }

    /// Updates the content of the uniform buffer with some generic Content
    pub fn update_content(&mut self, queue: &wgpu::Queue, content: Content) {
        let new_content = bytemuck::bytes_of(&content);
        if self.previous_content == new_content {
            // this helps prevent unnesesary data transfer to the gpu which speeds things up when we have lots of data to send in a frame
            return;
        }
        // Could do partial updates since we know the previous state.
        queue.write_buffer(&self.buffer, 0, new_content);
        self.previous_content = new_content.to_vec();
    }

    /// Return the binding resource, usually to be passed to the gpu through the command queue
    pub fn binding_resource(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }
}
