//! Quick enum mappings to strings
//! 
//! This crate provides a derive macro `#[derive(EnumMaping)]` to quickly create mappings between enum variants and strings.
//! 
//! For example instead of writing
//! ```
//! enum Example {
//!     V1,
//!     V2,
//!     Unknown
//! }
//! 
//! impl Default for Example {
//!     fn default() -> Self {
//!         Self::Unknown
//!     }
//! }
//! 
//! impl Example {
//!     fn to_mappedstr(&self) -> &'static str {
//!         match self {
//!             Self::V1 => "variant_1",
//!             Self::V2 => "variant_2",
//!             Self::Unknown => "unknown"
//!         }
//!     }
//! 
//!     fn from_mappedstr(s: &str) -> Self {
//!         match s {
//!             s if s == "variant_1" => Self::V1,
//!             s if s == "variant_2"  => Self::V2,
//!             s if s == "unknown" => Self::Unknown,
//!             _ => Self::default()   
//!         }
//!     }
//! }
//! ```
//! you can do
//! ```
//! use enum_maping::EnumMaping;
//! 
//! #[derive(EnumMaping)]
//! enum Example {
//!     #[mapstr("variant_1")]
//!     V1,
//!     #[mapstr("variant_2")]
//!     V2,
//!     #[mapstr("unknown")]
//!     Unknown
//! 
//! }
//! 
//! impl Default for Example {
//!     fn default() -> Self {
//!         Self::Unknown
//!     }
//! }
//! ```
//! This should be especially helpful for large number of variants.

use proc_macro::{TokenStream};
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{Token, parse_macro_input, DeriveInput};


struct Variant {
    ident: syn::Ident,
    maps: Vec<(Option<String>, String)>
}

