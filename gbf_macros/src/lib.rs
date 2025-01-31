use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Path};

/// Procedural macro to generate `From` implementations for the annotated type to its specified parent types.
#[proc_macro_derive(AstNodeTransform, attributes(convert_to))]
pub fn ast_node_transform(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let data = &input.data;

    // Parse the `#[convert_to(...)]` attribute
    let convert_to_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("convert_to"))
        .expect("Missing #[convert_to(...)] attribute for AstNodeTransform");

    let target_variants = parse_convert_to_attribute(convert_to_attr)
        .expect("Failed to parse #[convert_to(...)] attribute");

    // Generate `From` implementations
    let mut impls = Vec::new();

    for (to_ty, variant) in target_variants {
        match data {
            Data::Struct(_) => {
                // P<Source> -> P<Target>
                impls.push(quote! {
                    impl From<P<#name>> for P<#to_ty> {
                        fn from(id: P<#name>) -> Self {
                            P::from(#variant(id.into()))
                        }
                    }
                });

                // P<Source> -> Target
                impls.push(quote! {
                    impl From<P<#name>> for #to_ty {
                        fn from(id: P<#name>) -> Self {
                            #variant(id.into())
                        }
                    }
                });

                // Source -> P<Target>
                impls.push(quote! {
                    impl From<#name> for P<#to_ty> {
                        fn from(id: #name) -> Self {
                            P::from(#variant(id.into()))
                        }
                    }
                });

                // Source -> Target
                impls.push(quote! {
                    impl From<#name> for #to_ty {
                        fn from(id: #name) -> Self {
                            #variant(id.into())
                        }
                    }
                });
            }

            // Handle enums
            Data::Enum(_) => {
                // P<Source> -> P<Target>
                // impls.push(quote! {
                //     impl From<P<#name>> for P<#to_ty> {
                //         fn from(id: P<#name>) -> Self {
                //             P::from(#variant(id.into()))
                //         }
                //     }
                // });

                // // Source -> P<Target>
                // impls.push(quote! {
                //     impl From<P<#name>> for #to_ty {
                //         fn from(id: P<#name>) -> Self {
                //             #variant(id.into())
                //         }
                //     }
                // });

                // // P<Source> -> Target
                // impls.push(quote! {
                //     impl From<#name> for P<#to_ty> {
                //         fn from(id: #name) -> Self {
                //             P::from(#variant(id.into()))
                //         }
                //     }
                // });

                // Source -> Target
                impls.push(quote! {
                    impl From<#name> for #to_ty {
                        fn from(id: #name) -> Self {
                            #variant(id.into())
                        }
                    }
                });
            }

            _ => panic!("AstNodeTransform can only be applied to enums or structs"),
        }
    }

    let output = quote! {
        #(#impls)*
    };

    output.into()
}

/// Parses the `#[convert_to(...)]` attribute and extracts the list of target types and their variants.
fn parse_convert_to_attribute(attr: &Attribute) -> Result<Vec<(Path, Path)>, syn::Error> {
    attr.parse_args_with(|input: syn::parse::ParseStream| {
        let mut target_variants = Vec::new();

        while !input.is_empty() {
            // Parse the enum type and variant (e.g., ExprKind::Assignable)
            let variant: Path = input.parse()?;
            let parent = Path::from(variant.segments.first().unwrap().ident.clone());
            target_variants.push((parent, variant));

            // If there's a comma, consume it and continue
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(target_variants)
    })
}
