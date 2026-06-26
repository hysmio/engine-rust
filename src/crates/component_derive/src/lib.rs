use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::Ident;

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

    let propertyDescriptor = Ident::new("PropertyDescriptor", Span::call_site());

    let variables = match ast.data {
        syn::Data::Struct(ref data) => data
            .fields
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = field_name.to_string();
                let ty = &field.ty;
                quote! {
                    PropertyDescriptor {
                        name: String::from(#field_name_str),
                        description: None,
                        data_type: <#ty as IntoPropertyType>::PROPERTY_TYPE,
                    }
                }
            })
            .collect::<Vec<_>>(),
        _ => vec![],
    };

    let generated = quote! {
        impl Component for #name {
            fn name(&self) -> &'static str {
                "#name"
            }

            fn type_id(&self) -> std::any::TypeId {
                std::any::TypeId::of::<Self>()
            }

            fn properties(&self) -> Vec<PropertyDescriptor> {
                vec![
                    #(#variables),*
                ]
            }
        }
    };
    eprintln!("{}", generated.to_string());
    generated.into()
}