fn parse_variants(variants: &syn::punctuated::Punctuated<syn::Variant, Token![,]>) -> Vec<Variant> {
    variants.iter().filter_map(|variant| {
        let mut v = Variant { 
            ident: variant.ident.clone(),
            maps: Vec::new()
        };

        variant.attrs.iter()
            .filter_map(|attr| attr.parse_meta().ok())
            .filter_map(|meta| if let syn::Meta::List(syn::MetaList {path, nested, ..}) = meta {
                Some((path, nested))
            } else {
                None
            } 
            ).filter_map(|(path, nested)| {
                assert!(path.segments.len() == 1, "Found unexpected attribute.");
                match &path.segments[0] {
                    s if s.ident == "mapstr" => {
                        let mut fn_name = None;
                        let mut mapped_value = None;
                        for n in nested {
                            match n {
                                // Just literal, the value to map to 
                                syn::NestedMeta::Lit(syn::Lit::Str(l)) => {
                                    mapped_value = Some(l.value());
                                },
                                // Named argument, must be "fn_name"
                                syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {path, lit: syn::Lit::Str(l), ..})) => {
                                    match &path.segments[0].ident {
                                        s if s == "fname" => {
                                          fn_name = Some(l.value());  
                                        },
                                        s => panic!("Unknown mapstr named argument {:#?}.", quote!(#s))
                                    }
                                }
                                _ => {
                                    panic!("Unknown mapstr argument {:#?}.", quote!{#n})
                                }
                            }
                        }
                        if mapped_value.is_none() {
                            return None;
                        }

                        if fn_name.is_none() { // Use default fn name
                            return Some((None, mapped_value.unwrap()))
                        } else {
                            return Some((fn_name, mapped_value.unwrap()))
                        }
                    },
                    _ => {
                        return None;
                    }
                }
            }).for_each(|a| v.maps.push(a));
            if v.maps.len() > 0 {
                Some(v)
            } else {
               None
            }

        }).collect::<Vec<_>>()
}

/// Macro to derive custom mapings for enum types.
/// 
/// It provides function implementations for `to` and `from` functions for enum. 
/// Each mapping is specified on enum variant by attribute `#[mapstr(<value to map to>)]`.
/// Attribute has optional parameter `fname` which specifies function names as `to_<fname>` and `from_<fname>`.
/// This only needs to be specified on the attribute of the first variant.
/// 
/// Multiple mappings can be specified for single variant but (at least for now) all variants must specify same number of mappings.
/// If `fname` is provided, the order doesn't matter. If it is not then that `mapstr` must be at the same position as on the first variant.
/// 
/// # Variant attributes
/// * `mapstr(<value>, [fname=""])`
///     - `value` - string to map to
///     - `fname` - specify created function name as `to_<fname>` and `from_<fname>`. Optional, defaults to "mappedstr".
///     
/// 
/// # Current shortcomings
/// * Enum must implement [`Default`]  
///  Because the `from` function can take any str the `from` function will return default value if given input didn't match with the mappings.
///  In future it's planned to provide try_to/from methods which return Option/Result for unknown inputs. 
/// * Only variants without fields are supported
/// * All variants must have same number of mapings
/// * Error messages from macro are simple panics (and at places completely with wrong messages)
///  
/// # Examples
/// Simplest form with default function names
/// ```
/// use enum_maping::EnumMaping;
///
/// #[derive(EnumMaping, Debug, Eq, PartialEq)]
/// enum Example {
///     #[mapstr("variant_1")] // Use default function name "to/from_mappedstr"
///     V1,
///     #[mapstr("variant_2")]
///     V2,
///     #[mapstr("unknown")]
///     Unknown
/// 
/// }
/// 
/// impl Default for Example {
///     fn default() -> Self {
///         Self::Unknown
///     }
/// }
/// 
/// assert_eq!(Example::V1.to_mappedstr(), "variant_1");
/// assert_eq!(Example::V2.to_mappedstr(), "variant_2");
/// assert_eq!(Example::from_mappedstr("variant_1"), Example::V1);
/// assert_eq!(Example::from_mappedstr("variant_3"), Example::Unknown);
/// ```
/// This example expands to
/// ```ignore
/// impl Example {
///     fn to_mappedstr(&self) -> &'static str {
///         match self {
///             Self::V1 => "variant_1",
///             Self::V2 => "variant_2",
///             Self::Unknown => "unknown"
///         }
///     }
/// 
///     fn from_mappedstr(s: &str) -> Self {
///         match s {
///             s if s == "variant_1" => Self::V1,
///             s if s == "variant_2"  => Self::V2,
///             s if s == "unknown" => Self::Unknown,
///             _ => Self::default()   
///         }
///     }
/// }
/// ```
/// 
/// Custom function name can be specified. It's only required on the first variants. 
/// If following variants don't specify function names they must be in the same order as the first variant. 
/// ```
/// use enum_maping::EnumMaping;
/// 
/// #[derive(EnumMaping, Debug, Eq, PartialEq)]
/// enum Example {
///     #[mapstr(fname = "vname", "variant_1")] // Specify custom fn name "to/from_vname"
///     #[mapstr("V1")] // Use default
///     #[mapstr(fname = "pretty_vname", "Variant 1")] // Use default for second
///     V1,
///
///     #[mapstr("variant_2")] // for vname
///     #[mapstr("V2")] // for default
///     #[mapstr("Variant 2")] // for pretty_vname
///     V2,
///
///     #[mapstr(fname = "pretty_vname", "Variant 3")] // Can be in any position
///     #[mapstr("V3")] // must be second #[mapstr(..)] here to match with default mapping
///     #[mapstr(fname = "vname", "variant_3")] // Can be in any position
///     V3,
///
///     #[mapstr("unknown")]
///     #[mapstr("unknown")]
///     #[mapstr("unknown")]
///     Unknown,
/// }
///
/// impl Default for Example {
///     fn default() -> Self {
///         Self::Unknown
///     }
/// }
///
/// assert_eq!(Example::V1.to_vname(), "variant_1");
/// assert_eq!(Example::V2.to_vname(), "variant_2");
/// assert_eq!(Example::V3.to_vname(), "variant_3");
/// assert_eq!(Example::V1.to_mappedstr(), "V1");
/// assert_eq!(Example::V2.to_mappedstr(), "V2");
/// assert_eq!(Example::V3.to_mappedstr(), "V3");
/// assert_eq!(Example::V1.to_pretty_vname(), "Variant 1");
/// assert_eq!(Example::V2.to_pretty_vname(), "Variant 2");
/// assert_eq!(Example::V3.to_pretty_vname(), "Variant 3");
/// ```
#[proc_macro_derive(EnumMaping, attributes(mapstr))]
pub fn enum_map(item: TokenStream) -> TokenStream {
    let ast_struct = parse_macro_input!(item as DeriveInput);
    eprintln!("{:#?}", ast_struct);

    let name = &ast_struct.ident;
    let vis = &ast_struct.vis;


    let variants = if let syn::Data::Enum(syn::DataEnum {ref variants, ..}) = ast_struct.data {
        parse_variants(variants)
    } else {
        panic!("Expected an enum.")
    };

    let v_idents = variants.iter().map(|a| &a.ident).collect::<Vec<_>>();

    let mut fns: Vec<(String, Vec<(Ident, String)>)> = Vec::new();

    for v in variants.iter() {
        for (i, (ident, new_value)) in v.maps.iter().enumerate() {
            if let Some(ident) = ident {
                if let Some(f) = fns.iter_mut().find(|&& mut (ref ident2, _)| ident2 == ident ) {
                    f.1.push((v.ident.clone(), new_value.clone()));
                } else {
                    fns.push( (ident.clone(), vec![(v.ident.clone(), new_value.clone())] ) )
                }
            } else {
                if let Some(f) = fns.get_mut(i) {
                    f.1.push((v.ident.clone(), new_value.clone()));
                } else {
                    fns.push( ("mappedstr".into(), vec![(v.ident.clone(), new_value.clone())] ) )
                }
            }
        }
    };

    let fns = fns.iter().map(|a| {
        let fn_name = &a.0;
        let to_fn_name = format_ident!("to_{}", fn_name);
        let from_fn_name = format_ident!("from_{}", fn_name);
        let v_ident = a.1.iter().map(|b| &b.0).collect::<Vec<_>>();
        let v_str = a.1.iter().map(|b| &b.1).collect::<Vec<_>>();
        quote! {
            fn #to_fn_name(&self) -> &'static str {
                match self {
                    #(Self::#v_idents => #v_str,)*
                }
            }

            fn #from_fn_name(s: &str) -> Self {
                match s {
                    #(s if s == #v_str => Self::#v_idents,)*
                    _ => Self::default()
                }
            }
    
        }
    
    }).collect::<Vec<_>>();


    let expanded = quote! {
        impl #name {
            #(#fns)*
        }

    };

    eprintln!("{:#?}", expanded);

    TokenStream::from(expanded)
}
