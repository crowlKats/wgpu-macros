extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Ident;

use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{
  parse_macro_input, Data, DeriveInput, Expr, Fields, Lit, Meta, NestedMeta,
  Type,
};

enum Attr {
  Override(Ident),
  Norm(bool),
}

/// Allows for generation of a `wgpu::VertexBufferLayout`, which can be accessed
/// by the `LAYOUT` constant on the macro.
///
/// ```
/// # use wgpu;
/// # use wgpu_macros::VertexLayout;
/// # #[repr(C)]
/// #[derive(VertexLayout)]
/// struct Vertex {
///   position: [f32; 3],
///   tex_coords: [f32; 2],
/// }
///
/// fn main() {
///   Vertex::LAYOUT; // use in a RenderPipelineDescriptor
/// }
/// ```
///
/// # Changing `step_mode`
/// By default, the `step_mode` is set to `Vertex`.
/// To change the `step_mode` for the `VertexBufferLayout`, you can declare the
/// `layout` attribute macro for the struct, and passing one of the variants
/// of `wgpu::VertexStepMode`.
///
/// ```
/// # use wgpu_macros::VertexLayout;
/// # #[repr(C)]
/// #[derive(VertexLayout)]
/// #[layout(Instance)]
/// struct Vertex {}
/// ```
///
/// # Specifying shader `location`
///
/// By default the shader `location` starts at `0` and increments by `1` for each field.
///
/// However you can manually specify the `location` for a field:
///
/// ```
/// # use wgpu_macros::VertexLayout;
/// # #[repr(C)]
/// #[derive(VertexLayout)]
/// struct Vertex {
///   #[layout(location = 3)]
///   position: [f32; 3],
///
///   tex_coords: [u8; 2],
/// }
/// ```
///
/// So the `position` will be at shader location `3`.
///
/// If a field doesn't have a `location` it will increment the previous field's location by `1`.
///
/// Because `tex_coords` doesn't have a `location` it will have a `location` of `4`.
///
/// # Using `norm` Variants
///
/// By specifying `norm` the `layout` attribute macro for the field you want,
/// it will use the `norm` variant corresponding to the field value.
///
/// ```
/// # use wgpu_macros::VertexLayout;
/// # #[repr(C)]
/// #[derive(VertexLayout)]
/// struct Vertex {
///   # position: [f32; 3],
///   #[layout(norm)]
///   tex_coords: [u8; 2],
/// }
/// ```
///
/// So `Uint8x2` becomes `Unorm8x2`.
///
/// # Overriding Generated `VertexFormat`
///
/// By specifying the wanted `VertexFormat` in the `layout` attribute macro for
/// the field you want, you can override the generated `VertexFormat`.
///
/// ```
/// # use wgpu_macros::VertexLayout;
/// # #[repr(C)]
/// #[derive(VertexLayout)]
/// struct Vertex {
///   # position: [f32; 3],
///   #[layout(Uint16x4)]
///   tex_coords: [f32; 2],
/// }
/// ```
#[proc_macro_derive(VertexLayout, attributes(layout))]
pub fn vertex_layout(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let step_mode = input
    .attrs
    .into_iter()
    .find_map(|attr| {
      if *attr.path.get_ident().unwrap() == "layout" {
        match attr.parse_meta().unwrap() {
          Meta::List(list) => match list.nested.into_iter().next().unwrap() {
            NestedMeta::Meta(Meta::Path(path)) => {
              let ident = path.get_ident().unwrap();
              if matches!(ident.to_string().as_ref(), "Vertex" | "Instance") {
                Some(ident.clone())
              } else {
                panic!("Invalid value")
              }
            }
            _ => panic!("Invalid value"),
          },
          _ => panic!("Invalid value"),
        }
      } else {
        None
      }
    })
    .unwrap_or_else(|| format_ident!("Vertex"));

  let name = input.ident;

  let data = match input.data {
    Data::Struct(data) => data,
    _ => panic!("Only structs can derive VertexLayout"),
  };
  let fields = match data.fields {
    Fields::Named(fields) => fields.named,
    Fields::Unnamed(fields) => fields.unnamed,
    Fields::Unit => panic!("Unit structs arent allowed for VertexLayout"),
  };

  let mut location: u32 = 0;

  let vertices = fields.into_iter().map(|field| {
    let span = field.span();
    let attr = field
      .attrs
      .into_iter()
      .find_map(|attr| {
        if *attr.path.get_ident().unwrap() == "layout" {
          match attr.parse_meta().unwrap() {
            Meta::List(list) => match list.nested.into_iter().next().unwrap() {
              NestedMeta::Meta(Meta::Path(path)) => {
                let ident = path.get_ident().unwrap();
                if *ident == "norm" {
                  Some(Attr::Norm(true))
                } else {
                  Some(Attr::Override(ident.clone()))
                }
              }
              NestedMeta::Meta(Meta::NameValue(name_value)) => {
                let ident = name_value.path.get_ident().unwrap();
                if matches!(ident.to_string().as_ref(), "location") {
                  match name_value.lit {
                    Lit::Int(lit) => {
                      location = lit.base10_parse().unwrap();
                      None
                    },
                    _ => panic!("Invalid value"),
                  }
                } else {
                  panic!("Invalid value")
                }
              }
              _ => panic!("Invalid value"),
            },
            _ => panic!("Invalid value"),
          }
        } else {
          None
        }
      })
      .unwrap_or(Attr::Norm(false));

    let ident = match attr {
      Attr::Override(ident) => ident,
      Attr::Norm(norm) => {
        let (ty, len) = match field.ty {
          Type::Array(array) => {
            let len = match array.len {
              Expr::Lit(lit) => match lit.lit {
                Lit::Int(int) => int,
                _ => unreachable!(),
              },
              _ => unreachable!(),
            };
            let ty = match *array.elem {
              Type::Path(p) => p.path,
              _ => unreachable!(),
            };

            (ty, len.base10_parse::<usize>().unwrap())
          }
          Type::Path(path) => (path.path, 1),
          ty => panic!("Type '{:?}' isnt allowed for VertexLayout", ty),
        };
        let ty = ty.segments.into_iter().last().unwrap().ident.to_string();

        let full_type = match (ty.as_ref(), norm) {
          ("u8", false) => "Uint8",
          ("u8", true) => "Unorm8",
          ("i8", false) => "Sint8",
          ("i8", true) => "Snorm8",

          ("u16", false) => "Uint16",
          ("u16", true) => "Unorm16",
          ("i16", false) => "Sint16",
          ("i16", true) => "Snorm16",

          ("f32", false) => "Float32",
          ("u32", false) => "Uint32",
          ("i32", false) => "Sint32",

          ("f64", false) => "Float64",
          (ty, true) => panic!("Type '{ty}' cannot be normalized"),
          (ty, _) => panic!("Type '{ty}' is not allowed"),
        };

        match (full_type, len) {
          ("Uint8", 2 | 4) => {}
          ("Unorm8", 2 | 4) => {}
          ("Sint8", 2 | 4) => {}
          ("Snorm8", 2 | 4) => {}
          ("Uint16", 2 | 4) => {}
          ("Unorm16", 2 | 4) => {}
          ("Sint16", 2 | 4) => {}
          ("Snorm16", 2 | 4) => {}
          ("Float32", 1 | 2 | 3 | 4) => {}
          ("Uint32", 1 | 2 | 3 | 4) => {}
          ("Sint32", 1 | 2 | 3 | 4) => {}
          ("Float64", 1 | 2 | 3 | 4) => {}
          (_, len) => panic!("Type '{ty}' cannot be used {len} times"),
        }

        if len == 1 {
          quote::format_ident!("{full_type}", span = span)
        } else {
          quote::format_ident!("{full_type}x{}", len.to_string(), span = span)
        }
      }
    };

    let output = quote!(#location => #ident);

    location += 1;

    output
  });

  let tokens = quote! {
    impl #name {
      pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<#name>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::#step_mode,
        attributes: &wgpu::vertex_attr_array![#(#vertices),*],
      };
    }
  };

  tokens.into()
}
