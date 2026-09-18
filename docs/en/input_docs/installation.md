# Installation

Sailfish requires Rust 1.89 or later. Add the following dependency to your `Cargo.toml`:

``` toml
[dependencies]
sailfish = "0.11.8"
```

The default `derive` feature re-exports the derive macros from `sailfish`, so a
separate dependency on `sailfish-macros` is not required.

## Feature Flags

Sailfish accepts the following feature flags:

| Feature | Enabled by default | Description |
| -- | -- | -- |
| `config` | Yes | Read `sailfish.toml` configuration files when compiling templates. When disabled, these files are ignored. |
| `derive` | Yes | Re-export the `TemplateSimple`, `TemplateOnce`, `TemplateMut`, and `Template` derive macros. |
| `perf-inline` | Yes | Add more `#[inline]` attributes. This may improve rendering performance, but generates a slightly larger binary. |
| `json` | No | Enable the `json` filter for values implementing `serde::Serialize`. |
| `hermetic` | No | Compile templates without writing generated template files to disk. Template source files are still read at compile time. |

For example, enable the JSON filter while keeping the default features:

``` toml
[dependencies]
sailfish = { version = "0.11.8", features = ["json"] }
```

To use derive macros without reading configuration files:

``` toml
[dependencies]
sailfish = { version = "0.11.8", default-features = false, features = ["derive", "perf-inline"] }
```

Options set with `#[template(...)]` remain available when `config` is disabled.
