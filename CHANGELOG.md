<a name="v0.4.0"></a>
## [v0.4.0](https://github.com/rust-sailfish/sailfish/compare/v0.3.4...v0.4.0) (2022-03-10)

## Fix

* Fix some issues pointing to the wrong documentation links

##Breaking Change

* Changed format to .TOML instead of .yaml to better match what Rust uses.

<a name="v0.3.4"></a>
## [v0.3.4](https://github.com/rust-sailfish/sailfish/compare/v0.3.3...v0.3.4) (2021-02-13)

## Fix

* Update some dependencies

<a name="v0.3.3"></a>
## [v0.3.3](https://github.com/rust-sailfish/sailfish/compare/v0.3.2...v0.3.3) (2021-04-06)

## Fix

* Improve error message for missing semicolon in code blocks

<a name="v0.3.2"></a>
## [v0.3.2](https://github.com/rust-sailfish/sailfish/compare/v0.3.1...v0.3.2) (2021-03-29)

## Fix

* Avoid sable/nightly mismatch error caused by proc-macro2 crate 

<a name="v0.3.1"></a>
## [v0.3.1](https://github.com/rust-sailfish/sailfish/compare/v0.3.0...v0.3.1) (2021-01-23)

## New Features

* Allow unsized types for filters

## Fix

* Workaround for incorrect cargo fingerprints

<a name="v0.3.0"></a>
## [v0.3.0](https://github.com/rust-sailfish/sailfish/compare/v0.2.2...v0.3.0) (2020-12-20)

## Breaking changes

* No longer requires `extern crate sailfish_macros` (which raise compilation warnings with v0.3.0)
* Remove `TemplaceOnce::render_to_string` method (already deprecated in v0.2.1)
* Forbid implementing `TemplateOnce` trait by yourself
* Change `RenderError` into enum
* Update error format in `sailfish-compiler`

## New features

* New filters: `json`, `truncate`
* Impl `Send`/`Sync` for `Buffer`

## Fix

* Fix rendering issue on continue/break statements
* Do not panic when buffer size decreased
* Remove unsafe usage of `ptr::add()`
* Properly handle slices with size greater than `isize::MAX`

<a name="v0.2.3"></a>
## [v0.2.3](https://github.com/rust-sailfish/sailfish/compare/v0.2.2...v0.2.3) (2020-11-29)

## Fix

* Use `std::result::Result` in derive macro to allow custom Result types (#34)

<a name="v0.2.2"></a>
## [v0.2.2](https://github.com/rust-sailfish/sailfish/compare/v0.2.1...v0.2.2) (2020-11-11)

## Fix

* Update proc-macro2 version (#32)

<a name="v0.2.1"></a>
## [v0.2.1](https://github.com/rust-sailfish/sailfish/compare/v0.2.0...v0.2.1) (2020-08-04)

### Features

* Add trim filter

### Fix

* Fix incorrect syntax highlighting in vim
* Avoid capacity overflow in `Buffer::with_capacity`
* Avoid dangerous conversion from String to Buffer
* Fix docs typo (#30)
* Search rustfmt command along all toolchains

<a name="v0.2.0"></a>
## [v0.2.0](https://github.com/rust-sailfish/sailfish/compare/v0.1.3...v0.2.0) (2020-07-17)

### Breaking Changes

* Remove Buffer::set_len method
* Syntactically disallow invalid filter expression
* remove runtime::Context API
* Remove register_escape_fn API

### Features

* Implement Compiler::compile_str() function
* Implement filters
