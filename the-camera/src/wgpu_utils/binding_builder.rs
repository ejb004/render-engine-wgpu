/// Struct representing a bind group layput continaing a [wgpu::BindGroupLayout] and an associated [Vec<wgpu::BindGroupLayoutEntry>]
pub struct BindGroupLayoutWithDesc {
    pub layout: wgpu::BindGroupLayout,
    pub entries: Vec<wgpu::BindGroupLayoutEntry>,
}

/// Tool to create bind group layouts
pub struct BindGroupLayoutBuilder {
    /// Each layout builder will have n number of entries, stored in a [Vec]. Each entry is a [wgpu::BindGroupLayoutEntry].
    /// we will also need to increment the bind index with each layout we add. We would find the binding index/group here in the
    /// shader: " **@group(0) @binding(0)** "
    entries: Vec<wgpu::BindGroupLayoutEntry>,
    next_binding_index: u32,
}

impl BindGroupLayoutBuilder {
    /// constructor function
    pub fn new() -> Self {
        BindGroupLayoutBuilder {
            entries: Vec::new(),
            next_binding_index: 0,
        }
    }

    /// Add a binding to the BindGroupLayoutBuilder instance
    // Function takes ownership of the struct instance with mut self (not &mut) then returns the struct again after it has been mutated to the rest of the program
    pub fn add_binding(mut self, binding: wgpu::BindGroupLayoutEntry) -> Self {
        self.next_binding_index += 1;
        self.entries.push(binding);
        self
    }

    /// Create a [wgpu::BindGroupLayoutEntry] and adds it to the [BindGroupLayoutBuilder]
    pub fn next_binding(
        self,
        visibility: wgpu::ShaderStages,
        bind_type: wgpu::BindingType,
    ) -> Self {
        let binding = self.next_binding_index;
        self.add_binding(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: bind_type,
            count: None,
        })
    }

    /// Calls next_binding function with the specification that the binding should be visible to only a fragment shader
    pub fn next_binding_fragment(self, ty: wgpu::BindingType) -> Self {
        self.next_binding(wgpu::ShaderStages::FRAGMENT, ty)
    }

    /// Calls next_binding function with the specification that the binding should be visible to only a vertex shader
    pub fn next_binding_vertex(self, ty: wgpu::BindingType) -> Self {
        self.next_binding(wgpu::ShaderStages::VERTEX, ty)
    }

    /// Calls next_binding function with the specification that the binding should be visible to only a fragment and vertex shaders
    pub fn next_binding_rendering(self, ty: wgpu::BindingType) -> Self {
        self.next_binding(
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty,
        )
    }

    /// Calls next_binding function with the specification that the binding should be visible to only a compute shader
    pub fn next_binding_compute(self, ty: wgpu::BindingType) -> Self {
        self.next_binding(wgpu::ShaderStages::COMPUTE, ty)
    }

    /// Calls next_binding function with the specification that the binding should be visible to all shaders
    pub fn next_binding_all(self, ty: wgpu::BindingType) -> Self {
        self.next_binding(
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
            ty,
        )
    }

    /// Creates a bind group layout with a description/label passed in for debugging and identification
    pub fn create(self, device: &wgpu::Device, label: &str) -> BindGroupLayoutWithDesc {
        BindGroupLayoutWithDesc {
            layout: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &self.entries,
                label: Some(label),
            }),
            entries: self.entries,
        }
    }
}

/// Builder for wgpu::BindGroups following the exact layout from a wgpu::BindGroupLayout
// Makes life simpler by assuming that order of elements in the bind group is equal to order of elements in the bind group layout.
pub struct BindGroupBuilder<'a> {
    layout_with_desc: &'a BindGroupLayoutWithDesc,
    entries: Vec<wgpu::BindGroupEntry<'a>>,
}

impl<'a> BindGroupBuilder<'a> {
    /// Constructor
    pub fn new(layout_with_desc: &'a BindGroupLayoutWithDesc) -> Self {
        BindGroupBuilder {
            layout_with_desc,
            entries: Vec::new(),
        }
    }

    // Uses same binding index as binding group layout at the same ordering
    pub fn resource(mut self, resource: wgpu::BindingResource<'a>) -> Self {
        // assert_eq!(self.entries.len(), self.layout_with_desc.entries.len());
        self.entries.push(wgpu::BindGroupEntry {
            binding: self.layout_with_desc.entries[self.entries.len()].binding,
            resource,
        });
        self
    }
    pub fn buffer(self, buffer_binding: &'a wgpu::Buffer) -> Self {
        self.resource(buffer_binding.as_entire_binding())
    }
    pub fn sampler(self, sampler: &'a wgpu::Sampler) -> Self {
        self.resource(wgpu::BindingResource::Sampler(sampler))
    }
    pub fn texture(self, texture_view: &'a wgpu::TextureView) -> Self {
        self.resource(wgpu::BindingResource::TextureView(texture_view))
    }

    /// Creates a Bind group with the given label and layouts+entries stored by the Builder
    pub fn create(&self, device: &wgpu::Device, label: &str) -> wgpu::BindGroup {
        assert_eq!(self.entries.len(), self.layout_with_desc.entries.len());
        let descriptor = wgpu::BindGroupDescriptor {
            layout: &self.layout_with_desc.layout,
            entries: &self.entries,
            label: Some(label),
        };
        device.create_bind_group(&descriptor)
    }
}
