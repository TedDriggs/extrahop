# Extrahop REST API Client
Rust tools for working with the ExtraHop REST API.

This library is not an exhaustive strongly-typed client for the API; using that model is not recommended
as it may lead to breakages during deserialization that don't impact your code. Instead, the library
provides utilities which should be used in concert with structs defined in consuming libraries to make
request and response handling easier.

# Examples
```rust
use extrahop;
let client = Client::new("extrahop", ApiKey::new("YOUR_KEY"));
let rsp = client.get("dashboards").send();
// handle a normal reqwest response.
```