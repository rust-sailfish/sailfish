# Tags

## Code block

You can write Rust statement inside `<% %>` tag.

```ejs
<% let mut total = 0; %>
<% for elem in arr.iter().filter(|e| e.is_valid()) { %>
    <% total += elem.value() as u64; %>
    <% dbg!(total); %>
    <% if total > 100 { break; } %>
    Printed until the total value exceeds 100.
<% } %>
```

!!! Note
    Make sure that you cannot omit braces, parenthesis, and semicolons.

Sailfish is smart enough to figure out where the code block ends, so you can even include `%>` inside Rust comments or string literals.

```text
<% /* Tag does not ends at %>! */ %>
```

If you need to simply render `<%` character, you can escape it, or use evaluation block (described below).

```text
<%% is converted into <%= "<%" %> character.
```

Although almost all Rust statement is supported, the following statements inside templates may cause a strange compilation error.

- Function/Macro definition that render some contents
- `impl` item
- Macro call which defines some local variable.
- Macro call which behaviour depends on the path to source file
- Generator expression (yield)

## Evaluation block

Rust expression inside `<%= %>` tag is evaluated and the result will be rendered.

```ejs
<%# The following code simple renders `3` %>
<% let a = 1; %><%= a + 2 %>
```

If the result contains `&"'<>` characters, sailfish replaces these characters with the equivalent html.

If you want to render the results without escaping, you can use `<%- %>` tag or [configure sailfish to not escape by default](../options.md). For example,

```ejs
<div>
  <%- "<h1>Hello, World!</h1>" %>
</div>
```

This template results in the following output.

```ejs
<div>
  <h1>Hello, World!</h1>
</div>
```

!!! Note
    Evaluation block does not return any value, so you cannot use the block to pass the render result to another code block. The following code is invalid.

    ```
    <% let result = %><%= 1 %><% ; %>
    ```
