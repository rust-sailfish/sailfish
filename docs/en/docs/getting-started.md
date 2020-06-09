# Getting Started

## Prepare the template file

Create a new directory named `templates` in the same directory as `Cargo.toml`. Copy the following contents and paste it to a new file named `templates/hello.stpl`.

```ejs
<html>
  <body>
    <% for msg in &messages { %>
      <div><%= msg %></div>
    <% } %>
  </body>
</html>
```

Now your project structure should be like this:

```text
Cargo.toml
src/
    (Source files)
templates/
    hello.stpl
```

## Render the template

Import the sailfish crates:

```rust
#[macro_use]
extern crate sailfish_macros;  // enable derive macros

use sailfish::TemplateOnce;  // import `TemplateOnce` trait
```

Define the template struct to be rendered:

```rust
#[derive(TemplateOnce)]  // automatically implement `TemplateOnce` trait
#[template(path = "hello.stpl")]  // specify the path to template
struct HelloTemplate<'a> {
    // data to be passed to the template
    messages: &'a [String],
}
```

And render the data with `render_once()` method.

```rust
fn main() {
    let ctx = HelloTemplate {
        messages: &[String::from("foo"), String::from("bar")];
    }

    // Now render templates with given data
    println!("{}", ctx.render_once().unwrap());
}
```

That's it!

You can find more examples in the [example](https://github.com/Kogia-sima/sailfish/tree/master/examples) directory in the sailfish repository.
