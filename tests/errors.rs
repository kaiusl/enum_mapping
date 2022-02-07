use enum_map::EnumMap;


#[derive(EnumMap, Debug, Eq, PartialEq)]
enum Example {
    #[mapstr("variant_1", name = "vname")]
    #[mapstr("V1", name = "short", default_to="unknown", default_from=Unknown)]
    #[mapstr("Variant 1", name = "pretty_vname")]
    #[mapstr("VARIANT_1", name = "caps", no_to)]
    V1,

    #[mapstr("variant_2")]
    #[mapstr("V2")]
    #[mapstr("Variant 2")]
    #[mapstr("VARIANT_1", name = "caps2", no_from)]
    V2,

    #[mapstr("Variant 3", name = "pretty_vname")]
    #[mapstr("V3")]
    #[mapstr("variant_3", name = "vname")]
    V3,

    #[mapstr("unknown", name = "vname", default)]
    Unknown,

    #[mapstr("err", name = "error")]
    Error
}

fn main() {
    Example::V1.try_to_caps(); 
    Example::V2.try_from_caps2(); 
} 

#[derive(EnumMap)]
enum Ex0 {
    #[mapstr()]
    V1,
}

#[derive(EnumMap)]
enum Ex {
    #[mapstr("fa")]
    V1,
}

#[derive(EnumMap)]
enum Ex2 {
    #[mapstr(name="fas")]
    V1,
}

#[derive(EnumMap)]
enum Ex3 {
    #[mapstr("fa", name="n", "faas")]
    V1,
}


#[derive(EnumMap)]
enum Ex4 {
    #[mapstr("fa", name="n", fas)]
    V1,
}

#[derive(EnumMap)]
enum Ex5 {
    #[mapstr("fa", name="n",  nasdf="faas")]
    V1,
}

#[derive(EnumMap)]
enum Ex6 {
    #[mapstr("fa", name="n", name=true)]
    V1,
}

#[derive(EnumMap)]
enum Ex7 {
    #[mapstr2("fa", name="n",  name=true)]
    V1,
}

#[derive(EnumMap)]
enum Ex8 {
    #[mapstr("fa", name="n",  default_from="fa")]
    V1,
}

#[derive(EnumMap)]
struct Sa {}