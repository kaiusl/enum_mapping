# Roadmap
- [ ] Add option to derive some traits like
    - [ ] Display
    - [ ] [Try_]From<&'static str> for Self
    - [ ] [Try_]From<Self> for &'static str
    - [ ] AsRef<&'static str>
- [ ] Use default field values to construct variants from string.
- [ ] Use field values on to_ methods. For example #[mapstr("string_{0}")] where {0} would be replaced by the first field value. For struct like fields we could do {x} where x is field name. 
- [ ] Reverse construction of above.