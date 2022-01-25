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
//! 
//! impl Example {
//!     fn to_vname(&self) -> &'static str {
//!         match self {
//!             Self::V1 => "variant_1",
//!             Self::V2 => "variant_2",
//!             _ => "unknown"
//!         }
//!     }
//! 
//!     fn from_vname(s: &str) -> Self {
//!         match s {
//!             s if s == "variant_1" => Self::V1,
//!             s if s == "variant_2"  => Self::V2,
//!             _ => Self::Unknown   
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
//!     #[mapstr(name="vname", "variant_1")]
//!     V1,
//!     #[mapstr("variant_2")]
//!     V2,
//!     #[mapstr("unknown", default)]
//!     Unknown
//! }
//! ```

use proc_macro::{TokenStream};
use proc_macro2::{Ident};
use quote::{format_ident, quote};
use syn::{Token, parse_macro_input, DeriveInput, spanned::Spanned};

#[derive(Debug)]
struct Maping {
    variant: Ident,
    to: String,
    has_fields: bool
}

#[derive(Debug)]
struct MapingFunction {
    name: String,
    mapings: Vec<Maping>,
    to: bool,
    from: bool,
    default_to: Option<String>,
    default_from: Option<Ident>,
    try_: bool
}

/// Parse enum variants to MapingFunction, which can be used to create to/from functions
fn parse_variants_to_maping(variants: &syn::punctuated::Punctuated<syn::Variant, Token![,]>) -> syn::Result<Vec<MapingFunction>> {
    let mut maping_fns: Vec<MapingFunction> = Vec::new();

    variants.iter().map(|variant| {
        let mut mapstr_idx: usize = 0;
        let has_fields = variant.fields != syn::Fields::Unit;
        variant.attrs.iter()
            .filter_map(|attr| attr.parse_meta().ok())
            .filter_map(|meta| if let syn::Meta::List(syn::MetaList {path, nested, ..}) = meta {
                Some((path, nested))
            } else {
                None
            }).map(|(path, nested)| {
                if path.segments.len() > 1 {
                    return Err(syn::Error::new(path.span(), "Found unknown attribute on variant."));
                }
                match &path.segments[0] {
                    s if s.ident == "mapstr" => {
                        let b = parse_mapstr_attr(&mut maping_fns, &variant.ident, mapstr_idx, has_fields, &nested);
                        mapstr_idx += 1;
                        return b;
                    },
                    _ => {return Err(syn::Error::new(path.span(), "Found unknown attribute on variant."));}
                }
            }).collect::<Result<Vec<_>, _>>()
        }).collect::<Result<Vec<_>, _>>()?;

        Ok(maping_fns)
}

