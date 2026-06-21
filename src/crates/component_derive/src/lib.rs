use proc_macro::TokenStream;
use quote::{quote, ToTokens};

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
        syn::Data::Struct(ref data) => {
            data.fields.iter()
                .filter_map(|field| {
                    Some(format!("crate::PropertyDescriptor {{ name: \"{}\", description: None, data_type: {} }}", field.ident.clone().unwrap(), field.ty.to_token_stream()))
                })
                .collect::<Vec<String>>().join(",\n")
        },
        _ => { String::new() }
    };

    eprintln!("{:}", variables);

    let generated = quote! {
        impl Component for #name {
            fn name(&self) -> &'static str {
                "#name"
            }

            fn type_id(&self) -> std::any::TypeId {
                std::any::TypeId::of::<Self>()
            }

            fn properties(&self) -> Vec<&PropertyDescriptor> {
                vec![

                ]
            }
        }
    };
    generated.into()
}