# Welcome to Sailfish Documentation!

Sailfish is a simple, small, and extremely fast template engine for Rust. This documentation guides you how to get started with sailfish.

This documentation mainly focuses on concepts of the library, general usage, and template syntax. If you've read this documentation and need more specific information, you might want to read the [sailfish API docs](https://docs.rs/sailfish).

## Why Sailfish ?

There are many libraries for template rendering in Rust. Among those libraries, sailfish aims at **rapid development** and **rapid rendering**. Sailfish has many features that other libraries might not support.

- Write a Rust code directly inside templates, supporting many Rust syntax (struct definition, closure, macro invocation, etc.)
- [Built-in filters](https://docs.rs/sailfish/latest/sailfish/runtime/filter/index.html)
- Minimal dependencies (<15 crates in total)
- Extremely fast (See [benchmarks](https://github.com/djc/template-benchmarks-rs))
- Template rendering is always type-safe because templates are statically compiled.
- Syntax highlighting ([vscode](http://github.com/rust-sailfish/sailfish/blob/master/syntax/vscode), [vim](http://github.com/rust-sailfish/sailfish/blob/master/syntax/vim))

## Upcoming features

Since sailfish is on early stage of development, there are many upcoming features that is not supported yet. You can find many [RFC](https://github.com/rust-sailfish/sailfish/issues?q=is%3Aissue+is%3Aopen+label%3A%22Status%3A+RFC%22)s in my repository. These RFC include:

- `Template` trait (which does not consume itself)
- Template inheritance (block, partials, etc.)

If you have any idea about them or want to implement that feature, please send a comment on the issue!

## License

Copyright &copy; 2020 Ryohei Machida

This project is [MIT](https://github.com/rust-sailfish/sailfish/blob/master/LICENSE) licensed
