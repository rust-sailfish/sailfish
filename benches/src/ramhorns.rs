use ramhorns::{Template, Content};

pub fn big_table(b: &mut criterion::Bencher<'_>, size: &usize) {
    let mut table = Vec::with_capacity(*size);
    for _ in 0..*size {
        let mut inner = Vec::with_capacity(*size);
        for i in 0..*size {
            inner.push(i);
        }
        table.push(inner);
    }
    let tpl = Template::new(SOURCE).unwrap();
    let ctx = BigTable { table };
    b.iter(|| {
        tpl.render(&ctx)
    });
}

#[derive(Content)]
struct BigTable {
    table: Vec<Vec<usize>>,
}

static SOURCE: &'static str = "<html>
    {{#table}}
        <tr>{{#.}}<td>{{.}}</td>{{/.}}</tr>
    {{/table}}
</html>";

pub fn teams(b: &mut criterion::Bencher<'_>) {
    let tpl = Template::new(TEAMS_TEMPLATE).unwrap();
    let teams = Teams {
        year: 2015,
        teams: vec![
            Team {
                name: "Jiangsu".into(),
                class: "champion".into(),
                score: 43,
            },
            Team {
                name: "Beijing".into(),
                class: String::new(),
                score: 27,
            },
            Team {
                name: "Guangzhou".into(),
                class: String::new(),
                score: 22,
            },
            Team {
                name: "Shandong".into(),
                class: String::new(),
                score: 12,
            },
        ],
    };
    b.iter(|| {
        tpl.render(&teams)
    });
}

#[derive(Content)]
struct Teams {
    year: u16,
    teams: Vec<Team>,
}

#[derive(Content)]
struct Team {
    name: String,
    class: String,
    score: u8,
}

static TEAMS_TEMPLATE: &'static str = "<html>
  <head>
    <title>{{year}}</title>
  </head>
  <body>
    <h1>CSL {{year}}</h1>
    <ul>
    {{#teams}}
      <li class=\"{{class}}\">
      <b>{{name}}</b>: {{score}}
      </li>
    {{/teams}}
    </ul>
  </body>
</html>";
