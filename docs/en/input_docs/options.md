# Configuration

## Derive options

You can control the rendering behaviour via `template` attribute.

``` rust
#[derive(TemplateSimple)]
#[template(path = "template.stpl", escape = false)]
struct TemplateStruct {
    ...
}
```

`template` attribute accepts the following options.

- `path`: path to template file. Either `path` or `source` is required.
- `source`: inline template source given as a string literal, compiled at compile time. Either `path` or `source` is required, and the two cannot be combined. `include` is not available in inline templates since there is no directory to resolve against.
- `escape`: Enable HTML escaping (default: `true`)
- `delimiter`: Replace the '%' character used for the tag delimiter (default: '%')
- `rm_whitespace`: try to strip whitespaces as much as possible without collapsing HTML structure (default: `false`). This option might not work correctly if your templates have inline `script` tag.
- `rm_newline`: remove `\n` and `\r` characters from literal template text (default: `false`). This does not change values rendered by expressions such as `<%= value %>`.

For small templates you can inline the source instead of pointing to a file.

``` rust
#[derive(TemplateSimple)]
#[template(source = "<div><%= name %></div>", escape = false)]
struct TemplateStruct<'a> {
    name: &'a str,
}
```

You can split the options into multiple `template` attributes.

``` rust
#[derive(TemplateSimple)]
#[template(path = "template.stpl")]
#[template(delimiter = '?')]
#[template(rm_whitespace = true)]
struct TemplateStruct {
    ...
}
```

## Configuration file

Reading configuration files requires the `config` feature, which is enabled by
default. When it is disabled, `sailfish.toml` files are ignored, but derive options
still apply.

Sailfish allows global and local configuration in a file named `sailfish.toml`. Sailfish looks for this file in same directory as `Cargo.toml` and all parent directories.
If, for example, `Cargo.toml` exists in `/foo/bar/baz` directory, then the following configuration files would be scanned in this order.

- `/sailfish.toml`
- `/foo/sailfish.toml`
- `/foo/bar/sailfish.toml`
- `/foo/bar/baz/sailfish.toml`

For `escape`, `delimiter`, and the optimization settings, values in the deeper
directory take precedence over values in ancestor directories. `template_dirs`
lists are combined: directories from deeper configuration files are searched
first, in the order listed in each file.

If a key is specified in both configuration file and derive options, then the value specified in the derive options takes precedence over the configuration file.

### Configuration file format

Configuration files are written in TOML. Here is an example with the default
rendering options and an explicit template directory:

``` toml
template_dirs = ["templates"]
escape = true
delimiter = "%"

[optimizations]
rm_whitespace = false
rm_newline = false
```

Paths in `template_dirs` can be absolute or relative to the configuration file
that defines them. After searching the configured directories, Sailfish falls
back to the `templates` directory next to the crate's `Cargo.toml`.

The `escape` and `delimiter` options have the same meaning as the derive options.
Set `rm_whitespace` and `rm_newline` inside `[optimizations]`. The `path` and
`source` options are only available on `#[template(...)]`.

You can also embed environment variables in `template_dirs` paths by wrapping the variable name with `${` and `}` like `${MY_ENV_VAR}`:

```toml
template_dirs = ["${CI}/path/to/project/${MYVAR}/templates"]
```
