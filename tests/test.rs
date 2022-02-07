use enum_map::EnumMap;

#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/errors.rs");
}

#[test]
fn simple() {
    #[derive(EnumMap, Debug, Eq, PartialEq)]
    enum E {
        #[mapstr("variant_1", name = "vname")]
        V1,

        #[mapstr("variant_2")]
        V2,

        #[mapstr("unknown", name = "vname", default)]
        Unknown,
    }

    assert_eq!(E::V1.to_vname(), "variant_1");
    assert_eq!(E::V2.to_vname(), "variant_2");
    assert_eq!(E::Unknown.to_vname(), "unknown");

    assert_eq!(E::from_vname("variant_1"), E::V1);
    assert_eq!(E::from_vname("variant_2"), E::V2);
    assert_eq!(E::from_vname("unknown"), E::Unknown);
    assert_eq!(E::from_vname("err"), E::Unknown);
    assert_eq!(E::from_vname("random"), E::Unknown);
}

#[test]
fn basic() {
    #[derive(EnumMap, Debug, Eq, PartialEq)]
    enum Example {
        #[mapstr("variant_1", name = "vname")]
        #[mapstr("V1", name = "short", default_to="u", default_from=Unknown, r#try)]
        #[mapstr("Variant 1", name = "pretty_vname")]
        V1,

        #[mapstr("variant_2")]
        #[mapstr("V2")]
        //#[mapstr("Variant 2")]
        #[mapstr("VARIANT_2", name = "caps")]
        V2,

        #[mapstr("Variant 3", name = "pretty_vname")]
        #[mapstr("V3", default)]
        #[mapstr("variant_3", name = "vname")]
        #[mapstr("VARIANT_3")]
        V3,

        #[mapstr("unknown", name = "vname", default)]
        Unknown,

        #[mapstr("err", name = "error")]
        #[mapstr("ERR", name = "caps")]
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
    assert_eq!(Example::Unknown.to_short(), "u");
    assert_eq!(Example::Error.to_short(), "u");

    assert_eq!(Example::V1.try_to_short(), Some("V1"));
    assert_eq!(Example::V2.try_to_short(), Some("V2"));
    assert_eq!(Example::V3.try_to_short(), Some("V3"));
    assert_eq!(Example::Unknown.try_to_short(), None);
    assert_eq!(Example::Error.try_to_short(), None);

    assert_eq!(Example::V1.try_to_pretty_vname(), Some("Variant 1"));
    assert_eq!(Example::V2.try_to_pretty_vname(), None);
    assert_eq!(Example::V3.try_to_pretty_vname(), Some("Variant 3"));
    assert_eq!(Example::Unknown.try_to_pretty_vname(), None);
    assert_eq!(Example::Error.try_to_pretty_vname(), None);

    assert_eq!(Example::V1.try_to_caps(), None);
    assert_eq!(Example::V2.try_to_caps(), Some("VARIANT_2"));
    assert_eq!(Example::V3.try_to_caps(), Some("VARIANT_3"));
    assert_eq!(Example::Unknown.try_to_caps(), None);
    assert_eq!(Example::Error.try_to_caps(), Some("ERR"));

    assert_eq!(Example::V3.try_to_error(), None);
    assert_eq!(Example::Error.try_to_error(), Some("err"));

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

#[test]
fn multi_default() {
    #[derive(EnumMap, Debug, Eq, PartialEq)]
    enum E {
        #[mapstr("variant_1", name = "dv2", default_to = "error", default_from = Error)]
        #[mapstr("variant_1", name = "dv2_t", default_to = "error2")]
        #[mapstr("variant_1", name = "dv2_f",  default_from = Error)]
        V1,

        #[mapstr("variant_2")]
        #[mapstr("variant_2", default_to = "error3")] // default_to should be ignored
        #[mapstr("variant_2", default_from=Unknown)] // default_from should be ignored
        V2,

        #[mapstr("unknown", name = "dv2", default)]
        #[mapstr("unknown", name = "dv2_t", default)]
        #[mapstr("unknown", name = "dv2_f", default)]
        Unknown,

        Error,
    }

    assert_eq!(E::V1.to_dv2(), "variant_1");
    assert_eq!(E::V2.to_dv2(), "variant_2");
    assert_eq!(E::Unknown.to_dv2(), "unknown");
    assert_eq!(E::Error.to_dv2(), "error");

    assert_eq!(E::from_dv2("variant_1"), E::V1);
    assert_eq!(E::from_dv2("variant_2"), E::V2);
    assert_eq!(E::from_dv2("unknown"), E::Unknown);
    assert_eq!(E::from_dv2("err"), E::Error);
    assert_eq!(E::from_dv2("random"), E::Error);

    assert_eq!(E::V1.to_dv2_t(), "variant_1");
    assert_eq!(E::V2.to_dv2_t(), "variant_2");
    assert_eq!(E::Unknown.to_dv2_t(), "unknown");
    assert_eq!(E::Error.to_dv2_t(), "error2");

    assert_eq!(E::from_dv2_t("variant_1"), E::V1);
    assert_eq!(E::from_dv2_t("variant_2"), E::V2);
    assert_eq!(E::from_dv2_t("unknown"), E::Unknown);
    assert_eq!(E::from_dv2_t("err"), E::Unknown);
    assert_eq!(E::from_dv2_t("random"), E::Unknown);

    assert_eq!(E::V1.to_dv2_f(), "variant_1");
    assert_eq!(E::V2.to_dv2_f(), "variant_2");
    assert_eq!(E::Unknown.to_dv2_f(), "unknown");
    assert_eq!(E::Error.to_dv2_f(), "unknown");

    assert_eq!(E::from_dv2_f("variant_1"), E::V1);
    assert_eq!(E::from_dv2_f("variant_2"), E::V2);
    assert_eq!(E::from_dv2_f("unknown"), E::Unknown);
    assert_eq!(E::from_dv2_f("err"), E::Error);
    assert_eq!(E::from_dv2_f("random"), E::Error);
}

#[test]
fn display_default() {
    #[derive(EnumMap, Debug, Eq, PartialEq)]
    enum E {
        #[mapstr("variant_1", name = "vname", display)]
        V1,

        #[mapstr("variant_2")]
        V2,

        #[mapstr("unknown", name = "vname", default)]
        Unknown,

        Err,
    }

    assert_eq!(E::V1.to_vname(), "variant_1");
    assert_eq!(E::V2.to_vname(), "variant_2");
    assert_eq!(E::Unknown.to_vname(), "unknown");
    assert_eq!(E::Err.to_vname(), "unknown");

    assert_eq!(E::from_vname("variant_1"), E::V1);
    assert_eq!(E::from_vname("variant_2"), E::V2);
    assert_eq!(E::from_vname("unknown"), E::Unknown);
    assert_eq!(E::from_vname("err"), E::Unknown);
    assert_eq!(E::from_vname("random"), E::Unknown);

    assert_eq!(format!("{}", E::V1), String::from("variant_1"));
    assert_eq!(format!("{}", E::V2), String::from("variant_2"));
    assert_eq!(format!("{}", E::Unknown), String::from("unknown"));
    assert_eq!(format!("{}", E::Err), String::from("unknown"));
}

#[test]
fn display_no_default() {
    #[derive(EnumMap, Debug, Eq, PartialEq)]
    enum E {
        #[mapstr("variant_1", name = "vname", display)]
        V1,

        #[mapstr("variant_2")]
        V2,

        #[mapstr("unknown", name = "vname")]
        Unknown,

        Err,
    }

    assert_eq!(E::V1.try_to_vname(), Some("variant_1"));
    assert_eq!(E::V2.try_to_vname(), Some("variant_2"));
    assert_eq!(E::Unknown.try_to_vname(), Some("unknown"));
    assert_eq!(E::Err.try_to_vname(), None);

    assert_eq!(E::try_from_vname("variant_1"), Some(E::V1));
    assert_eq!(E::try_from_vname("variant_2"), Some(E::V2));
    assert_eq!(E::try_from_vname("unknown"), Some(E::Unknown));
    assert_eq!(E::try_from_vname("err"), None);
    assert_eq!(E::try_from_vname("random"), None);

    assert_eq!(format!("{}", E::V1), String::from("variant_1"));
    assert_eq!(format!("{}", E::V2), String::from("variant_2"));
    assert_eq!(format!("{}", E::Unknown), String::from("unknown"));
    assert_eq!(format!("{}", E::Err), String::from("Unknown variant"));
}
