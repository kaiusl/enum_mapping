# #[derive(EnumMaping)]

[![Rust](https://github.com/kaiusl/enum_mapping/actions/workflows/rust.yml/badge.svg)](https://github.com/kaiusl/enum_mapping/actions/workflows/rust.yml)

Quick enum mappings to strings

This crate provides a derive macro `#[derive(EnumMaping)]` to quickly create mappings between enum variants and strings.

For example instead of writing
```rust
enum Example {
    V1,
    V2,
    Unknown
}

impl Example {
    fn to_vname(&self) -> &'static str {
        match self {
            Self::V1 => "variant_1",
            Self::V2 => "variant_2",
            _ => "unknown"
        }
    }
    fn from_vname(s: &str) -> Self {
        match s {
            s if s == "variant_1" => Self::V1,
            s if s == "variant_2"  => Self::V2,
            _ => Self::Unknown   
        }
    }
}
```
you can do
```rust
use enum_maping::EnumMaping;

#[derive(EnumMaping)]
enum Example {
    #[mapstr(name="vname", "variant_1")]
    V1,
    #[mapstr("variant_2")]
    V2,
    #[mapstr("unknown", default)]
    Unknown
}
```