fn parse_mapstr_attr(
    maping_fns: &mut Vec<MapingFunction>, 
    vident: &Ident, 
    mapstr_idx: usize, 
    has_fields: bool, 
    nested: &syn::punctuated::Punctuated<syn::NestedMeta, Token![,]>
) -> syn::Result<()> {
    let mut fn_name = None;
    let mut mapped_value = None;
    let mut to = true;
    let mut from = true;
    let mut default_to = None;
    let mut default_from = None;
    let mut is_default = false;
    let mut try_ = false;
    // Go through attributes inside mapstr and extract found values
    for n in nested {
        match n {
            // Just literal, the value to map to 
            syn::NestedMeta::Lit(syn::Lit::Str(l)) => {
                if mapped_value.is_none() {
                    mapped_value = Some(l.value());
                } else {
                    return Err(syn::Error::new_spanned(n, "`value` is set twice. Remove one."));
                }
            },
            // Named arguments
            syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {path, lit: syn::Lit::Str(l), ..})) => {
                match &path.segments[0].ident {
                    s if s == "name" => {
                        if fn_name.is_none() {
                            fn_name = Some(l.value())
                        } else {
                            return Err(syn::Error::new_spanned(n, "`name` is set twice. Remove one."));
                        }
                    },
                    s if s == "default_to" => {
                        if default_to.is_none() {
                            default_to = Some(l.value())
                        } else {
                            return Err(syn::Error::new_spanned(n, "`default_to` is set twice. Remove one."));
                        }
                    },
                    s if s == "default_from" => {
                        if default_from.is_none() {
                            default_from = Some(format_ident!("{}", l.value()))
                        } else {
                            return Err(syn::Error::new_spanned(n, "`default_from` is set twice. Remove one."));
                        }
                    },
                    _ => { return Err(syn::Error::new_spanned(n, "Found unknown `mapstr` parameter.")); }
                }
            },
            syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {path, lit: syn::Lit::Bool(l), ..})) => {
                match &path.segments[0].ident {
                    s if s == "to" => { to = l.value(); },
                    s if s == "from" => { from = l.value(); }
                    s if s == "try" => { try_ = l.value(); }
                    _ => { return Err(syn::Error::new_spanned(n, "Found unknown `mapstr` parameter.")); }
                }
            },
            syn::NestedMeta::Meta(syn::Meta::Path(syn::Path {segments, ..})) => {
                if segments.len() != 1 {
                    return Err(syn::Error::new_spanned(n, "Found unknown `mapstr` parameter."));
                }
                match &segments[0].ident {
                    s if s == "default" => { is_default = true },
                    _ => { return Err(syn::Error::new_spanned(n, "Found unknown `mapstr` parameter")); }
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(n, "Found unknown `mapstr` parameter."));
            }
        }
    }

    // Create `MapingFunction`-s
    if let Some(mapped_value) = mapped_value {
        // If current variant is marked default, set default_to/from
        // Ignore is they are already set
        if is_default && default_to.is_none() {
            default_to = Some(mapped_value.clone());
        }
        if is_default && default_from.is_none() {
            default_from = Some(vident.clone());
        }

        // Function to add maping if function is already present.
        let add_if_present = |map_fn: &mut MapingFunction| {
            // Mapping fn already present, add new maping
            // Don't if marked as default, as _ branch would cover it anyway
            if !is_default { 
                map_fn.mapings.push(Maping {
                    variant: vident.clone(),
                    to: mapped_value.clone(),
                    has_fields
                });
            }
            // Set defaults if unset
            if map_fn.default_to.is_none() {
                map_fn.default_to = default_to.clone();
            }
            if map_fn.default_from.is_none() {
                map_fn.default_from = default_from.clone();
            }
            map_fn.to &= to; // If once set to false, stays false. So if any mapstr sets to=false, function won't be generates
            map_fn.from &= from; // Same as above
            map_fn.try_ |= try_; // If once set to true, stays true. 
        };

        if let Some(fn_name) = fn_name {
            // Found named mapping
            if let Some(map_fn) = maping_fns.iter_mut().find(|a| a.name == fn_name) {
                add_if_present(map_fn);
            } else {
                // First encounter of such maping function
                let mapings = if is_default {
                    Vec::new()
                } else  {
                    vec![Maping {
                        variant: vident.clone(),
                        to: mapped_value,
                        has_fields,
                    }]
                };

                maping_fns.push(MapingFunction {
                    name: fn_name,
                    mapings,
                    to,
                    from,
                    default_to,
                    default_from,
                    try_
                });
            }
        } else {
            // Found unnamed mapping
            if let Some(map_fn) = maping_fns.get_mut(mapstr_idx) {
                add_if_present(map_fn);
            } else {
                return Err(syn::Error::new_spanned(nested, "Parameter `name` must be specified. Simplest form should be #[mapstr(name=\"name\", \"value\")]"));
            }
        }
    } else {
        return Err(syn::Error::new_spanned(nested, "Missing a value to map to. Simplest form should be #[mapstr(name=\"name\", \"value\")]"));
    }

    Ok(())
}


