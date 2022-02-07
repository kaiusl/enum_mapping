use itertools::{Either, Itertools};
use proc_macro::TokenStream;
use proc_macro2::{Ident};
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
    syn::custom_keyword!(display);
}

/// Main entry of #[derive(EnumMap)] macro
pub(crate) fn enum_map(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as syn::ItemEnum);
    let enum_ident = &ast.ident;
    let enum_vis = &ast.vis;

    let mut mapings = match Mapings::parse(&ast.variants) {
        Ok(mapings) => mapings,
        Err(e) => return e.to_compile_error().into(),
    };

    let expansions = mapings.iter_mut().map(|m| m.expand(enum_ident, enum_vis));

    TokenStream::from(quote! {
        #(#expansions)*
    })
}

/// Struct to parse variants and hold intermediate state
struct Mapings {
    mapings: Vec<Maping>,
    errors: MultiError,
    is_display_implemented: bool
}

impl Mapings {
    fn parse(variants: &syn::punctuated::Punctuated<syn::Variant, Token![,]>) -> syn::Result<Vec<Maping>> {
        let mut s = Self {
            mapings: Vec::new(),
            errors: MultiError::new(),
            is_display_implemented: false
        };
    
        variants
            .iter()
            .for_each(|v| s.parse_variant(v));
    
        s.errors.inner.map(|_| s.mapings)
    }

    /// Parse single variant
    fn parse_variant(&mut self, variant: &syn::Variant) {
        let mut mapstr_idx: usize = 0;
        let has_fields = !variant.fields.is_empty();
        let mut mapings_on_this_variant: Vec<String> = Vec::new();

        variant
            .attrs
            .iter()
            .filter(|&a| a.path.segments.len() == 1)
            .for_each(|a| match &a.path.segments[0] {
                s if s.ident == "mapstr" => {
                    if let Err(e) = self.parse_mapstr_attribute(
                        &variant.ident,
                        mapstr_idx,
                        has_fields,
                        a,
                        &mut mapings_on_this_variant
                    ) {
                        self.errors.update(e);
                    }

                    mapstr_idx += 1;
                }
                _ => {}
            });
    }

    /// Parse single #[mapstr(..)]
    fn parse_mapstr_attribute(&mut self,
        vident: &Ident,
        mapstr_idx: usize,
        has_fields: bool,
        attr: &syn::Attribute,
        mapings_on_this_variant: &mut Vec<String>
    ) -> syn::Result<()> {
        let args = attr
            .parse_args_with(MapStrArguments::parse)?
            .finalize(vident);
    
        let maping = if let Some(ref fn_name) = args.name {
            let value = fn_name.value();
            // Found named mapping
            if let Some(_) = mapings_on_this_variant.iter().find(|&a| a == &value) {
                return Err(Error::duplicate_maping(value.as_str(), fn_name.span()).into())
            } else {
                mapings_on_this_variant.push(fn_name.value().clone());
            }
            self.mapings.iter_mut().find(|a| a.name == fn_name.value())
        } else {
            // Found unnamed mapping, maping must be already added, error otherwise as we don't know the name of maping.
            if let Some(m) = self.mapings.get_mut(mapstr_idx) {
                Some(m)
            } else {
                return Err(Error::arg_not_set("name", attr.path.span()).into());
            }
        };

        // Check if to implement Display
        if let Some(kw) = args.impl_display {
            if self.is_display_implemented 
                && match &maping {
                    Some(maping) => !maping.impl_display,
                    None => true
            } {
                // Some other maping is already implementing display
                return Err(Error::trait_already_implemented("Display", kw.span()).into());
            } else {
                self.is_display_implemented = true;
            }
        }
    
        // Add rule
        let rule = || {
            MapingRule {
                variant: vident.clone(),
                to: args.mapped_value,
                has_fields,
            }
        };
        
        match maping {
            Some(maping) => {
                // Maping fn already present, add new rule to the maping
                maping.rules.push(rule());
                // Set defaults if unset
                if maping.default_to.is_none() {
                    maping.default_to = args.default_to;
                }
                if maping.default_from.is_none() {
                    maping.default_from = args.default_from;
                }
                maping.create_to &= args.create_to; // If once set to false, stays false. So if any mapstr sets to=false, function won't be generated
                maping.create_from &= args.create_from; // Same as above
                maping.create_try |= args.create_try; // If once set to true, stays true.
                maping.impl_display |= args.impl_display.is_some();
            }
    
            None => {
                // First encounter of such maping
                let rules = if args.is_default {
                    // We can ignore first maping if it's set as default since we know that defaults are not set and
                    // we cannot override it with later default.
                    Vec::new()
                } else {
                    vec![rule()]
                };
    
                self.mapings.push(Maping {
                    name: args.name.unwrap().value(),
                    rules,
                    create_to: args.create_to,
                    create_from: args.create_from,
                    default_to: args.default_to,
                    default_from: args.default_from,
                    create_try: args.create_try,
                    impl_display: args.impl_display.is_some(),
                });
            }
        }
        Ok(())
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
    impl_display: bool,
}

impl Maping {
    fn expand(&mut self, eident: &syn::Ident, evis: &syn::Visibility) -> proc_macro2::TokenStream {
        let (m_fields, m_no_fields): (Vec<_>, Vec<_>) = std::mem::take(&mut self.rules)
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

        let to = self.create_to(evis, &v_fields, &v_no_fields);
        let from = self.create_from(evis, &v_no_fields);

        let display = self.create_display(eident, &v_fields, &v_no_fields);

        quote! {
            impl #eident {
                #to
                #from
            }

            #display
        }
    }

