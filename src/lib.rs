use std::collections::HashMap;

use proc_macro::{TokenStream};
use proc_macro2::Ident;
use quote::{format_ident, quote, __private::ext::RepToTokensExt};
use syn::{ItemStruct, Token, bracketed, parenthesized, parse_macro_input, punctuated::Punctuated, ItemEnum, DeriveInput};


struct Variant {
    ident: syn::Ident,
    maps: Vec<(Option<String>, String)>
}



#[proc_macro_derive(EnumMap, attributes(mapstr))]
pub fn database(item: TokenStream) -> TokenStream {
    let ast_struct = parse_macro_input!(item as DeriveInput);
    eprintln!("{:#?}", ast_struct);

    let name = &ast_struct.ident;
    let vis = &ast_struct.vis;


    let variants = if let syn::Data::Enum(syn::DataEnum {ref variants, ..}) = ast_struct.data {
        variants
    } else {
        panic!("Expected an enum.")
    };


    let variants = variants.iter().filter_map(|variant| {
        let mut v = Variant { 
            ident: variant.ident.clone(),
            maps: Vec::new()
        };

        for attr in &variant.attrs {

            let b = &attr.parse_meta().unwrap();
            eprintln!("\n\n attr of {} \n\n{:#?}", &variant.ident, b);
            if let syn::Meta::List(syn::MetaList {path, nested, ..}) = b {
                match &path.segments[0] {
                    s if s.ident == "mapstr" => {
                        
                        let mut fn_name = None;
                        let mut mapped_value = None;
                        for n in nested {
                            match n {
                                syn::NestedMeta::Lit(syn::Lit::Str(l)) => {
                                    mapped_value = Some(l.value());
                                },
                                syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {path, lit: syn::Lit::Str(l), ..})) => {
                                    match &path.segments[0].ident {
                                        s if s == "fn_name" => {
                                          fn_name = Some(l.value());  
                                        },
                                        _ => panic!("Unknown mapstr named argument.")
                                    }
                                }
                                _ => {
                                    continue;
                                }
                            }
                        }

                        if mapped_value.is_none() {
                            continue;
                        }

                        if fn_name.is_none() {
                            v.maps.push((Some("str".to_string()), mapped_value.unwrap()));
                        } else {
                            v.maps.push((fn_name, mapped_value.unwrap()));
                        }
                    },
                    _ => {
                        continue;
                    }
                }
            }            
        }
        if v.maps.len() > 0 {
            Some(v)
        } else {
           None
        }
    }).collect::<Vec<_>>();

    let v_idents = variants.iter().map(|a| &a.ident).collect::<Vec<_>>();
    //let v_str = variants.iter().map(|a| &a.maps["kunos_id"]).collect::<Vec<_>>();

    let mut fns: Vec<(String, Vec<(Ident, String)>)> = Vec::new();

    for v in variants.iter() {
        for (i, (ident, new_value)) in v.maps.iter().enumerate() {
            if let Some(ident) = ident {
                if let Some(f) = fns.get_mut(i) {
                    f.1.push((v.ident.clone(), new_value.clone()));
                } else {
                    fns.push( (ident.clone(), vec![(v.ident.clone(), new_value.clone())] ) )
                }
            } else {
                if let Some(f) = fns.get_mut(i) {
                    f.1.push((v.ident.clone(), new_value.clone()));
                } else {
                    fns.push( ("str".into(), vec![(v.ident.clone(), new_value.clone())] ) )
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

            fn #from_fn_name(&self, s: &str) -> Self {
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
