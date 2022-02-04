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

use proc_macro::TokenStream;
mod enum_map;

/// Macro to derive custom mapings for enum types.
///
/// It provides function implementations for `to` and `from` functions for enum.
/// Maping is specified on enum variant by attribute `#[mapstr(..)]`. 
///
/// Multiple mappings can be specified for single variant.
/// If `name` is provided, the order doesn't matter.
/// If it is not then that `mapstr` must be at the same position as it first appeared. See [Examples](#examples) below.
///
/// By default this macro will create two functions 
/// * `fn try_to_<fname>(&self) -> Option<&'static str`>,
/// * `fn try_from_<fname>(s: &str) -> Option<Self>`.
/// 
/// If defaults are set the created functions are 
/// * `fn to_<fname>(&self) -> &'static str`,
/// * `fn from_<fname>(s: &str) -> Self`.
/// 
/// First set of functions can still be then created be passing argument `try` to the `mapstr` attribute.
///
/// # Variant attributes
/// * `mapstr(<value>, name="..", [default, default_to="..", default_from="..", try, no_to, no_from])`
///     - `value`: string - string to map to
///     - `name`: string - set created function name as `[try]_to_<name>` and `[try]_from_<fame>`. Must be set on first variant part of the maping.
///     - `default` - set variant as default. Optional. If set resulting functions will return directly `&str` and `Self` and remove "try" from the name.
///     - `default_to`: string - set default string to map to. Optional. If set resulting function will return directly `&str` and remove "try" from the function name.
///     - `default_from`: string - set default variant to map to. Optional. If set resulting function will return directly `Self` and remove "try" from the function name.
///     - `try` - if set create functions returning [`Option`](_) even if defaults are set. 
///     - `no_to` - if set don't create `to` methods.
///     - `no_from` - if set don't create `from` methods.
///
/// Optional arguments can be specified on any of the variants but only the first specification is used.
///
/// # Current shortcomings
/// * Variants with fields have limited support. They cannot be created with `from` functions and in `to` functions the field values are currently ignored. 
/// 
///   If maping is applied to an enum which variants have field then `to` function ignores field values.
///   `From` function must return default or `None` instead of variant with fields as we don't really know what to provide in those fields.
/// 
///   I suppose if all variants have the same field we could create function with extra parameters but if there are many different
///   types stored in variants then every single one of them would need to be in the function signature and that's not reasonable thing to do.
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
///     #[mapstr(name = "short", "V1", default_to="u", default_from="Unknown")] // Set defaults
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
///     #[mapstr("V3")] // Must be second to be part of "short" maping.
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
/// ```
#[proc_macro_derive(EnumMaping, attributes(mapstr))]
pub fn enum_map(item: TokenStream) -> TokenStream {
    enum_map::enum_map(item)
}
