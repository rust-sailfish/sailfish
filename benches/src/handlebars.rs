use std::collections::BTreeMap;

use ::handlebars::{to_json, Handlebars};
use criterion;
use serde::Serialize;
use serde_json;
use serde_json::value::Value as Json;

pub fn big_table(b: &mut criterion::Bencher<'_>, size: &usize) {
    let mut table = Vec::with_capacity(*size);
    for _ in 0..*size {
        let mut inner = Vec::with_capacity(*size);
        for i in 0..*size {
            inner.push(i);
        }
        table.push(inner);
    }

    let ctx = BigTable { table };
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("big-table.html", SOURCE)
        .unwrap();
    b.iter(|| handlebars.render("big-table.html", &ctx).ok().unwrap());
}

#[derive(Serialize)]
struct BigTable {
    table: Vec<Vec<usize>>,
}

static SOURCE: &'static str = "<html>
    {{#each table as |n|}}
        <tr>{{#each n as |v|}}<td>{{v}}</td>{{/each}}</tr>
    {{/each}}
</html>";

pub fn teams(b: &mut criterion::Bencher<'_>, _: &usize) {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("table", TEAMS_TEMPLATE)
        .ok()
        .expect("Invalid template format");

    let data = teams_data();
    b.iter(|| handlebars.render("table", &data).ok().unwrap())
}

fn teams_data() -> BTreeMap<String, Json> {
    let mut data = BTreeMap::new();

    data.insert("year".to_string(), to_json(&"2015".to_owned()));

    let mut teams = Vec::new();

    for v in vec![
        ("Jiangsu", 43u16),
        ("Beijing", 27u16),
        ("Guangzhou", 22u16),
        ("Shandong", 12u16),
    ]
    .iter()
    {
        let (name, score) = *v;
        let mut t = BTreeMap::new();
        t.insert("name".to_string(), to_json(&name));
        t.insert("score".to_string(), to_json(&score));
        teams.push(t)
    }

    data.insert("teams".to_string(), to_json(&teams));
    data
}

static TEAMS_TEMPLATE: &'static str = "<html>
  <head>
    <title>{{year}}</title>
  </head>
  <body>
    <h1>CSL {{year}}</h1>
    <ul>
    {{#each teams}}
      <li class=\"{{#if @first}}champion{{/if}}\">
      <b>{{name}}</b>: {{score}}
      </li>
    {{/each}}
    </ul>
  </body>
</html>";
