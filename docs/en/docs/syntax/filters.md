# Filters

Filters are used to format the rendered contents.

Example:

=== "Template"

    ``` rhtml
    message: <%= "foo\nbar" | dbg %>
    ```

=== "Result"

    ``` html
    message: &quot;foo\nbar&quot;
    ```

!!! Note
    Since `dbg` filter accepts `<T: std::fmt::Debug>` types, that type isn't required to implement [`Render`](https://docs.rs/sailfish/latest/sailfish/runtime/trait.Render.html) trait. That means you can pass the type which doesn't implement `Render` trait.


## Syntax

- Apply filter and HTML escaping

``` rhtml
<%= expression | filter %>
```

- Apply filter only

``` rhtml
<%- expression | filter %>
```

## Built-In Filters

Built-In filters can be found in [`sailfish::runtime::filter`](https://docs.rs/sailfish/latest/sailfish/runtime/filter/index.html) module.
