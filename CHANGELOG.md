## 0.2.4

### Improvements
- Exposed `ResultExt` in crate root; this enables error chaining.

### Breaking Changes
- `ApiKey::new` now takes `Into<String>` rather than requiring a `String`.
- `<ApiKey as FromStr>::Err` is now `Error` instead of `&'static str`.
- Added new `ErrorKind` variant: `ApiKeyParseError`.