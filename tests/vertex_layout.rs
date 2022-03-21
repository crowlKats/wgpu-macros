use wgpu_macros::VertexLayout;

#[repr(C)]
#[derive(VertexLayout)]
struct Vertex {
  #[layout(norm)]
  a: [u8; 2],
  #[layout(Float64)]
  b: [f32; 2],
  c: [u16; 2],
  d: f64,
}

#[test]
fn test() {
  assert_eq!(
    Vertex::LAYOUT,
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          format: wgpu::VertexFormat::Unorm8x2,
          offset: 0,
          shader_location: 0,
        },
        wgpu::VertexAttribute {
          format: wgpu::VertexFormat::Float64,
          offset: 0 + wgpu::VertexFormat::Unorm8x2.size(),
          shader_location: 1,
        },
        wgpu::VertexAttribute {
          format: wgpu::VertexFormat::Uint16x2,
          offset: 0
            + wgpu::VertexFormat::Unorm8x2.size()
            + wgpu::VertexFormat::Float64.size(),
          shader_location: 2,
        },
        wgpu::VertexAttribute {
          format: wgpu::VertexFormat::Float64,
          offset: 0
            + wgpu::VertexFormat::Unorm8x2.size()
            + wgpu::VertexFormat::Float64.size()
            + wgpu::VertexFormat::Uint16x2.size(),
          shader_location: 3,
        },
      ],
    }
  );
}
