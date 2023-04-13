<div align="center">

![SailFish](./resources/logo.png)

Simple, small, and extremely fast template engine for Rust

[![Tests](https://github.com/rust-sailfish/sailfish/workflows/Tests/badge.svg)](https://github.com/rust-sailfish/sailfish/actions?query=workflow%3ATests)
[![Version](https://img.shields.io/crates/v/sailfish)](https://crates.io/crates/sailfish)
[![dependency status](https://deps.rs/repo/github/rust-sailfish/sailfish/status.svg)](https://deps.rs/repo/github/rust-sailfish/sailfish)
[![Rust 1.60](https://img.shields.io/badge/rust-1.60+-lightgray.svg)](https://blog.rust-lang.org/2022/04/07/Rust-1.60.0.html)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/rust-sailfish/sailfish/blob/master/LICENSE)

[User Guide](https://rust-sailfish.github.io/sailfish/) | [API Docs](https://docs.rs/sailfish) | [Examples](./examples)

</div>

## ✨ Features

- Simple and intuitive syntax inspired by [EJS](https://ejs.co/)
- Include another template file inside template
- Built-in filters
- Minimal dependencies (<15 crates in total)
- Extremely fast (See [benchmarks](https://github.com/djc/template-benchmarks-rs))
- Better error message
- Syntax highlighting support ([vscode](./syntax/vscode), [vim](./syntax/vim))
- Works on Rust 1.60 or later

## 🐟 Example

Dependencies:

```toml
[dependencies]
sailfish = "0.6.1"
```

Template file (templates/hello.stpl):

```erb
<html>
  <body>
    <% for msg in &messages { %>
      <div><%= msg %></div>
    <% } %>
  </body>
</html>
```

Code:

```rust
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "hello.stpl")]
struct HelloTemplate {
    messages: Vec<String>
}

fn main() {
    let ctx = HelloTemplate {
        messages: vec![String::from("foo"), String::from("bar")],
    };
    println!("{}", ctx.render_once().unwrap());
}
```

You can find more examples in [examples](./examples) directory.

## 🐾 Roadmap

- `Template` trait ([RFC](https://github.com/rust-sailfish/sailfish/issues/3))
- Template inheritance (block, partials, etc.)

## 👤 Author

🇯🇵 **Ryohei Machida**

* GitHub: [@Kogia-sima](https://github.com/Kogia-sima)

## 🤝 Contributing

Contributions, issues and feature requests are welcome!

Since sailfish is an immature library, there are many [planned features](https://github.com/rust-sailfish/sailfish/labels/Type%3A%20RFC) that is on a stage of RFC. Please leave a comment if you have an idea about its design!

Also I welcome any pull requests to improve sailfish! Find issues with [Status: PR Welcome](https://github.com/rust-sailfish/sailfish/issues?q=is%3Aissue+is%3Aopen+label%3A%22Status%3A+PR+Welcome%22) label, and [let's create a new pull request](https://github.com/rust-sailfish/sailfish/pulls)!

## Show your support

Give a ⭐️ if this project helped you!

## 📝 License

Copyright © 2020 [Ryohei Machida](https://github.com/Kogia-sima).

This project is [MIT](https://github.com/rust-sailfish/sailfish/blob/master/LICENSE) licensed.

***
_This README was generated with ❤️ by [readme-md-generator](https://github.com/kefranabg/readme-md-generator)_
