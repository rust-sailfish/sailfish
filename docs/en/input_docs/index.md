# Welcome to Sailfish Documentation!

Sailfish is a simple, small, and extremely fast template engine for Rust. This documentation guides you how to get started with sailfish.

This documentation mainly focuses on concepts of the library, general usage, and template syntax. If you've read this documentation and need more specific information, you might want to read the [sailfish API docs](https://docs.rs/sailfish).

## Why Sailfish ?

There are many libraries for template rendering in Rust. Among those libraries, sailfish aims at **rapid development** and **rapid rendering**. Sailfish has many features that other libraries might not support.

- Write a Rust code directly inside templates, supporting many Rust syntax (struct definition, closure, macro invocation, etc.)
- [Built-in filters](https://docs.rs/sailfish/latest/sailfish/runtime/filter/index.html)
- Minimal dependencies
- Extremely fast (See [benchmarks](https://github.com/djc/template-benchmarks-rs))
- Template rendering is always type-safe because templates are statically compiled.
- Syntax highlighting ([vscode](https://github.com/rust-sailfish/sailfish/tree/main/syntax/vscode), [vim](https://github.com/rust-sailfish/sailfish/tree/main/syntax/vim))
- Consuming, mutable, and shared-reference rendering through [four template traits](getting-started.md#choosing-a-template-trait)

## Upcoming features

You can find proposed features in the repository's [RFCs](https://github.com/rust-sailfish/sailfish/issues?q=is%3Aissue+is%3Aopen+label%3A%22Status%3A+RFC%22). Planned work includes:

- Template inheritance (block, partials, etc.)

If you have any idea about them or want to implement that feature, please send a comment on the issue!

## License

Copyright &copy; 2020 Ryohei Machida

This project is [MIT](https://github.com/rust-sailfish/sailfish/blob/main/LICENSE) licensed
