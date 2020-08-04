
<a name="v0.2.1"></a>
## [v0.2.1](https://github.com/Kogia-sima/sailfish/compare/v0.2.0...v0.2.1) (2020-07-17)

### Features

* Add trim filter

### Bug fix

* Fix incorrect syntax highlighting in vim
* Avoid capacity overflow in `Buffer::with_capacity`
* Avoid dangerous conversion from String to Buffer
* Fix docs typo (#30)
* Search rustfmt command along all toolchains

<a name="v0.2.0"></a>
## [v0.2.0](https://github.com/Kogia-sima/sailfish/compare/v0.1.3...v0.2.0) (2020-07-17)

### Breaking Changes

* Remove Buffer::set_len method
* Syntactically disallow invalid filter expression
* remove runtime::Context API
* Remove register_escape_fn API

### Features

* Implement Compiler::compile_str() function
* Implement filters
