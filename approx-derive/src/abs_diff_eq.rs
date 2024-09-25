use darling::{ast::Generics, FromMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr, Path};

#[derive(FromMeta)]
#[darling(allow_unknown_fields)]
struct Opts {
    epsilon: Path,
    absolute: Expr,
}

pub(crate) fn derive(item: DeriveInput) -> syn::Result<TokenStream> {
    let (
        super::Opts {
            value: Opts { epsilon, absolute },
            generics: Generics { where_clause, .. },
            data,
        },
        params,
    ) = super::Opts::parse(&item)?;

    let ident = item.ident;
    let comparisons = super::comparisons(data, "abs_diff_eq", None);

    Ok(quote! {
        #[automatically_derived]
        impl<#(#params)*> AbsDiffEq for #ident <#(#params)*> #where_clause {
            type Epsilon = #epsilon;

            fn default_epsilon() -> Self::Epsilon {
                #absolute
            }

            fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
                #comparisons
            }
        }
    })
}
