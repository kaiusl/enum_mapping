use std::{num::ParseIntError, slice::SliceIndex};

use enum_maping::*;
use syn::ext::IdentExt;

#[repr(u8)]
#[derive(EnumMaping)]
pub enum Tracks {
    #[mapstr(fname = "kunos_id", "BMW")]
    #[mapstr("BMW2")]
    Bmw = 0,
    #[mapstr("Audi")]
    #[mapstr("Audi2")]
    Audi,
    #[mapstr("Mercedes")]
    #[mapstr("Mercedes2")]
    Mercedes
}

impl Default for Tracks {
    fn default() -> Self {
        Self::Bmw
    }
}



#[test]
fn main() {

    Tracks::from_mappedstr("BMW");
}

#[derive(EnumMaping, Debug, Eq, PartialEq)]
enum Example {
    #[mapstr(fname = "vname", "variant_1")] // Specify custom fn name "to/from_vname"
    #[mapstr("V1")] // Use default
    #[mapstr(fname = "pretty_vname", "Variant 1")] // "to/from_pretty_vname"
    V1,

    #[mapstr("variant_2")] // for vname
    #[mapstr("V2")]
    #[mapstr("Variant 2")] // for default 
    V2,

    #[mapstr(fname = "pretty_vname", "Variant 3")] // for pretty_vname
    #[mapstr("V3")] // must be second #[mapstr(..)] here to match with default mapping
    #[mapstr(fname = "vname", "variant_3")] // for vname
    V3,

    #[mapstr("unknown")]
    #[mapstr("unknown")]
    #[mapstr("unknown")]
    Unknown,
}

impl Default for Example {
    fn default() -> Self {
        Self::Unknown
    }
}

#[test]
fn main2() {
    assert_eq!(Example::V1.to_vname(), "variant_1");
    assert_eq!(Example::V2.to_vname(), "variant_2");
    assert_eq!(Example::V3.to_vname(), "variant_3");
    assert_eq!(Example::V1.to_mappedstr(), "V1");
    assert_eq!(Example::V2.to_mappedstr(), "V2");
    assert_eq!(Example::V3.to_mappedstr(), "V3");
    assert_eq!(Example::V1.to_pretty_vname(), "Variant 1");
    assert_eq!(Example::V2.to_pretty_vname(), "Variant 2");
    assert_eq!(Example::V3.to_pretty_vname(), "Variant 3");

}