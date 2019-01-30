## 0.2.7

### Improvements
- Added `FromStr` impl for `QueryTime`

## 0.2.6

### Improvements
- Added `danger_new_unverified` to `Client` to support `--insecure` flag being exposed by other tools.

## 0.2.5

### Improvements
- Added `activitymap` module for loading activity map edges
- Added `QueryTime` to safely work with possibly-unitized time values for metrics and records requests

## 0.2.4

### Improvements
- Exposed `ResultExt` in crate root; this enables error chaining.

### Breaking Changes
- Moved to latest versions of `serde`, `reqwest`, and `error-chain`.
- `ApiKey::new` now takes `Into<String>` rather than requiring a `String`.
- `<ApiKey as FromStr>::Err` is now `Error` instead of `&'static str`.
- Added new `ErrorKind` variant: `ApiKeyParseError`.