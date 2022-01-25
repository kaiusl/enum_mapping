use enum_maping::EnumMaping;


#[derive(EnumMaping, Debug, Eq, PartialEq)]
enum Example {
    #[mapstr(name = "vname", "variant_1")]
    #[mapstr(name = "short", "V1", default_to="unknown", default_from="Unknown")]
    #[mapstr(name = "pretty_vname", "Variant 1")]
    #[mapstr(name = "caps", "VARIANT_1", to=false)]
    V1,

    #[mapstr("variant_2")]
    #[mapstr("V2")]
    #[mapstr("Variant 2")]
    #[mapstr(name = "caps2", "VARIANT_1", from=false)]
    V2,

    #[mapstr(name = "pretty_vname", "Variant 3")]
    #[mapstr("V3")]
    #[mapstr(name = "vname", "variant_3")]
    V3,

    #[mapstr(name = "vname", default, "unknown")]
    Unknown,

    #[mapstr(name = "error", "err")]
    Error
}

fn main() {
    Example::V1.try_to_caps(); 
    Example::V2.try_from_caps2(); 
} 

#[derive(EnumMaping)]
enum Ex0 {
    #[mapstr()]
    V1,
}

#[derive(EnumMaping)]
enum Ex {
    #[mapstr("fa")]
    V1,
}

#[derive(EnumMaping)]
enum Ex2 {
    #[mapstr(name="fas")]
    V1,
}

#[derive(EnumMaping)]
enum Ex3 {
    #[mapstr(name="n", "fa", "faas")]
    V1,
}


#[derive(EnumMaping)]
enum Ex4 {
    #[mapstr(name="n", "fa", fas)]
    V1,
}

#[derive(EnumMaping)]
enum Ex5 {
    #[mapstr(name="n", "fa", nasdf="faas")]
    V1,
}

#[derive(EnumMaping)]
enum Ex6 {
    #[mapstr(name="n", "fa", name=true)]
    V1,
}

#[derive(EnumMaping)]
enum Ex7 {
    #[mapstr2(name="n", "fa", name=true)]
    V1,
}



#[derive(EnumMaping)]
struct Sa {

}