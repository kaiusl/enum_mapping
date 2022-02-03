use enum_maping::EnumMaping;

#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/errors.rs");
}

#[test]
fn basic() {
    #[derive(EnumMaping, Debug, Eq, PartialEq)]
    enum Example {
        #[mapstr(name = "vname", "variant_1")]
        #[mapstr(
            name = "short",
            "V1",
            default_to = "unknown",
            default_from = "Unknown",
            try
        )]
        #[mapstr(name = "pretty_vname", "Variant 1")]
        V1,

        #[mapstr("variant_2")]
        #[mapstr("V2")]
        //#[mapstr("Variant 2")]
        #[mapstr(name = "caps", "VARIANT_2")]
        V2,

        #[mapstr(name = "pretty_vname", "Variant 3")]
        #[mapstr("V3")]
        #[mapstr(name = "vname", "variant_3")]
        #[mapstr("VARIANT_3")]
        V3,

        #[mapstr(name = "vname", default, "unknown")]
        Unknown,

        #[mapstr(name = "error", "err")]
        #[mapstr(name = "caps", "ERR")]
        Error,
    }

    assert_eq!(Example::V1.to_vname(), "variant_1");
    assert_eq!(Example::V2.to_vname(), "variant_2");
    assert_eq!(Example::V3.to_vname(), "variant_3");
    assert_eq!(Example::Unknown.to_vname(), "unknown");
    assert_eq!(Example::Error.to_vname(), "unknown");

    assert_eq!(Example::V1.to_short(), "V1");
    assert_eq!(Example::V2.to_short(), "V2");
    assert_eq!(Example::V3.to_short(), "V3");
    assert_eq!(Example::Unknown.to_short(), "unknown");
    assert_eq!(Example::Error.to_short(), "unknown");

    // assert_eq!(Example::V1.try_to_short(), Some("V1"));
    // assert_eq!(Example::V2.try_to_short(), Some("V2"));
    // assert_eq!(Example::V3.try_to_short(), Some("V3"));
    // assert_eq!(Example::Unknown.try_to_short(), None);
    // assert_eq!(Example::Error.try_to_short(), None);

    // assert_eq!(Example::V1.try_to_pretty_vname(), Some("Variant 1"));
    // assert_eq!(Example::V2.try_to_pretty_vname(), None);
    // assert_eq!(Example::V3.try_to_pretty_vname(), Some("Variant 3"));
    // assert_eq!(Example::Unknown.try_to_pretty_vname(), None);
    // assert_eq!(Example::Error.try_to_pretty_vname(), None);

    // assert_eq!(Example::V1.try_to_caps(), None);
    // assert_eq!(Example::V2.try_to_caps(), Some("VARIANT_2"));
    // assert_eq!(Example::V3.try_to_caps(), Some("VARIANT_3"));
    // assert_eq!(Example::Unknown.try_to_caps(), None);
    // assert_eq!(Example::Error.try_to_caps(), Some("ERR"));

    // assert_eq!(Example::V3.try_to_error(), None);
    // assert_eq!(Example::Error.try_to_error(), Some("err"));

    assert_eq!(Example::from_vname("variant_1"), Example::V1);
    assert_eq!(Example::from_vname("variant_2"), Example::V2);
    assert_eq!(Example::from_vname("variant_3"), Example::V3);
    assert_eq!(Example::from_vname("unknown"), Example::Unknown);
    assert_eq!(Example::from_vname("err"), Example::Unknown);
    assert_eq!(Example::from_vname("random"), Example::Unknown);

    assert_eq!(Example::from_short("V1"), Example::V1);
    assert_eq!(Example::from_short("V2"), Example::V2);
    assert_eq!(Example::from_short("V3"), Example::V3);
    assert_eq!(Example::from_short("unknown"), Example::Unknown);
    assert_eq!(Example::from_short("err"), Example::Unknown);
    assert_eq!(Example::from_short("random"), Example::Unknown);

    assert_eq!(Example::try_from_short("V1"), Some(Example::V1));
    assert_eq!(Example::try_from_short("V2"), Some(Example::V2));
    assert_eq!(Example::try_from_short("V3"), Some(Example::V3));
    assert_eq!(Example::try_from_short("unknown"), None);
    assert_eq!(Example::try_from_short("err"), None);
    assert_eq!(Example::try_from_short("random"), None);

    assert_eq!(
        Example::try_from_pretty_vname("Variant 1"),
        Some(Example::V1)
    );
    assert_eq!(Example::try_from_pretty_vname("Variant 2"), None);
    assert_eq!(
        Example::try_from_pretty_vname("Variant 3"),
        Some(Example::V3)
    );
    assert_eq!(Example::try_from_pretty_vname("unknown"), None);
    assert_eq!(Example::try_from_pretty_vname("err"), None);
    assert_eq!(Example::try_from_pretty_vname("random"), None);

    assert_eq!(Example::try_from_caps("VARIANT_1"), None);
    assert_eq!(Example::try_from_caps("VARIANT_2"), Some(Example::V2));
    assert_eq!(Example::try_from_caps("VARIANT_3"), Some(Example::V3));
    assert_eq!(Example::try_from_caps("unknown"), None);
    assert_eq!(Example::try_from_caps("ERR"), Some(Example::Error));
    assert_eq!(Example::try_from_caps("random"), None);

    assert_eq!(Example::try_from_error("Variant 3"), None);
    assert_eq!(Example::try_from_error("err"), Some(Example::Error));
}
