use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{Ident, Visibility};

#[proc_macro_derive(Component, attributes(property))]
pub fn component_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate.
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation.
    impl_component_macro(&ast)
}

fn impl_component_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let variables = match ast.data {
        syn::Data::Struct(ref data) => data
            .fields
            .iter()
            .filter_map(|field| {
                if field.vis == Visibility::Inherited {
                    return None;
                }

                if let Some(_) = field.attrs.iter().find(|attr| {
                    eprintln!("attr: {:#?}", attr);
                    match &attr.meta {
                        syn::Meta::List(meta) => {
                            meta.path.to_token_stream().to_string() == "property"
                                && meta
                                    .tokens
                                    .clone()
                                    .into_iter()
                                    .find(|token| token.to_string() == "hidden")
                                    .is_some()
                        }
                        _ => false,
                    }
                }) {
                    return None;
                }

                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = field_name.to_string();
                let ty = &field.ty;
                Some(quote! {
                    PropertyDescriptor {
                        name: String::from(#field_name_str),
                        description: None,
                        data_type: <#ty as IntoPropertyType>::PROPERTY_TYPE,
                    }
                })
            })
            .collect::<Vec<_>>(),
        _ => vec![],
    };

    let generated = quote! {
        impl Component for #name {
            fn name(&self) -> &'static str {
                "#name"
            }

            fn as_any(&self) -> &dyn std::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

            fn properties(&self) -> Vec<PropertyDescriptor> {
                vec![
                    #(#variables),*
                ]
            }
        }
    };

    generated.into()
}