    /// Create [try]_to function TokenStreams
    fn create_to(
        &self,
        enum_vis: &syn::Visibility,
        (v_idents_fields, v_str_fields): &(Vec<Ident>, Vec<String>),
        (v_idents_no_fields, v_str_no_fields): &(Vec<Ident>, Vec<String>),
    ) -> proc_macro2::TokenStream {
        if !self.create_to {
            return quote! {};
        }

        let to = |def_to| {
            let to_fn_name = format_ident!("to_{}", self.name);
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
            let to_fn_name = format_ident!("try_to_{}", self.name);
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

        match (&self.default_to, &self.create_try) {
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
        &self,
        enum_vis: &syn::Visibility,
        (v_idents_no_fields, v_str_no_fields): &(Vec<Ident>, Vec<String>),
    ) -> proc_macro2::TokenStream {
        if !self.create_from {
            return quote! {};
        }

        let from = |def_from| {
            let from_fn_name = format_ident!("from_{}", self.name);
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
            let from_fn_name = format_ident!("try_from_{}", self.name);
            quote! {
                #enum_vis fn #from_fn_name(s: &str) -> ::std::option::Option<Self> {
                    match s {
                        #(s if s == #v_str_no_fields => ::std::option::Option::Some(Self::#v_idents_no_fields),)*
                        _ => None
                    }
                }
            }
        };

        match (&self.default_from, &self.create_try) {
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

    /// Create impl block for Display trait
    fn create_display(
        &self,
        eident: &Ident,
        (v_idents_fields, v_str_fields): &(Vec<Ident>, Vec<String>),
        (v_idents_no_fields, v_str_no_fields): &(Vec<Ident>, Vec<String>),
    ) -> proc_macro2::TokenStream {
        if !self.impl_display {
            return quote! {};
        }

        let def = if let Some(ref def) = self.default_to {
            def.clone()
        } else {
            String::from("Unknown variant")
        };

        quote! {
            impl ::std::fmt::Display for #eident {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    match self {
                        #(Self::#v_idents_no_fields => write!(f, #v_str_no_fields),)*
                        #(Self::#v_idents_fields(..) => write!(#v_str_fields),)*
                        _ => write!(f,  #def)
                    }
                }
            }
        }
    }
}
/// Parameters from one #[mapstr(..)]
#[derive(Debug)]
struct MapStrArguments {
    name: Option<syn::LitStr>,
    mapped_value: String,
    create_to: bool,
    create_from: bool,
    default_to: Option<String>,
    default_from: Option<Ident>,
    is_default: bool,
    create_try: bool,
    impl_display: Option<kw::display>,
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
            return Err(Error::arg_not_set("value", input.span()).into());
        }

        let mut name = None;
        let mut create_to = true;
        let mut create_from = true;
        let mut default_to = None;
        let mut default_from = None;
        let mut is_default = false;
        let mut create_try = false;
        let mut impl_display = None;

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
                            name = Some(value);
                        } else {
                            return Err(Error::arg_set_twice("name", input.span()).into());
                        }
                    }
                    MapStrArgument::DefaultTo { value, .. } => {
                        if default_to.is_none() {
                            default_to = Some(value.value());
                        } else {
                            return Err(Error::arg_set_twice("default_to", input.span()).into());
                        }
                    }
                    MapStrArgument::DefaultFrom { value, .. } => {
                        if default_from.is_none() {
                            default_from = Some(value);
                        } else {
                            return Err(Error::arg_set_twice("default_from", input.span()).into());
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
                    MapStrArgument::ImplDisplay { kw_token } => {
                        impl_display = Some(kw_token);
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
            impl_display,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum MapStrArgument {
    Name {
        kw_token: kw::name,
        eq_token: Token![=],
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
    ImplDisplay {
        kw_token: kw::display,
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
        } else if lookahead.peek(kw::display) {
            item_kw!(ImplDisplay)
        } else {
            Err(lookahead.error())
        }
    }
}
