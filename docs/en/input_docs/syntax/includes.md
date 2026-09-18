# Includes

You can also include another template inside templates using `include!` macro.

Consider the following example.

- `templates/header.stpl`

``` html
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
<meta name="format-detection" content="telephone=no">
<link rel="icon" type="image/x-icon" href="favicon.ico">
```

- `templates/index.stpl`

``` rhtml
<html>
  <head>
    <% include!("./header.stpl"); %>
  </head>
  <body>
    Main contents
  </body>
</html>
```

Then you can see the `header.stpl` is embedded in the output.

``` html
<html>
  <head>
    <meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
<meta name="format-detection" content="telephone=no">
<link rel="icon" type="image/x-icon" href="favicon.ico">
  </head>
  <body>
    Main contents
  </body>
</html>
```

Like [`std::include!`](https://doc.rust-lang.org/std/macro.include.html) macro in Rust, the provided path is interpreted as a relative path to the current template file.

Use `/` as the path separator for templates shared between Unix and Windows.
Windows also accepts `\`, which must be escaped as `\\` in a regular Rust string
literal.

Includes require a file-backed template (`#[template(path = "...")]`). They are
not available in inline templates using `#[template(source = "...")]`, which have
no template directory to resolve paths against.
