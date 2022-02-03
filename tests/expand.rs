// use enum_maping::EnumMaping;

// #[derive(EnumMaping)]
// enum Ex0 {
//     #[mapstr()]
//     V1,
// }

// #[derive(EnumMaping)]
// enum Ex {
//     #[mapstr("fa")]
//     V1,
// }

// #[derive(EnumMaping)]
// enum Ex2 {
//     #[mapstr(name="fas")]
//     V1,
// }

// #[derive(EnumMaping)]
// enum Ex3 {
//     #[mapstr(name="n", "fa", "faas")]
//     V1,
// }

// #[derive(EnumMaping)]
// enum Ex4 {
//     #[mapstr(name="n", "fa", fas, "fas")]
//     V1,
// }

// #[derive(EnumMaping)]
// enum Ex5 {
//     #[mapstr(name="n", "fa", nasdf="faas")]
//     V1,
// }

// #[derive(EnumMaping)]
// enum Ex6 {
//     #[mapstr( "fa", name=true)]
//     V1,
// }

// #[derive(EnumMaping)]
// enum Ex7 {
//     #[mapstr2(name="n", "fa", name=true)]
//     V1,
// }

// #[derive(EnumMaping)]
// struct Sa {

// }

// #[derive(EnumMaping)]
// enum Example {
//     #[mapstr(name = "vname", "variant_1")]
//     #[mapstr(
//         name = "short",
//         "V1",
//         default_to = "unknown",
//         default_from = "Unknown",
//         try
//     )]
//     #[mapstr(name = "pretty_vname", "Variant 1")]
//     V1,

//     #[mapstr("variant_2")]
//     #[mapstr("V2")]
//     #[mapstr("Variant 2")]
//     V2,

//     #[mapstr(name = "pretty_vname", "Variant 3")]
//     #[mapstr("V3")]
//     #[mapstr(name = "vname", "variant_3")]
//     V3,

//     #[mapstr(name = "vname", default, "unknown")]
//     Unknown,

//     #[mapstr(name = "error", "err")]
//     Error(String),
// }

// fn main() {
//     assert_eq!(Example::V1.to_vname(), "variant_1");
//     assert_eq!(Example::V2.to_vname(), "variant_2");
//     assert_eq!(Example::V3.to_vname(), "variant_3");
//     assert_eq!(Example::Unknown.to_vname(), "unknown");
//     assert_eq!(Example::Error.to_vname(), "unknown");

//     assert_eq!(Example::V1.to_short(), "V1");
//     assert_eq!(Example::V2.to_short(), "V2");
//     assert_eq!(Example::V3.to_short(), "V3");
//     assert_eq!(Example::Unknown.to_short(), "unknown");
//     assert_eq!(Example::Error.to_short(), "unknown");

//     assert_eq!(Example::V1.try_to_pretty_vname(), Some("Variant 1"));
//     assert_eq!(Example::V2.try_to_pretty_vname(), Some("Variant 2"));
//     assert_eq!(Example::V3.try_to_pretty_vname(), Some("Variant 3"));
//     assert_eq!(Example::Unknown.try_to_pretty_vname(), None);
//     assert_eq!(Example::Error.try_to_pretty_vname(), None);

//     assert_eq!(Example::V3.try_to_error(), None);
//     assert_eq!(Example::Error.try_to_error(), Some("err"));

//     assert_eq!(Example::from_vname("variant_1"), Example::V1);
//     assert_eq!(Example::from_vname("variant_2"), Example::V2);
//     assert_eq!(Example::from_vname("variant_3"), Example::V3);
//     assert_eq!(Example::from_vname("unknown"), Example::Unknown);
//     assert_eq!(Example::from_vname("err"), Example::Unknown);
//     assert_eq!(Example::from_vname("random"), Example::Unknown);

//     assert_eq!(Example::from_short("V1"), Example::V1);
//     assert_eq!(Example::from_short("V2"), Example::V2);
//     assert_eq!(Example::from_short("V3"), Example::V3);
//     assert_eq!(Example::from_short("unknown"), Example::Unknown);
//     assert_eq!(Example::from_short("err"), Example::Unknown);
//     assert_eq!(Example::from_short("random"), Example::Unknown);

//     assert_eq!(Example::try_from_pretty_vname("Variant 1"), Some(Example::V1));
//     assert_eq!(Example::try_from_pretty_vname("Variant 2"), Some(Example::V2));
//     assert_eq!(Example::try_from_pretty_vname("Variant 3"), Some(Example::V3));
//     assert_eq!(Example::try_from_pretty_vname("unknown"), None);
//     assert_eq!(Example::try_from_pretty_vname("err"), None);
//     assert_eq!(Example::try_from_pretty_vname("random"), None);

//     assert_eq!(Example::try_from_error("Variant 3"), None);
//     assert_eq!(Example::try_from_error("err"), Some(Example::Error));
//}