/// Macro to derive custom mapings for enum types.
/// 
/// It provides function implementations for `to` and `from` functions for enum. 
/// Maping is specified on enum variant by attribute `#[mapstr(_)]`.
/// 
/// Multiple mappings can be specified for single variant.
/// If `fname` is provided, the order doesn't matter. 
/// If it is not then that `mapstr` must be at the same position as it first appeared.
/// 
/// By default this macro will create two functions `fn try_to_<fname>(&self) -> Option<&'static str`> and `fn try_from_<fname>(s: &str) -> Option<Self>`.
/// If defaults are set the created functions are `fn to_<fname>(&self) -> &'static str` and `fn from_<fname>(s: &str) -> Self`. 
/// First set of functions can still be then created be passing argument `try=true` to the `mapstr` attribute.
/// 
/// If maping is applied to an enum which variants have field then `to` function ignores field values. 
/// `From` function must return default or `None` instead of variant with fields as we don't really know what to provide in those fields.
/// I suppose if all variants have the same field we could create function with extra parameters but if there are many different
/// types stored in variants then every single one of them would need to be in the function signature and that's not reasonable thing to do.
/// 
/// # Variant attributes
/// * `mapstr(<value>, name="", [default, default_to="", default_from="", to=true, from=true, ])`
///     - `value`: string - string to map to
///     - `name`: string - set created function name as `(try)_to_<fname>` and `(try)_from_<fname>`. Must be set on first variant part of the mapping.
///     - `default` - set variant as default. Optional. If set resulting functions will return directly `&str` and `Self` and remove "try" from the name.
///     - `default_to`: string - set default string to map to. Optional. If set resulting function will return directly `&str` and remove "try" from the function name.
///     - `default_from`: string - set default variant to map to. Optional. If set resulting function will return directly `Self` and remove "try" from the function name.
///     - `try`: bool - create functions returning [`Option`](_) also if defaults are set. Optional, defaults to `false`.
///     - `to`: bool - create `to` function. Optional, defaults to `true`.
///     - `from`: bool - create `from` function. Optional, defaults to `true`. 
/// 
/// Optional arguments can be specified on any of the variants but only the first specification is used.
/// 
/// # Current shortcomings
/// * Error messages from macro are simple panics (and at places completely with wrong messages)
///  
/// # Examples
/// Simplest form with default function names
/// ```
/// use enum_maping::EnumMaping;
///
/// #[derive(EnumMaping, Debug, Eq, PartialEq)]
/// enum Example {
///     #[mapstr(name="vname", "variant_1")]
///     V1,
///     #[mapstr("variant_2")]
///     V2,
///     #[mapstr("unknown", default)] // Set as default variant
///     Unknown
/// 
/// }
/// 
/// assert_eq!(Example::V1.to_vname(), "variant_1");
/// assert_eq!(Example::V2.to_vname(), "variant_2");
/// assert_eq!(Example::from_vname("variant_1"), Example::V1);
/// assert_eq!(Example::from_vname("variant_3"), Example::Unknown);
/// ```
/// This example expands to
/// ```ignore
/// impl Example {
///     fn to_vname(&self) -> &'static str {
///         match self {
///             Self::V1 => "variant_1",
///             Self::V2 => "variant_2",
///             _ => "unknown"
///         }
///     }
/// 
///     fn from_vname(s: &str) -> Self {
///         match s {
///             s if s == "variant_1" => Self::V1,
///             s if s == "variant_2"  => Self::V2,
///             _ => Self::Unknown   
///         }
///     }
/// }
/// ```
/// Following shows different options.
/// ```
/// use enum_maping::EnumMaping;
/// 
/// #[derive(EnumMaping, Debug, Eq, PartialEq)]
/// enum Example {
///     #[mapstr(name = "vname", "variant_1")] //
///     #[mapstr(name = "short", "V1", default_to="unknown", default_from="Unknown")] // Set defaults
///     #[mapstr(name = "pretty", "Variant 1")]
///     V1,
/// 
///     // Mapings in the same order as on the first variant
///     #[mapstr("variant_2")] // vname
///     #[mapstr("V2")] // short
///     // ignore in pretty
///     #[mapstr(name = "caps", "VARIANT_2")] // Create new maping ignoring the first variant.
///     V2,
/// 
///     // If `name` is specified, order doesn't matter. If not it must be in the correct place.
///     #[mapstr(name = "pretty", "Variant 3")] // Can be reordered
///     #[mapstr("V3")] // Must be second to be part of "short" maping
///     #[mapstr(name = "vname", "variant_3")] // Can be reordered
///     #[mapstr("VARIANT_3")] // part of "Caps" as that was specified fourth
///     V3,
/// 
///     #[mapstr(name = "vname", default, "unknown")] // Set this variant to be default of "vname" maping
///     Unknown,
/// 
///     #[mapstr(name = "caps", "ERR")]
///     Error
/// }
/// 
/// assert_eq!(Example::V1.to_vname(), "variant_1");
/// assert_eq!(Example::Unknown.to_vname(), "unknown");
/// assert_eq!(Example::Error.to_vname(), "unknown");
/// assert_eq!(Example::from_vname("variant_1"), Example::V1);
/// assert_eq!(Example::from_vname("err"), Example::Unknown);
/// 
/// assert_eq!(Example::V3.try_to_pretty(), Some("Variant 3"));
/// assert_eq!(Example::V2.try_to_pretty(), None);
/// assert_eq!(Example::Unknown.try_to_pretty(), None);
/// assert_eq!(Example::try_from_pretty("Variant 3"), Some(Example::V3));
/// assert_eq!(Example::try_from_pretty("unknown"), None); 
/// 
/// ```
#[proc_macro_derive(EnumMaping, attributes(mapstr))]
pub fn enum_map(item: TokenStream) -> TokenStream {
    let ast_struct = parse_macro_input!(item as DeriveInput);
    //eprintln!("{:#?}", ast_struct);

    let name = &ast_struct.ident;
    let vis = &ast_struct.vis;


    let variants = if let syn::Data::Enum(syn::DataEnum {ref variants, ..}) = ast_struct.data {
        variants
    } else {
        return syn::Error::new_spanned(ast_struct, "Derive macro `EnumMaping` expected an enum.").into_compile_error().into();
    };

    let fns = match parse_variants_to_maping(variants) {
        Ok(fns) => fns,
        Err(e) => {
            return e.to_compile_error().into()
        }
    };

    let fns = fns.iter().map(|m_fn| {
        let fn_name = &m_fn.name;
        let v_idents_no_fields = m_fn.mapings.iter().filter_map(|b| if b.has_fields {None} else {Some(&b.variant)}).collect::<Vec<_>>();
        let v_str_no_fields = m_fn.mapings.iter().filter_map(|b| if b.has_fields {None} else {Some(&b.to)}).collect::<Vec<_>>();
        let v_idents_fields = m_fn.mapings.iter().filter_map(|b| if !b.has_fields {None} else {Some(&b.variant)}).collect::<Vec<_>>();
        let v_str_fields = m_fn.mapings.iter().filter_map(|b| if !b.has_fields {None} else {Some(&b.to)}).collect::<Vec<_>>();
        
        let to = if m_fn.to {
            // to_<fname>(_) -> &'static str
            let to = if let Some(def_to) = &m_fn.default_to {
                let to_fn_name = format_ident!("to_{}", fn_name);
                quote! {
                    #vis fn #to_fn_name(&self) -> &'static str {
                        match self {
                            #(Self::#v_idents_no_fields => #v_str_no_fields,)*
                            #(Self::#v_idents_fields(_) => #v_str_fields,)*
                            _ => #def_to
                        }
                    }
                }
            } else {
                quote!{}
            };
            // try_to_<name>(_) -> Option<&'static str>
            if m_fn.default_to.is_none() || m_fn.try_ {
                let to_fn_name = format_ident!("try_to_{}", fn_name);
                quote! {
                    #to

                    #vis fn #to_fn_name(&self) -> std::option::Option<&'static str> {
                        match self {
                            #(Self::#v_idents_no_fields => std::option::Option::Some(#v_str_no_fields),)*
                            #(Self::#v_idents_fields(_) => std::option::Option::Some(#v_str_fields),)*
                            _ => None
                        }
                    }
                }
            } else {
                to
            }
        } else {
            quote! {}
        };

        let from = if m_fn.from {
            // from_<fname>(_) -> Self
            let from = if let Some(def_from) = &m_fn.default_from {
                let from_fn_name = format_ident!("from_{}", fn_name);
                quote! {
                    #vis fn #from_fn_name(s: &str) -> Self {
                        match s {
                            #(s if s == #v_str_no_fields => Self::#v_idents_no_fields,)*
                            _ => Self::#def_from
                        }
                    }
                }
            } else {
                quote! {}
            };
            // try_from_<fname>(_) -> Option<Self>
            if m_fn.default_from.is_none() || m_fn.try_ {
                let from_fn_name = format_ident!("try_from_{}", fn_name);
                quote! {
                    #from

                    #vis fn #from_fn_name(s: &str) -> std::option::Option<Self> {
                        match s {
                            #(s if s == #v_str_no_fields => std::option::Option::Some(Self::#v_idents_no_fields),)*
                            _ => None
                        }
                    }
                }
            } else {
                from
            }
        } else { 
            quote! {}
        };


        quote! {
            #to

            #from    
        }
    
    }).collect::<Vec<_>>();

    TokenStream::from(quote! {
        impl #name {
            #(#fns)*
        }
    })
}
