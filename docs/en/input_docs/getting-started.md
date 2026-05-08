# Getting Started

## Prepare the template file

Create a new directory named `templates` in the same directory as `Cargo.toml`. Copy the following contents and paste it to a new file named `templates/hello.stpl`.

``` rhtml
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

<ol><li>Import the sailfish crates:</li></ol>

```rust
use sailfish::TemplateSimple;
```

<ol start="2"><li>Define the template struct to be rendered:</li></ol>

```rust
#[derive(TemplateSimple)]  // automatically implement `TemplateSimple` trait
#[template(path = "hello.stpl")]  // specify the path to template
struct HelloTemplate {
    // data to be passed to the template
    messages: Vec<String>,
}
```

<ol start="3"><li>Render the data with <code>render_once()</code> method.</li></ol>

```rust
fn main() {
    let ctx = HelloTemplate {
        messages: vec![String::from("foo"), String::from("bar")],
    };

    // Now render templates with given data
    println!("{}", ctx.render_once().unwrap());
}
```

That's it!

You can find more examples in the [example](https://github.com/rust-sailfish/sailfish/tree/master/examples) directory in the sailfish repository.
