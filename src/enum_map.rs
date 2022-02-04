use itertools::{Either, Itertools};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Token};

use crate::helpers::MultiError;

/// Main entry of #[derive(EnumMaping)] macro
pub(crate) fn enum_map(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let enum_ident = &ast.ident;
    let enum_vis = &ast.vis;

    let variants = if let syn::Data::Enum(syn::DataEnum { ref variants, .. }) = ast.data {
        variants
    } else {
        return syn::Error::new_spanned(ast, "Derive macro `EnumMaping` expected an enum.")
            .into_compile_error()
            .into();
    };

    let mut mapings = match parse_variants_to_maping(variants) {
        Ok(mapings) => mapings,
        Err(e) => return e.to_compile_error().into(),
    };

    let fns = mapings.iter_mut().map(|m| {
        let (m_fields, m_no_fields): (Vec<_>, Vec<_>) = std::mem::take(&mut m.rules)
            .into_iter()
            .partition_map(|vm| {
                let out = if vm.has_fields {
                    Either::Left
                } else {
                    Either::Right
                };
                out((vm.variant, vm.to))
            });
        let v_fields = m_fields.into_iter().unzip();
        let v_no_fields = m_no_fields.into_iter().unzip();

        let to = create_to(enum_vis, &v_fields, &v_no_fields, m);
        let from = create_from(enum_vis, &v_no_fields, m);

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
    m_fn: &Maping,
) -> proc_macro2::TokenStream {
    if !m_fn.create_to {
        return quote! {};
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

    match (&m_fn.default_to, &m_fn.create_try) {
        (Some(def_to), false) => to(def_to),
        (Some(def_to), true) => {
            let to = to(def_to);
            let try_to = try_to();
            quote! {
                #to
                #try_to
            }
        }
        (None, _) => try_to(),
    }
}

/// Create [try_]from functions TokenStreams.
fn create_from(
    enum_vis: &syn::Visibility,
    (v_idents_no_fields, v_str_no_fields): &(Vec<Ident>, Vec<String>),
    m_fn: &Maping,
) -> proc_macro2::TokenStream {
    if !m_fn.create_from {
        return quote! {};
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

    match (&m_fn.default_from, &m_fn.create_try) {
        (Some(def_from), false) => from(def_from),
        (Some(def_from), true) => {
            let from = from(def_from);
            let try_from = try_from();
            quote! {
                #from
                #try_from
            }
        }
        (None, _) => try_from(),
    }
}

/// Single maping from variant to str
#[derive(Debug)]
struct MapingRule {
    variant: Ident,
    to: String,
    has_fields: bool,
}

/// One maping with `self.name`
#[derive(Debug)]
struct Maping {
    name: String,
    rules: Vec<MapingRule>,
    create_to: bool,
    create_from: bool,
    default_to: Option<String>,
    default_from: Option<Ident>,
    create_try: bool,
}

/// Parse enum variants to Maping, which are then used to create to/from functions
fn parse_variants_to_maping(
    variants: &syn::punctuated::Punctuated<syn::Variant, Token![,]>,
) -> syn::Result<Vec<Maping>> {
    let mut mapings: Vec<Maping> = Vec::new();
    let mut errors = MultiError::new();

    variants
        .iter()
        .for_each(|v| parse_variant(v, &mut mapings, &mut errors));

    errors.inner.map(|_| mapings)
}

/// Parse attributes on a single variant
fn parse_variant(variant: &syn::Variant, mapings: &mut Vec<Maping>, errors: &mut MultiError) {
    let mut mapstr_idx: usize = 0;
    let has_fields = !variant.fields.is_empty();

    for attr in &variant.attrs {
        let (path, nested) = match attr.parse_meta() {
            Ok(syn::Meta::List(syn::MetaList { path, nested, .. })) if path.segments.len() == 1 => {
                (path, nested)
            }
            _ => continue,
        };

        match &path.segments[0] {
            s if s.ident == "mapstr" => {
                let res = parse_mapstr_attribute(
                    mapings,
                    &variant.ident,
                    mapstr_idx,
                    has_fields,
                    &nested,
                    attr.path.span(),
                );
                if let Err(new_err) = res {
                    errors.update(new_err);
                }

                mapstr_idx += 1;
            }
            _ => continue,
        }
    }
}

/// Parse single #[mapstr(..)]
fn parse_mapstr_attribute(
    mapings: &mut Vec<Maping>,
    vident: &Ident,
    mapstr_idx: usize,
    has_fields: bool,
    nested: &syn::punctuated::Punctuated<syn::NestedMeta, Token![,]>,
    attr_span: proc_macro2::Span,
) -> syn::Result<()> {
    let mut errors = MultiError::new();

    // Go through attributes inside mapstr and extract found values
    let mut params = MapStrParams::new();
    nested.iter().for_each(|a| {
        if let Err(e) = parse_mapstr_argument(a, &mut params) {
            errors.update(e);
        };
    });
    params.finalize(vident);

    // Create `Maping` structs
    let maping = if params.mapped_value.is_some() {
        if let Some(ref fn_name) = params.name {
            // Found named mapping
            Ok(mapings.iter_mut().find(|a| &a.name == fn_name))
        } else {
            // Found unnamed mapping, maping must be already added, error otherwise as we don't know the name of maping.
            if let Some(m) = mapings.get_mut(mapstr_idx) {
                Ok(Some(m))
            } else {
                Err("Parameter `name` must be specified. Simplest form should be #[mapstr(name=\"name\", \"value\")]")
            }
        }
    } else {
        Err("Missing a value to map to. Simplest form should be #[mapstr(name=\"name\", \"value\")]")
    };

    match maping {
        Ok(Some(map)) => {
            // Maping fn already present, add new variant to the maping
            map.rules.push(MapingRule {
                variant: vident.clone(),
                to: params.mapped_value.unwrap(),
                has_fields,
            });
            // Set defaults if unset
            if map.default_to.is_none() {
                map.default_to = params.default_to;
            }
            if map.default_from.is_none() {
                map.default_from = params.default_from;
            }
            map.create_to &= params.create_to; // If once set to false, stays false. So if any mapstr sets to=false, function won't be generated
            map.create_from &= params.create_from; // Same as above
            map.create_try |= params.create_try; // If once set to true, stays true.
        }

        Ok(None) => {
            // First encounter of such maping
            let maping_rules = if params.is_default {
                // We can ignore first maping if it's set as default since we know that defaults are not set and
                // we cannot override it with later default.
                Vec::new()
            } else {
                vec![MapingRule {
                    variant: vident.clone(),
                    to: params.mapped_value.unwrap(),
                    has_fields,
                }]
            };

            mapings.push(Maping {
                name: params.name.unwrap(),
                rules: maping_rules,
                create_to: params.create_to,
                create_from: params.create_from,
                default_to: params.default_to,
                default_from: params.default_from,
                create_try: params.create_try,
            });
        }

        Err(e) => {
            let new_err = syn::Error::new(attr_span, e);
            errors.update(new_err);
        }
    }

    errors.inner
}

/// Parses single mapstr argument
fn parse_mapstr_argument(n: &syn::NestedMeta, params: &mut MapStrParams) -> syn::Result<()> {
    let res =
        match n {
            syn::NestedMeta::Lit(syn::Lit::Str(l)) => {
                // Only possible literal is the value to map to
                params.set_mapped_value(l.value())
            }
            syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                path, lit, ..
            })) if path.segments.len() == 1 => match &path.segments[0].ident {
                s if s == "name" => params.set_name(lit),
                s if s == "default_to" => params.set_default_to(lit),
                s if s == "default_from" => params.set_default_from(lit),
                _ => Err("Found unknown `mapstr` parameter."),
            },
            syn::NestedMeta::Meta(syn::Meta::Path(syn::Path { segments, .. }))
                if segments.len() == 1 =>
            {
                match &segments[0].ident {
                    s if s == "default" => params.set_is_default(),
                    s if s == "no_to" => params.set_no_to(),
                    s if s == "no_from" => params.set_no_from(),
                    s if s == "try" => params.set_try(),
                    _ => Err("Found unknown `mapstr` parameter."),
                }
            }
            _ => Err("Found unknown `mapstr` parameter."),
        };

    res.map_err(|e| syn::Error::new_spanned(n, e))
}

/// Parameters for one mapping function.
///
/// Handles collecting and updating parameters.
struct MapStrParams {
    name: Option<String>,
    mapped_value: Option<String>,
    create_to: bool,
    create_from: bool,
    default_to: Option<String>,
    default_from: Option<Ident>,
    is_default: bool,
    create_try: bool,
}

impl MapStrParams {
    fn new() -> Self {
        Self {
            name: None,
            mapped_value: None,
            create_to: true,
            create_from: true,
            default_to: None,
            default_from: None,
            is_default: false,
            create_try: false,
        }
    }

    fn set_mapped_value(&mut self, v: String) -> Result<(), &'static str> {
        if self.mapped_value.is_none() {
            self.mapped_value = Some(v);
            Ok(())
        } else {
            Err("`value` is set twice. Remove one.")
        }
    }

    fn set_name(&mut self, v: &syn::Lit) -> Result<(), &'static str> {
        if let syn::Lit::Str(l) = v {
            if self.name.is_none() {
                self.name = Some(l.value())
            } else {
                return Err("`name` is set twice. Remove one.");
            }
        } else {
            return Err("Named parameter `name` must be a string literal.");
        };

        Ok(())
    }

    fn set_default_to(&mut self, v: &syn::Lit) -> Result<(), &'static str> {
        if let syn::Lit::Str(l) = v {
            if self.default_to.is_none() {
                self.default_to = Some(l.value())
            } else {
                return Err("`default_to` is set twice. Remove one.");
            }
        } else {
            return Err("Named parameter `default_to` must be a string literal.");
        }
        Ok(())
    }

    fn set_default_from(&mut self, v: &syn::Lit) -> Result<(), &'static str> {
        if let syn::Lit::Str(l) = v {
            if self.default_from.is_none() {
                self.default_from = Some(format_ident!("{}", l.value()))
            } else {
                return Err("`default_from` is set twice. Remove one.");
            }
        } else {
            return Err("Named parameter `default_from` must be a string literal.");
        }
        Ok(())
    }

    fn set_is_default(&mut self) -> Result<(), &'static str> {
        self.is_default = true;
        Ok(())
    }

    fn set_no_to(&mut self) -> Result<(), &'static str> {
        self.create_to = false;
        Ok(())
    }

    fn set_no_from(&mut self) -> Result<(), &'static str> {
        self.create_from = false;
        Ok(())
    }

    fn set_try(&mut self) -> Result<(), &'static str> {
        self.create_try = true;
        Ok(())
    }

    fn finalize(&mut self, vident: &syn::Ident) {
        // If current variant is marked default, set default_to/from
        // Ignore is they are already set
        if !self.is_default {
            return;
        }

        if self.default_to.is_none() {
            self.default_to = self.mapped_value.clone();
        }

        if self.default_from.is_none() {
            self.default_from = Some(vident.clone());
        }
    }
}
