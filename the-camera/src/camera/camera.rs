use cgmath::{Matrix4, SquareMatrix};

pub trait Camera: Sized {
    fn build_view_projection_matrix(&self) -> Matrix4<f32>;
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct CameraUniform {
    /// The eye position of the camera in homogenous coordinates.
    ///
    /// Homogenous coordinates are used to fullfill the 16 byte alignment requirement.
    pub view_position: [f32; 4],

    /// Contains the view projection matrix.
    pub view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    /// Creates a default [CameraUniform].
    fn default() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: convert_matrix4_to_array(Matrix4::identity()),
        }
    }
}

pub fn convert_matrix4_to_array(matrix4: Matrix4<f32>) -> [[f32; 4]; 4] {
    let mut result = [[0.0; 4]; 4];

    for i in 0..4 {
        for j in 0..4 {
            result[i][j] = matrix4[i][j];
        }
    }

    result
}
