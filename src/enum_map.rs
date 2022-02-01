use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Token};

pub(crate) fn enum_map(item: TokenStream) -> TokenStream {


    let ast_struct = parse_macro_input!(item as DeriveInput);
    // eprintln!("{:#?}", ast_struct);

    let name = &ast_struct.ident;
    let vis = &ast_struct.vis;

    let variants = if let syn::Data::Enum(syn::DataEnum { ref variants, .. }) = ast_struct.data {
        variants
    } else {
        return syn::Error::new_spanned(ast_struct, "Derive macro `EnumMaping` expected an enum.")
            .into_compile_error()
            .into();
    };

    let fns = match parse_variants_to_maping(variants) {
        Ok(fns) => fns,
        Err(e) => return e.to_compile_error().into(),
    };

    let fns = fns.iter().map(|m_fn| {
        let fn_name = &m_fn.name;
        let v_idents_no_fields = m_fn.mapings.iter().filter_map(|b| if b.has_fields {None} else {Some(&b.variant)}).collect::<Vec<_>>();
        let v_str_no_fields = m_fn.mapings.iter().filter_map(|b| if b.has_fields {None} else {Some(&b.to)}).collect::<Vec<_>>();
        let v_idents_fields = m_fn.mapings.iter().filter_map(|b| if !b.has_fields {None} else {Some(&b.variant)}).collect::<Vec<_>>();
        let v_str_fields = m_fn.mapings.iter().filter_map(|b| if !b.has_fields {None} else {Some(&b.to)}).collect::<Vec<_>>();
        
        let to = if m_fn.to {
            // to_..(_) -> &'static str
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
            // try_to_..(_) -> Option<&'static str>
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
            // from_..(_) -> Self
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
            // try_from_..(_) -> Option<Self>
            if m_fn.default_from.is_none() || m_fn.try_ {
                let from_fn_name = format_ident!("try_from_{}", fn_name);
                quote! {
                    #from

                    #vis fn #from_fn_name(s: &str) -> ::std::option::Option<Self> {
                        match s {
                            #(s if s == #v_str_no_fields => ::std::option::Option::Some(Self::#v_idents_no_fields),)*
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

#[derive(Debug)]
struct Maping {
    variant: Ident,
    to: String,
    has_fields: bool,
}

#[derive(Debug)]
struct MapingFunction {
    name: String,
    mapings: Vec<Maping>,
    to: bool,
    from: bool,
    default_to: Option<String>,
    default_from: Option<Ident>,
    try_: bool,
}

struct MultiError {
    inner: syn::Result<()>,
}

impl MultiError {
    fn new() -> Self {
        Self { inner: Ok(()) }
    }

    fn update(&mut self, new_err: syn::Error) {
        if let Err(ref mut e) = self.inner {
            e.combine(new_err);
        } else {
            self.inner = Err(new_err);
        }
    }
}

/// Parse enum variants to MapingFunction, which can be used to create to/from functions
fn parse_variants_to_maping(
    variants: &syn::punctuated::Punctuated<syn::Variant, Token![,]>,
) -> syn::Result<Vec<MapingFunction>> {
    let mut maping_fns: Vec<MapingFunction> = Vec::new();

    let mut errors = MultiError::new();

    for variant in variants {
        let mut mapstr_idx: usize = 0;
        let has_fields = variant.fields != syn::Fields::Unit;

        for attr in &variant.attrs {
            let (path, nested) = if let Ok(syn::Meta::List(syn::MetaList { path, nested, .. })) = attr.parse_meta() {
                (path, nested)
            } else {
                continue
            };
            
            if path.segments.len() > 1 {
                let new_err =
                    syn::Error::new(path.span(), "Found unknown attribute on variant.");
                errors.update(new_err);
            }
            match &path.segments[0] {
                s if s.ident == "mapstr" => {
                    let b = parse_mapstr_attr(
                        &mut maping_fns,
                        &variant.ident,
                        mapstr_idx,
                        has_fields,
                        &nested,
                        attr.path.span()
                    );
                    if let Err(new_err) = b {
                        errors.update(new_err);
                    }

                    mapstr_idx += 1;
                }
                _ => {
                    let new_err =
                        syn::Error::new(path.span(), "Found unknown attribute on variant.");
                    errors.update(new_err);
                }
            }
        }
            
    }

    if let Err(e) = errors.inner {
        Err(e)
    } else {
        Ok(maping_fns)
    }
}

fn parse_mapstr_attr(
    maping_fns: &mut Vec<MapingFunction>,
    vident: &Ident,
    mapstr_idx: usize,
    has_fields: bool,
    nested: &syn::punctuated::Punctuated<syn::NestedMeta, Token![,]>,
    attr_span: proc_macro2::Span
) -> syn::Result<()> {
    let mut fn_name = None;
    let mut mapped_value = None;
    let mut to = true;
    let mut from = true;
    let mut default_to = None;
    let mut default_from = None;
    let mut is_default = false;
    let mut try_ = false;

    let mut errors = MultiError::new();
    // Go through attributes inside mapstr and extract found values
    for n in nested {
        match n {
            // Just literal, the value to map to
            syn::NestedMeta::Lit(syn::Lit::Str(l)) => {
                if mapped_value.is_none() {
                    mapped_value = Some(l.value());
                } else {
                    let new_err = syn::Error::new_spanned(n, "`value` is set twice. Remove one.");
                    errors.update(new_err);
                }
            }
            syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                path,
                lit,
                ..
            })) if path.segments.len() == 1 => match &path.segments[0].ident {
                s if s == "name" => {
                    if let syn::Lit::Str(l) = lit {
                        if fn_name.is_none() {
                            fn_name = Some(l.value())
                        } else {
                            let new_err =
                                syn::Error::new_spanned(n, "`name` is set twice. Remove one.");
                            errors.update(new_err);
                        }
                    } else {
                        let new_err =
                                syn::Error::new_spanned(n, "Named parameter `name` must be a string literal.");
                        errors.update(new_err);
                    }
                },
                s if s == "default_to" => {
                    if let syn::Lit::Str(l) = lit {
                        if default_to.is_none() {
                            default_to = Some(l.value())
                        } else {
                            let new_err =
                                syn::Error::new_spanned(n, "`default_to` is set twice. Remove one.");
                            errors.update(new_err);
                        }
                    } else {
                        let new_err =
                                syn::Error::new_spanned(n, "Named parameter `default_to` must be a string literal.");
                        errors.update(new_err);
                    }
                }
                s if s == "default_from" => {
                    if let syn::Lit::Str(l) = lit {
                        if default_from.is_none() {
                            default_from = Some(format_ident!("{}", l.value()))
                        } else {
                            let new_err =
                                syn::Error::new_spanned(n, "`default_from` is set twice. Remove one.");
                            errors.update(new_err);
                        }
                    } else {
                        let new_err =
                                syn::Error::new_spanned(n, "Named parameter `default_from` must be a string literal.");
                        errors.update(new_err);
                    }
                }
                _ => {
                    let new_err = syn::Error::new_spanned(n, "Found unknown `mapstr` parameter.");
                    errors.update(new_err);
                }
            },
            syn::NestedMeta::Meta(syn::Meta::Path(syn::Path { segments, .. }))
                if segments.len() == 1 =>
            {
                match &segments[0].ident {
                    s if s == "default" => is_default = true,
                    s if s == "no_to" => to = false,
                    s if s == "no_from" => from = false,
                    s if s == "try" => try_ = true,
                    _ => {
                        let new_err = syn::Error::new_spanned(n, "Found unknown `mapstr` parameter.");
                        errors.update(new_err);
                    }
                }
            }
            _ => {
                let new_err = syn::Error::new_spanned(n, "Found unknown `mapstr` parameter.");
                errors.update(new_err);
            }
        }
    }

    // Create `MapingFunction` structs
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
                    has_fields,
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

        if let Some(fn_name) = fn_name { // Found named mapping
            if let Some(map_fn) = maping_fns.iter_mut().find(|a| a.name == fn_name) {
                add_if_present(map_fn);
            } else {
                // First encounter of such maping function
                let mapings = if is_default {
                    Vec::new()
                } else {
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
                    try_,
                });
            }
        } else { // Found unnamed mapping
            if let Some(map_fn) = maping_fns.get_mut(mapstr_idx) {
                add_if_present(map_fn);
            } else {
                let new_err = syn::Error::new(attr_span, "Parameter `name` must be specified. Simplest form should be #[mapstr(name=\"name\", \"value\")]");
                errors.update(new_err);
            }
        }
    } else {
        let new_err = syn::Error::new(attr_span, "Missing a value to map to. Simplest form should be #[mapstr(name=\"name\", \"value\")]");
        errors.update(new_err);
    }

    errors.inner
}