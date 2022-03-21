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

#[proc_macro_derive(VertexLayout, attributes(layout))]
pub fn vertex_layout(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let step_mode = input
    .attrs
    .into_iter()
    .find_map(|attr| {
      if attr.path.get_ident().unwrap().to_string() == "layout" {
        match attr.parse_meta().unwrap() {
          Meta::List(list) => match list.nested.into_iter().next().unwrap() {
            NestedMeta::Meta(meta) => match meta {
              Meta::Path(path) => {
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
          },
          _ => panic!("Invalid value"),
        }
      } else {
        None
      }
    })
    .unwrap_or(format_ident!("Vertex"));

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

  let vertices = fields.into_iter().enumerate().map(|(n, field)| {
    let span = field.span().clone();
    let attr = field
      .attrs
      .into_iter()
      .find_map(|attr| {
        if attr.path.get_ident().unwrap().to_string() == "layout" {
          match attr.parse_meta().unwrap() {
            Meta::List(list) => match list.nested.into_iter().next().unwrap() {
              NestedMeta::Meta(meta) => match meta {
                Meta::Path(path) => {
                  let ident = path.get_ident().unwrap();
                  if ident.to_string() == "norm" {
                    Some(Attr::Norm(true))
                  } else {
                    Some(Attr::Override(ident.clone()))
                  }
                }
                _ => panic!("Invalid value"),
              },
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

    let n = n as u32;
    quote!(#n => #ident)
  });

  let tokens = quote! {
    impl #name {
      pub const LAYOUT: wgpu::VertexBufferLayout = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<#name>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::#step_mode,
        attributes: &wgpu::vertex_attr_array![#(#vertices),*],
      };
    }
  };

  tokens.into()
}
