extern crate proc_macro2;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Fields, GenericParam,
    Generics,
};

/// Implements bakery::Recipe trait for the derived type
#[proc_macro_derive(Recipe)]
pub fn derive_bakery(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // This code is derived from the heapsize_derive example from the syn crate:
    // https://github.com/dtolnay/syn/blob/master/examples/heapsize/heapsize_derive/src/lib.rs
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    // add_trait_bounds will mark generic types to impl the trait `bakery::Recipe`
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = recipe_token_stream(&input.data);

    let implementation = match input.data {
        Data::Struct(_) => {
            quote! {
                let nid = tree.create_struct(None, "S");
                #fields
                nid
            }
        }
        Data::Enum(_) => {
            quote! {
                let nid_storage_ty = i32::recipe(tree);
                let nid = tree.create_enum(None, "E", nid_storage_ty);
                let mut next_enum_value = 0;
                #fields
                nid
            }
        }
        Data::Union(_) => unimplemented!(),
    };

    let expanded = quote! {
        impl #impl_generics bakery::Recipe for #name #ty_generics #where_clause {
            fn recipe(tree: &mut bakery::NodeTree) -> u32 {
                #implementation
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

// Add a bound `T: bakery::Recipe` to every type parameter T
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(bakery::Recipe));
        }
    }
    generics
}

fn recipe_token_stream(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let quotes = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        let ty = &f.ty;
                        quote_spanned! {
                            f.span() =>
                                // To comply with borrow checker, those two lines cannot be merged.
                                let nid_ty = <#ty> :: recipe(tree);
                                tree.create_struct_member(nid, stringify!(#name), nid_ty);
                        }
                    });
                    quote! {
                        #( #quotes )*
                    }
                }
                Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
            }
        }
        Data::Enum(ref data) => {
            let quotes_variant = data.variants.iter().map(|variant| {
                let name = &    variant.ident;
                let quote_fields = match variant.fields {
                    Fields::Named(ref fields) => {
                        // This enumeration has named values
                        // We create a `RecStruct` node as a child of a `RecEnumItem` node.
                        let quotes_fields = fields.named.iter().map(|f| {
                            let name = &f.ident;
                            let ty = &f.ty;
                            quote! {
                                let nid_ty = <#ty> :: recipe(tree);
                                tree.create_struct_member(nid_struct, stringify!(#name), nid_ty);
                            }
                        });
                        quote! {
                            let nid_struct = tree.create_struct(Some(nid_variant), "");
                            #( #quotes_fields )*
                        }
                    }
                    Fields::Unnamed(ref fields) => {
                        // This enumeration has tuple data
                        // We create a `RecTuple` node as a child of a `RecEnumItem` node.
                        let quotes_fields = fields.unnamed.iter().map(|f| {
                            let ty = &f.ty;
                            quote! {
                                let nid_ty = <#ty> :: recipe(tree);
                                tree.create_tuple_member(nid_tuple, nid_ty);
                            }
                        });
                        quote! {
                            let nid_tuple = tree.create_tuple(Some(nid_variant));
                            #( #quotes_fields )*
                        }
                    }
                    Fields::Unit => quote!{}
                };
                quote! {
                    let nid_variant = tree.create_enum_member(nid, stringify!(#name), next_enum_value.into());
                    #quote_fields
                }
            });
            quote! {
                #( #quotes_variant )*
            }
        }
        Data::Union(_) => unimplemented!(),
    }
}
