use itertools::{Itertools, Either};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Token};

/// Main entry of #[derive(EnumMaping)] macro
pub(crate) fn enum_map(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    // eprintln!("{:#?}", ast_struct);
    let enum_ident = &ast.ident;
    let enum_vis = &ast.vis;

    let variants = if let syn::Data::Enum(syn::DataEnum { ref variants, .. }) = ast.data {
        variants
    } else {
        return syn::Error::new_spanned(ast, "Derive macro `EnumMaping` expected an enum.")
            .into_compile_error()
            .into();
    };

    let mut fns = match parse_variants_to_maping(variants) {
        Ok(fns) => fns,
        Err(e) => return e.to_compile_error().into(),
    };

    let fns = fns.iter_mut().map(|m_fn| {
        let (m_fn_fields, m_fn_no_fields): (Vec<_>, Vec<_>) = std::mem::take(&mut m_fn.mapings)
            .into_iter()
            .partition_map(|b| {
                let out = if b.has_fields {
                    Either::Left
                } else {
                    Either::Right
                };
                out((b.variant, b.to))
            });
        let v_fields: (Vec<_>, Vec<_>) = m_fn_fields
            .into_iter()
            .unzip();
        let v_no_fields: (Vec<_>, Vec<_>) = m_fn_no_fields
            .into_iter()
            .unzip();

        let to = create_to(enum_vis, &v_fields, &v_no_fields, m_fn);
        let from = create_from(enum_vis, &v_no_fields, m_fn);

        quote! {
            #to
            #from
        }
    });

    TokenStream::from(quote! {
        impl #enum_ident {
            #(#fns)*
        }
    })
}

/// Create [try]_to function TokenStreams
fn create_to(
    enum_vis: &syn::Visibility, 
    (v_idents_fields, v_str_fields): &(Vec<Ident>, Vec<String>),
    (v_idents_no_fields, v_str_no_fields): &(Vec<Ident>, Vec<String>),
    m_fn: &MapingFunction
) -> proc_macro2::TokenStream {
    if !m_fn.to {
        return quote!{};
    }
    let fn_name = &m_fn.name;

    let to = |def_to| {
        let to_fn_name = format_ident!("to_{}", fn_name);
        quote! {
            #enum_vis fn #to_fn_name(&self) -> &'static str {
                match self {
                    #(Self::#v_idents_no_fields => #v_str_no_fields,)*
                    #(Self::#v_idents_fields(..) => #v_str_fields,)*
                    _ => #def_to
                }
            }
        }
    };

    let try_to = || {
        let to_fn_name = format_ident!("try_to_{}", fn_name);
        quote! {
            #enum_vis fn #to_fn_name(&self) -> ::std::option::Option<&'static str> {
                match self {
                    #(Self::#v_idents_no_fields => ::std::option::Option::Some(#v_str_no_fields),)*
                    #(Self::#v_idents_fields(..) => ::std::option::Option::Some(#v_str_fields),)*
                    _ => ::std::option::Option::None
                }
            }
        }
    };

    match (&m_fn.default_to, &m_fn.try_) {
        (Some(def_to), false) => to(def_to),
        (Some(def_to), true) => {
            let to = to(def_to);
            let try_to = try_to();
            quote!{
                #to
                #try_to
            }
        },
        (None, _) => try_to()
    }
}

/// Create [try_]from functions TokenStreams.
fn create_from(
    enum_vis: &syn::Visibility,
    (v_idents_no_fields, v_str_no_fields): &(Vec<Ident>, Vec<String>),
    m_fn: &MapingFunction
) -> proc_macro2::TokenStream {
    if !m_fn.from {
        return quote!{};
    }

    let fn_name = &m_fn.name;

    let from = |def_from| {
        let from_fn_name = format_ident!("from_{}", fn_name);
        quote! {
            #enum_vis fn #from_fn_name(s: &str) -> Self {
                match s {
                    #(s if s == #v_str_no_fields => Self::#v_idents_no_fields,)*
                    _ => Self::#def_from
                }
            }
        }
    };

    let try_from = || {
        let from_fn_name = format_ident!("try_from_{}", fn_name);
        quote! {
            #enum_vis fn #from_fn_name(s: &str) -> ::std::option::Option<Self> {
                match s {
                    #(s if s == #v_str_no_fields => ::std::option::Option::Some(Self::#v_idents_no_fields),)*
                    _ => None
                }
            }
        }
    };

    match (&m_fn.default_from, &m_fn.try_) {
        (Some(def_from), false) => from(def_from),
        (Some(def_from), true) => {
            let from = from(def_from);
            let try_from = try_from();
            quote!{
                #from
                #try_from
            }
        },
        (None, _) => try_from()
    }
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
        let has_fields = !variant.fields.is_empty();

        for attr in &variant.attrs {
            let (path, nested) = if let Ok(syn::Meta::List(syn::MetaList {
                path, nested, ..
            })) = attr.parse_meta()
            {
                (path, nested)
            } else {
                continue;
            };

            if path.segments.len() > 1 {
                let new_err = syn::Error::new(path.span(), "Found unknown attribute on variant.");
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
                        attr.path.span(),
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
    attr_span: proc_macro2::Span,
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
                path, lit, ..
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
                        let new_err = syn::Error::new_spanned(
                            n,
                            "Named parameter `name` must be a string literal.",
                        );
                        errors.update(new_err);
                    }
                }
                s if s == "default_to" => {
                    if let syn::Lit::Str(l) = lit {
                        if default_to.is_none() {
                            default_to = Some(l.value())
                        } else {
                            let new_err = syn::Error::new_spanned(
                                n,
                                "`default_to` is set twice. Remove one.",
                            );
                            errors.update(new_err);
                        }
                    } else {
                        let new_err = syn::Error::new_spanned(
                            n,
                            "Named parameter `default_to` must be a string literal.",
                        );
                        errors.update(new_err);
                    }
                }
                s if s == "default_from" => {
                    if let syn::Lit::Str(l) = lit {
                        if default_from.is_none() {
                            default_from = Some(format_ident!("{}", l.value()))
                        } else {
                            let new_err = syn::Error::new_spanned(
                                n,
                                "`default_from` is set twice. Remove one.",
                            );
                            errors.update(new_err);
                        }
                    } else {
                        let new_err = syn::Error::new_spanned(
                            n,
                            "Named parameter `default_from` must be a string literal.",
                        );
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
                        let new_err =
                            syn::Error::new_spanned(n, "Found unknown `mapstr` parameter.");
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
            map_fn.to &= to; // If once set to false, stays false. So if any mapstr sets to=false, function won't be generated
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
        } else {
            // Found unnamed mapping
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
