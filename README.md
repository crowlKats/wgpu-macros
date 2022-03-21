# wgpu-macros
A set of useful proc macros for [wgpu](https://github.com/gfx-rs/wgpu/).

## VertexLayout derive macro
Generates a [`wgpu::VertexBufferLayout`](https://docs.rs/wgpu/latest/wgpu/struct.VertexBufferLayout.html),
accessible through a `LAYOUT` constant on the struct.

There is an additional `layout` helper attribute macro that can be specified on 
the struct, which allows specifying the [`step_mode`](https://docs.rs/wgpu/latest/wgpu/struct.VertexBufferLayout.html#structfield.step_mode).
Possible values are `Vertex` & `Instance`, and by default, the value is `Vertex`.

The `layout` helper attribute macro also can be used on individual fields to 
either override the generated [`VertexFormat`][VertexFormat] by specifying a 
[`VertexFormat`][VertexFormat] variant, or to specify if the `norm` version of 
the generated [`VertexFormat`][VertexFormat] should be used (so `Uint8` becomes `Unorm8`).

[VertexFormat]: https://docs.rs/wgpu/latest/wgpu/enum.VertexFormat.html