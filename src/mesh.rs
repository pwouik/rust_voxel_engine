#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Face {
    pub pos_dir: [u8; 4],
    pub texture: u32,
    pub light: [u8; 4],
}
pub struct Mesh {
    pub storage_buffer: wgpu::Buffer,
    pub bind_group: Option<wgpu::BindGroup>,
    pub num_elements: u32,
}
impl Mesh {
    pub fn destroy(&self) {
        self.storage_buffer.destroy();
    }
}
