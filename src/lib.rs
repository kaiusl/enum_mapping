//! Quick enum mapings to strings
//!
//! This crate provides a derive macro `#[derive(EnumMap)]` to quickly create mappings between enum variants and strings.
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
//! use enum_map::EnumMap;
//!
//! #[derive(EnumMap)]
//! enum Example {
//!     #[mapstr("variant_1", name="vname")]
//!     V1,
//!     #[mapstr("variant_2")]
//!     V2,
//!     #[mapstr("unknown", default)]
//!     Unknown
//! }
//! ```

use proc_macro::TokenStream;
mod enum_map;

mod helpers;

/// # Macro to derive custom mapings for enum types.
/// It provides function implementations for `to` and `from` functions for enum.
/// Maping is specified on enum variant by attribute `#[mapstr(..)]`.
/// Multiple mappings can be specified for single variant.
/// If `name` is provided, the order doesn't matter.
/// If it is not then that `mapstr` must be at the same position as it first appeared. [Examples](#examples) below.
/// By default this macro will create two functions
/// * `fn try_to_<name>(&self) -> Option<&'static str`>,
/// * `fn try_from_<name>(s: &str) -> Option<Self>`.
/// 
/// If defaults are set the created functions are
/// * `fn to_<name>(&self) -> &'static str`,
/// * `fn from_<name>(s: &str) -> Self`.
/// 
/// First set of functions can still be then created be passing argument `try` to `mapstr` attribute.
/// # Variant attributes
/// * `mapstr(<value> [,opts])`
///     - `value`: *string literal* - string to map to
///     - `name=".."` : *string literal* - set created function name as `[try]_to_<name>` `[try]_from_<fame>`. Must be set on /// first variant part of the maping.
///     - `default_to=".."` : *string literal* - set default string to map to. Optional. If set result function will return /// directly `&str` and remove "try" from the function name.
///     - `default_from=..` : *identifier* - set default variant to map to. Optional. If set result function will return directly /// `Self` and remove "try" from the function name.
///     - `default` : *optional keyword* - set variant as default. Optional. If set resulting functions wreturn directly `&str` /// and `Self` and remove "try" from the name. Arguments `default_to/from` take precedence over this keyword.
///     - `try` : *optional keyword* - if set create functions returning [`Option`](_) even if defaults are set.
///     - `no_to` : *optional keyword* - if set don't create `to` methods.
///     - `no_from` : *optional keyword* - if set don't create `from` methods.
///     - `display` : *optional keyword* - create implementation for [`Display`](std::fmt::Display) trait. It can only be present /// on one maping set. If default is not set then default display is `"Unknown variant"`.
/// 
/// Optional arguments can be specified on any of the variants but only the first specification is used.
/// # Current shortcomings
/// * Variants with fields have limited support. They cannot be created with `frfunctions and in `to` functions the field values /// are currently ignored.
///   If maping is applied to an enum which variants have field then `to` function ignofield values.
///   `From` function must return default or `None` instead of variant with fields asdon't really know what to provide in those /// fields.
///   I suppose if all variants have the same field we could create function with exparameters but if there are many different
///   types stored in variants then every single one of them would need to be in function signature and that's not reasonable /// thing to do.
///  
/// # Examples
/// Simplest form with default function names
/// ```rust
/// use enum_map::EnumMap;
/// #[derive(EnumMap, Debug, Eq, PartialEq)]
/// enum Example {
///     #[mapstr("variant_1", name="vname")]
///     V1,
///     #[mapstr("variant_2")]
///     V2,
///     #[mapstr("unknown", default)] // Set as default variant
///     Unknown
/// }
/// assert_eq!(Example::V1.to_vname(), "variant_1");
/// assert_eq!(Example::V2.to_vname(), "variant_2");
/// assert_eq!(Example::from_vname("variant_1"), Example::V1);
/// assert_eq!(Example::from_vname("variant_3"), Example::Unknown);
/// ```
/// This example expands to
/// ```rust ignore
/// impl Example {
///     fn to_vname(&self) -> &'static str {
///         match self {
///             Self::V1 => "variant_1",
///             Self::V2 => "variant_2",
///             _ => "unknown"
///         }
///     }
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
/// ```rust
/// use enum_map::EnumMap;
/// #[derive(EnumMap, Debug, Eq, PartialEq)]
/// enum Example {
///     #[mapstr("variant_1", name = "vname")] //
///     #[mapstr("V1", name = "short", default_to="u", default_from=Unknown)] // set defaults
///     #[mapstr("Variant 1", name = "pretty", default_to="PrettyV1")]
///     V1,
/// 
///     // Mapings in the same order as on the first variant
///     #[mapstr("variant_2")] // vname
///     #[mapstr("V2")] // short
///     // ignore `V2` in pretty
///     #[mapstr("VARIANT_2", name = "caps")] // Create new maping ignoring the first variant.
///     V2,
/// 
///     // If `name` is specified, order doesn't matter. If not it must be in the corrct place.
///     #[mapstr("Variant 3", name = "pretty")] // Can be reordered
///     #[mapstr("V3")] // Must be second to be part of "short" maping.
///     #[mapstr("variant_3", name = "vname")] // Can be reordered
///     #[mapstr("VARIANT_3")] // part of "Caps" as that was specified fourth
///     V3,
/// 
///     #[mapstr("unknown", name = "vname", default)] // Set this variant to be default "vname" maping
///     Unknown,
/// 
///     #[mapstr("ERR", name = "caps")]
///     Error
/// }
/// assert_eq!(Example::V1.to_vname(), "variant_1");
/// assert_eq!(Example::Unknown.to_vname(), "unknown");
/// assert_eq!(Example::Error.to_vname(), "unknown");
/// assert_eq!(Example::from_vname("variant_1"), Example::V1);
/// assert_eq!(Example::from_vname("err"), Example::Unknown);
/// assert_eq!(Example::V3.to_pretty(), "Variant 3");
/// assert_eq!(Example::V2.to_pretty(), "PrettyV1");
/// assert_eq!(Example::Unknown.to_pretty(), "PrettyV1");
/// assert_eq!(Example::try_from_pretty("Variant 3"), Some(Example::V3));
/// assert_eq!(Example::try_from_pretty("unknown"), None);
/// ```
#[proc_macro_derive(EnumMap, attributes(mapstr))]
pub fn enum_map(item: TokenStream) -> TokenStream {
    enum_map::enum_map(item)
}
