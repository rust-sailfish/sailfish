# Tags

## Code block

You can write Rust statement inside `<% %>` tag.

=== "Template"

    ``` rhtml
    <%
    let mut total = 0;
    for i in 1.. {
        total += i;
        if i > 100 {
            break;
        }
    }
    %>
    <div>total = <%= total %></div>
    ```

=== "Result"

    ``` html
    <div>total = 105</div>
    ```

!!! Note
    Make sure that you cannot omit braces, parenthesis, and semicolons.

Sailfish is smart enough to figure out where the code block ends, so you can even include `%>` inside Rust comments or string literals.

=== "Template"

    ``` text
    <% /* Tag does not ends at %>! */ %>
    ```

=== "Result"

    ``` text
    ```

If you need to simply render `<%` character, you can escape it, or use evaluation block (described below).

=== "Template"

    ``` text
    <%% is converted into <%- "<%" %> character.
    ```

=== "Result"

    ``` text
    <% is converted into <% character
    ```

Although almost all Rust statement is supported, the following statements inside templates may cause a strange compilation error.

- Function/Macro definition that render some contents
- `impl` item
- Macro call which defines some local variable.
- Macro call which behaviour depends on the path to source file
- Generator expression (yield)

## Evaluation block

Rust expression inside `<%= %>` tag is evaluated and the result will be rendered.

=== "Template"

    ``` rhtml
    <% let a = 1; %><%= a + 2 %>
    ```

=== "Result"

    ``` text
    3
    ```

If the result contains `&"'<>` characters, sailfish replaces these characters with the equivalent html.

If you want to render the results without escaping, you can use `<%- %>` tag or [configure sailfish to not escape by default](../options.md).

=== "Template"

    ``` rhtml
    <div>
      <%- "<h1>Hello, World!</h1>" %>
    </div>
    ```

=== "Result"

    ``` html
    <div>
      <h1>Hello, World!</h1>
    </div>
    ```

!!! Note
    Evaluation block does not return any value, so you cannot use the block to pass the render result to another code block. The following code is invalid.

    ``` rhtml
    <% let result = %><%= 1 %><% ; %>
    ```
