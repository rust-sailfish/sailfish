# Template Syntax Overview

## Tags

- `<% %>`: Inline tag, you can write Rust code inside this tag
- `<%= %>`: Evaluate the Rust expression and outputs the value into the template (HTML escaped)
- `<%- %>`: Evaluate the Rust expression and outputs the unescaped value into the template
- `<%# %>`: Comment tag
- `<%%`: Outputs a literal '<%'

## Condition

``` rhtml
<% if messages.is_empty() { %>
  <div>No messages</div>
<% } %>
```

## loop

``` rhtml
<% for (i, msg) in messages.iter().enumerate() { %>
  <div><%= i %>: <%= msg %></div>
<% } %>
```

## Includes

``` rhtml
<% include!("path/to/template"); %>
```

## Filters

``` rhtml
<%= message | upper %>
```

``` rhtml
{
    "id": <%= id %>
    "comment": <%- comment | json %>
}
```
