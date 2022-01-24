use std::{num::ParseIntError, slice::SliceIndex};

use enum_maping::*;
use syn::ext::IdentExt;

#[repr(u8)]
#[derive(EnumMap)]
pub enum Tracks {
    #[mapstr(fn_name = "kunos_id", "BMW")]
    #[mapstr(fn_name = "kunos_id2", "BMW2")]
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

}