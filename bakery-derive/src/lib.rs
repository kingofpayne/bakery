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

    let expanded = quote! {
        impl #impl_generics bakery::Recipe for #name #ty_generics #where_clause {
            fn recipe(tree: &mut bakery::NodeTree) -> u32 {
                let nid = tree.create_struct(None, "S");
                #fields
                nid
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
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}
