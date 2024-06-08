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

## Useful Filters

You can also use the Display filter to do things like format a date, or a UUID.

- Easily Display A Date

=== "Template"

    ```rhtml
    <span><%= chrono::NaiveDate::from_ymd_opt(2015, 9, 5).unwrap().and_hms_opt(23, 56, 4).unwrap().format("around %l %p on %b %-d") | disp %></span>
    ```

=== "Result"

    ```html
    <span>around 11 PM on Sep 5</span>
    ```

- Easily Display A UUID

=== "Template"

    ```rhtml
    <span><%= uuid::Uuid::new_v4().urn() | disp %></span>
    ```

=== "Result"

    ```html
    <span>urn:uuid:c3602585-a9a1-43f5-b0e6-8bc05d5444dd</span>
    ```
