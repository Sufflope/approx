use darling::{ast::Generics, FromMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr};

#[derive(FromMeta)]
#[darling(allow_unknown_fields)]
struct Opts {
    relative: Expr,
}

pub(crate) fn derive(item: DeriveInput) -> syn::Result<TokenStream> {
    let (
        super::Opts {
            value: Opts { relative },
            generics: Generics { where_clause, .. },
            data,
        },
        params,
    ) = super::Opts::parse(&item)?;

    let ident = item.ident;
    let comparisons = super::comparisons(data, "relative_eq", Some("max_relative"));

    Ok(quote! {
        #[automatically_derived]
        impl<#(#params)*> RelativeEq for #ident <#(#params)*> #where_clause {
            fn default_max_relative() -> Self::Epsilon {
                #relative
            }

            fn relative_eq(&self, other: &Self, epsilon: Self::Epsilon, max_relative: Self::Epsilon) -> bool {
                #comparisons
            }
        }
    })
}
