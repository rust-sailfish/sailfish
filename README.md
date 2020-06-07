<div align="center">

![SailFish](./resources/logo.png)

Simple, small, and extremely fast template engine for Rust

[![Build Status](https://travis-ci.org/Kogia-sima/sailfish.svg?branch=master)](https://travis-ci.org/Kogia-sima/sailfish)
[![Build status](https://ci.appveyor.com/api/projects/status/fa3et4rft4dyvdn9?svg=true)](https://ci.appveyor.com/project/Kogiasima/sailfish)
[![Version](https://img.shields.io/crates/v/sailfish)](https://crates.io/crates/sailfish)
[![docs](https://docs.rs/sailfish/badge.svg)](https://docs.rs/sailfish)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Kogia-sima/sailfish/blob/master/LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square)](http://makeapullrequest.com)

</div>

## ✨ Features

- Simple and intuitive syntax inspired by [EJS](https://ejs.co/)
- Relatively small number of dependencies (<15 crates in total)
- Extremely fast (See [benchmarks](./benches/README.md))
- Better error message
- Template rendering is always type-safe because templates are statically compiled.
- Syntax highlighting support ([vscode](./syntax/vscode), [vim](./syntax/vim))
- Automatically re-compile sources when template file is updated.

:warning: Currentry sailfish is in early-stage development. You can use this library but be sure that there might be some bugs. Also API is still unstable, and thus may changes frequently.

## 🐟 Example

Dependencies:

```toml
[dependencies]
sailfish = "0.0.3"
sailfish-macros = "0.0.3"
```

Template file (templates/hello.stpl):

```html
<DOCTYPE! html>
<html>
  <body>
    <%= content %>
  </body>
</html>
```

Code:

```rust
#[macro_use]
extern crate sailfish_macros;  // enable derive macro

use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "hello.stpl")]
struct Hello {
    content: String
}

fn main() {
    println!("{}", Hello { content: String::from("Hello, world!") }.render_once().unwrap());
}
```

You can find more examples in [examples](./examples) directory.

## 🐾 Roadmap

- `Template` trait ([RFC](https://github.com/Kogia-sima/sailfish/issues/3))
- Template inheritance (block, partials, etc.)
- Whitespace suppressing
- Filters ([RFC](https://github.com/Kogia-sima/sailfish/issues/2))
- Dynamic template compilation ([RFC](https://github.com/Kogia-sima/sailfish/issues/1))
- `format_templates!(fmt, args..)` macro

## 👤 Author

:jp: **Ryohei Machida**

* Github: [@Kogia-sima](https://github.com/Kogia-sima)

## 🤝 Contributing

Contributions, issues and feature requests are welcome!

Feel free to check [issues page](https://github.com/Kogia-sima/sailfish/issues). 

## Show your support

Give a ⭐️ if this project helped you!


## 📝 License

Copyright © 2020 [Ryohei Machida](https://github.com/Kogia-sima).

This project is [MIT](https://github.com/Kogia-sima/sailfish/blob/master/LICENSE) licensed.

***
_This README was generated with ❤️ by [readme-md-generator](https://github.com/kefranabg/readme-md-generator)_
