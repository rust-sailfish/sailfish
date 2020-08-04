# Template Syntax Overview

## Tags

- `<% %>`: Inline tag, you can write Rust code inside this tag
- `<%= %>`: Evaluate the Rust expression and outputs the value into the template (HTML escaped)
- `<%- %>`: Evaluate the Rust expression and outputs the unescaped value into the template
- `<%# %>`: Comment tag
- `<%%`: Outputs a literal '<%'

## Condition

```ejs
<% if messages.is_empty() { %>
  <div>No messages</div>
<% } %>
```

## loop

```ejs
<% for (i, msg) in messages.iter().enumerate() { %>
  <div><%= i %>: <%= msg %></div>
<% } %>
```

## Includes

```ejs
<% include!("path/to/template"); %>
```

Unlike EJS, you cannot omit the file extension.

## Filters

```ejs
<%= message | upper %>
```

```ejs
{
    "id": <%= id %>
    "comment": <%- comment | dbg %>
}
```
