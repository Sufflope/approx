mod abs_diff_eq;
mod relative_eq;

use darling::{
    ast::{Data, Fields, GenericParam, GenericParamExt, Generics, Style},
    util::Flag,
    FromDeriveInput, FromField, FromMeta, FromVariant,
};
use itertools::Itertools;
use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Ident};

#[derive(FromDeriveInput)]
#[darling(attributes(approx))]
struct Opts<T> {
    generics: Generics<GenericParam<Ident>>,
    data: Data<Variant, Field>,
    #[darling(flatten)]
    value: T,
}

impl<T> Opts<T> {
    fn parse(input: &DeriveInput) -> darling::Result<(Self, Vec<Ident>)>
    where
        T: FromMeta,
    {
        Self::from_derive_input(input).map(|opts| {
            let params = opts
                .generics
                .params
                .iter()
                .map(|param| param.as_type_param().cloned().unwrap())
                .collect();
            (opts, params)
        })
    }
}

#[derive(FromVariant)]
#[darling(attributes(approx))]
struct Variant {
    ident: Ident,
    fields: Fields<Field>,
}

#[derive(FromField)]
#[darling(attributes(approx), and_then = Self::validate)]
struct Field {
    ident: Option<Ident>,
    skip: Flag,
    approximate: Flag,
}

impl Field {
    fn validate(self) -> darling::Result<Self> {
        if self.skip.is_present() && self.approximate.is_present() {
            Err(
                darling::Error::custom("Cannot both skip and use approximate equality")
                    .with_span(&self.approximate.span()),
            )
        } else {
            Ok(self)
        }
    }
}

#[proc_macro_derive(AbsDiffEq, attributes(approx))]
pub fn abs_diff_eq(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    convert(abs_diff_eq::derive(item))
}

#[proc_macro_derive(RelativeEq, attributes(approx))]
pub fn relative_eq(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    convert(relative_eq::derive(item))
}

fn convert(tokens: syn::Result<TokenStream>) -> proc_macro::TokenStream {
    tokens.unwrap_or_else(syn::Error::into_compile_error).into()
}

fn comparisons(data: Data<Variant, Field>, f: &str, arg: Option<&str>) -> TokenStream {
    let comparisons = data
        .map_enum_variants(|variant| {
            let ident = variant.ident;
            let fields = variant.fields;
            match fields.style {
                Style::Tuple => {
                    let (comps, self_extractors, other_extractors): (Vec<_>, Vec<_>, Vec<_>) =
                        fields
                            .fields
                            .into_iter()
                            .enumerate()
                            .map(|(i, field)| {
                                if field.skip.is_present() {
                                    (None, format_ident!("_"), format_ident!("_"))
                                } else {
                                    let one = format_ident!("_{}", i);
                                    let other = format_ident!("other_{}", i);
                                    (
                                        Some(compare(&one, &other, field.approximate, f, arg)),
                                        one,
                                        other,
                                    )
                                }
                            })
                            .multiunzip();
                    let comps = comps.iter().flatten();
                    quote! {
                        Self::#ident(#(#self_extractors),*) => match other {
                            Self::#ident(#(#other_extractors),*) => #(#comps)&&*,
                            _ => false
                        }
                    }
                }
                Style::Struct => {
                    let (comps, self_extractors, other_extractors): (Vec<_>, Vec<_>, Vec<_>) =
                        fields
                            .fields
                            .into_iter()
                            .filter_map(|field| {
                                if field.skip.is_present() {
                                    None
                                } else {
                                    let one = field.ident.clone().unwrap();
                                    let other = format_ident!("other_{}", one);
                                    Some((
                                        compare(&one, &other, field.approximate, f, arg),
                                        one.clone(),
                                        quote! { #one: #other },
                                    ))
                                }
                            })
                            .multiunzip();
                    quote! {
                        Self::#ident { #(#self_extractors),*, .. } => match other {
                            Self::#ident {#(#other_extractors),*, ..} => #(#comps)&&*,
                            _ => false
                        }
                    }
                }
                Style::Unit => quote! { Self::#ident => self == other },
            }
        })
        .map_struct(|fields| {
            Fields::<TokenStream>::from((
                fields.style,
                fields
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, field)| {
                        if field.skip.is_present() {
                            None
                        } else {
                            let ident = match field.ident {
                                None => Literal::usize_unsuffixed(i).to_token_stream(),
                                Some(ident) => quote! { #ident },
                            };
                            Some(compare(
                                quote! { self.#ident },
                                quote! { other.#ident },
                                field.approximate,
                                f,
                                arg,
                            ))
                        }
                    })
                    .collect::<Vec<_>>(),
            ))
        });

    if comparisons.is_enum() {
        let comparisons = comparisons.take_enum().unwrap();
        quote! {
            match self {
                #(#comparisons),*
            }
        }
    } else {
        let comparisons = comparisons.take_struct().unwrap().fields.into_iter();
        quote!(#(#comparisons)&&*)
    }
}

fn compare<One, Other>(
    one: One,
    other: Other,
    approximate: Flag,
    f: &str,
    arg: Option<&str>,
) -> TokenStream
where
    One: ToTokens,
    Other: ToTokens,
{
    if approximate.is_present() {
        let q = |s| Ident::new(s, Span::call_site());
        let arg = arg.map(|arg| {
            let arg = q(arg);
            quote! {, #arg}
        });
        let f = q(f);
        quote! { #one.#f(&#other, epsilon #arg) }
    } else {
        quote! { #one == #other }
    }
}
