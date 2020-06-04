# Template trait

## Description

Currently `TemplateOnce::render_once` method consumes the object itself and not useful if you want to re-use the struct.

`Template` trait helps those situation. `Template` trait has `render()` method, which does not consume the object itself.

Like `TemplateOnce`, `Template` trait can be implemented using derive macro.

## Definition

```rust
pub trait Template {
    fn render(&self) -> RenderResult;
}
```

Since `RenderError` can be converted into `fmt::Error`, we can now implement `Display` trait for those structs.

```rust
impl<T: Template> Display for T {
    ...
}
```

## Disadvantage

If you derive this trait, you cannot move out the struct fields. For example, the following template

```html
<% for msg in messages { %><div><%= msg %></div><% } %>
```

will be transformed into the Rust code like

```rust
for msg in self.messages {
    render_text!(_ctx, "<div>");
    render!(_ctx, msg);
    render_text!(_ctx, "</div>");
}
```

which causes an compilation error because `self.messages` cannot be moved.
