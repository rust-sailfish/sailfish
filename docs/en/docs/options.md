# Derive Options

You can control the rendering behaviour via `template` attribute.

```rust
#[derive(TemplateOnce)]
#[template(path = "template.stpl", escape = false)]
struct TemplateStruct {
    ...
}
```

`template` attribute accepts the following options.

- `path`: path to template file. This options is always required.
- `escape`: Enable HTML escaping (default: `false`)
- `delimiter`: Replace the '%' character used for the tag delimiter (default: '%')
- `rm_whitespace`: try to strip whitespaces as much as possible without collapsing HTML structure (default: `false`). This option might not work correctly if your templates have inline `script` tag.

You can split the options into multiple `template` attributes.

```rust
#[derive(TemplateOnce)]
#[template(path = "template.stpl")]
#[template(delimiter = '?')]
#[template(rm_whitespace = true)]
struct TemplateStruct {
    ...
}
```
