# Installation

In order to use sailfish templates, you have add two dependencies in your `Cargo.toml`.

``` toml
[dependencies]
sailfish = "0.9.0"
```

## Feature Flags

Sailfish accepts the following feature flags

|Feature|Description|
|--|--|
|derive|enable derive macros (enabled by default)|
|json|enable `json` filter|
|perf-inline|Add more `#[inline]` attributes. This may improve rendering performance, but generates a bit larger binary (enabled by default)|
