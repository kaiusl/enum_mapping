#![allow(dead_code)]

use itertools::{Either, Itertools};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::{parse::Parse, parse_macro_input, spanned::Spanned, Token};

use crate::helpers::{Error, MultiError};

mod kw {
    syn::custom_keyword!(name);
    syn::custom_keyword!(default_to);
    syn::custom_keyword!(default_from);
    syn::custom_keyword!(no_to);
    syn::custom_keyword!(no_from);
    syn::custom_keyword!(default);
    syn::custom_keyword!(r#try);
    syn::custom_keyword!(mapstr);
}

/// Main entry of #[derive(EnumMap)] macro
pub(crate) fn enum_map(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as syn::ItemEnum);
    let enum_ident = &ast.ident;
    let enum_vis = &ast.vis;

    let mut mapings = match parse_variants_to_maping(&ast.variants) {
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
    maping: &Maping,
) -> proc_macro2::TokenStream {
    if !maping.create_to {
        return quote! {};
    }
    let fn_name = &maping.name;

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

    match (&maping.default_to, &maping.create_try) {
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
        match &attr.path.segments[0] {
            s if s.ident == "mapstr" => {
                let res =
                    parse_mapstr_attribute(mapings, &variant.ident, mapstr_idx, has_fields, attr);

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
    attr: &syn::Attribute,
) -> syn::Result<()> {
    let args = attr
        .parse_args_with(MapStrArguments::parse)?
        .finalize(vident);

    let maping = if let Some(ref fn_name) = args.name {
        // Found named mapping
        mapings.iter_mut().find(|a| &a.name == fn_name)
    } else {
        // Found unnamed mapping, maping must be already added, error otherwise as we don't know the name of maping.
        if let Some(m) = mapings.get_mut(mapstr_idx) {
            Some(m)
        } else {
            return Err(syn::Error::new(
                attr.path.span(),
                "argument `name` must be specified on first variant",
            ));
        }
    };

    match maping {
        Some(map) => {
            // Maping fn already present, add new variant to the maping
            map.rules.push(MapingRule {
                variant: vident.clone(),
                to: args.mapped_value,
                has_fields,
            });
            // Set defaults if unset
            if map.default_to.is_none() {
                map.default_to = args.default_to;
            }
            if map.default_from.is_none() {
                map.default_from = args.default_from;
            }
            map.create_to &= args.create_to; // If once set to false, stays false. So if any mapstr sets to=false, function won't be generated
            map.create_from &= args.create_from; // Same as above
            map.create_try |= args.create_try; // If once set to true, stays true.
        }

        None => {
            // First encounter of such maping
            let maping_rules = if args.is_default {
                // We can ignore first maping if it's set as default since we know that defaults are not set and
                // we cannot override it with later default.
                Vec::new()
            } else {
                vec![MapingRule {
                    variant: vident.clone(),
                    to: args.mapped_value,
                    has_fields,
                }]
            };

            mapings.push(Maping {
                name: args.name.unwrap(),
                rules: maping_rules,
                create_to: args.create_to,
                create_from: args.create_from,
                default_to: args.default_to,
                default_from: args.default_from,
                create_try: args.create_try,
            });
        }
    }
    Ok(())
}

/// Parameters for one mapping function.
///
/// Handles collecting and updating parameters.
#[derive(Debug)]
struct MapStrArguments {
    name: Option<String>,
    mapped_value: String,
    create_to: bool,
    create_from: bool,
    default_to: Option<String>,
    default_from: Option<Ident>,
    is_default: bool,
    create_try: bool,
}

impl MapStrArguments {
    fn finalize(mut self, vident: &syn::Ident) -> Self {
        if !self.is_default {
            return self;
        }

        if self.default_to.is_none() {
            self.default_to = Some(self.mapped_value.clone());
        }

        if self.default_from.is_none() {
            self.default_from = Some(vident.clone());
        }

        self
    }
}

impl syn::parse::Parse for MapStrArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Value must be first positional argument. It doesn't necessarily have to be but let's make syntax clear by forcing it.
        let lookahead = input.lookahead1();
        let mapped_value: String;
        if lookahead.peek(syn::LitStr) {
            mapped_value = input.parse::<syn::LitStr>()?.value();
        } else {
            return Err(Error::NotSet {
                arg: "value",
                span: input.span(),
            }
            .into());
        }

        let mut name = None;
        let mut create_to = true;
        let mut create_from = true;
        let mut default_to = None;
        let mut default_from = None;
        let mut is_default = false;
        let mut create_try = false;

        // There is somewhat optional comma. It's optional if `name` has been specified before.
        // It's not if we expect something afterwards. Parse anything after only is comma was found.
        // If name wasn't set before but there were some other arguments after then will get "`name` must be set" error later.
        if input.parse::<Token![,]>().is_ok() {
            let args: syn::punctuated::Punctuated<_, Token![,]> =
                input.parse_terminated(MapStrArgument::parse)?;

            for arg in args {
                match arg {
                    MapStrArgument::Name { value, .. } => {
                        if name.is_none() {
                            name = Some(value.value());
                        } else {
                            return Err(Error::SetTwice {
                                arg: "name",
                                span: value.span(),
                            }
                            .into());
                        }
                    }
                    MapStrArgument::Value { value, .. } => {
                        return Err(Error::SetTwice {
                            arg: "value",
                            span: value.span(),
                        }
                        .into())
                    }
                    MapStrArgument::DefaultTo { value, .. } => {
                        if default_to.is_none() {
                            default_to = Some(value.value());
                        } else {
                            return Err(Error::SetTwice {
                                arg: "default_to",
                                span: value.span(),
                            }
                            .into());
                        }
                    }
                    MapStrArgument::DefaultFrom { value, .. } => {
                        if default_from.is_none() {
                            default_from = Some(value);
                        } else {
                            return Err(Error::SetTwice {
                                arg: "default_from",
                                span: value.span(),
                            }
                            .into());
                        }
                    }
                    MapStrArgument::Default { .. } => {
                        is_default = true;
                    }
                    MapStrArgument::NoTo { .. } => {
                        create_to = false;
                    }
                    MapStrArgument::NoFrom { .. } => {
                        create_from = false;
                    }
                    MapStrArgument::Try { .. } => {
                        create_try = true;
                    }
                };
            }
        }

        Ok(Self {
            name,
            mapped_value,
            default_to,
            default_from,
            create_to,
            create_from,
            create_try,
            is_default,
        })
    }
}

#[derive(Debug)]
enum MapStrArgument {
    Name {
        kw_token: kw::name,
        eq_token: Token![=],
        value: syn::LitStr,
    },
    Value {
        value: syn::LitStr,
    },
    DefaultTo {
        kw_token: kw::default_to,
        eq_token: Token![=],
        value: syn::LitStr,
    },
    DefaultFrom {
        kw_token: kw::default_from,
        eq_token: Token![=],
        value: syn::Ident,
    },
    NoTo {
        kw_token: kw::no_to,
    },
    NoFrom {
        kw_token: kw::no_from,
    },
    Default {
        kw_token: kw::default,
    },
    Try {
        kw_token: kw::r#try,
    },
}

impl syn::parse::Parse for MapStrArgument {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        macro_rules! item_eq {
            ($t:ident) => {
                Ok(Self::$t {
                    kw_token: input.parse()?,
                    eq_token: input.parse()?,
                    value: input.parse()?,
                })
            };
        }

        macro_rules! item_kw {
            ($t:ident) => {
                Ok(Self::$t {
                    kw_token: input.parse()?,
                })
            };
        }

        if lookahead.peek(kw::name) {
            item_eq!(Name)
        } else if lookahead.peek(kw::default_to) {
            item_eq!(DefaultTo)
        } else if lookahead.peek(kw::default_from) {
            item_eq!(DefaultFrom)
        } else if lookahead.peek(kw::no_to) {
            item_kw!(NoTo)
        } else if lookahead.peek(kw::no_from) {
            item_kw!(NoFrom)
        } else if lookahead.peek(kw::default) {
            item_kw!(Default)
        } else if lookahead.peek(kw::r#try) {
            item_kw!(Try)
        } else if lookahead.peek(syn::LitStr) {
            Ok(Self::Value {
                value: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}
