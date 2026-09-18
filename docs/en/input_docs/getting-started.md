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

<ol><li>Import the template trait and derive macro:</li></ol>

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

You can find more examples in the [examples](https://github.com/rust-sailfish/sailfish/tree/main/examples) directory in the sailfish repository.

## Choosing a template trait

The derive macro determines how a template accesses its fields and whether
rendering consumes or borrows the context:

| Trait | Field access in the template | Rendering method | Context ownership |
| -- | -- | -- | -- |
| `TemplateSimple` | `messages` | `render_once(self)` | Consumes the context; fields become local variables. |
| `TemplateOnce` | `self.messages` | `render_once(self)` | Consumes the context; fields and methods are accessed through `self`. |
| `TemplateMut` | `self.messages` | `render_mut(&mut self)` | Mutably borrows the context, allowing repeated rendering and mutation. |
| `Template` | `self.messages` | `render(&self)` | Borrows the context through a shared reference, allowing repeated rendering. |

Deriving `TemplateMut` also implements `TemplateOnce`. Deriving `Template` also
implements `TemplateMut` and `TemplateOnce`. `TemplateSimple` is a separate trait.
Import the trait that provides the method you call.

When switching the example above to `#[derive(Template)]`, import
`sailfish::Template`, change the loop to `for msg in &self.messages`, and call
`ctx.render()`. You can then render the same context more than once.

Each trait also has a method that appends to a `sailfish::runtime::Buffer`:
`render_once_to`, `render_mut_to`, or `render_to`, with the same ownership rules as
the corresponding method returning a string.

## Handling rendering errors

Rendering methods return a `Result`. Errors can come from custom `Render`
implementations, filters such as `json` or `disp`, nested templates, or explicit
error returns in template code. The examples use `unwrap()` for brevity; in an
application, handle the error or propagate it with `?`.

If a method that appends to a buffer returns an error, the buffer may contain
partial output. Rendering does not automatically roll back changes to the buffer.
