use std::io::Write;

use criterion;

pub fn big_table(b: &mut criterion::Bencher<'_>, size: &usize) {
    let mut table = Vec::with_capacity(*size);
    for _ in 0..*size {
        let mut inner = Vec::with_capacity(*size);
        for i in 0..*size {
            inner.push(i);
        }
        table.push(inner);
    }
    b.iter(|| {
        let mut output = Vec::new();
        write!(&mut output, "<table>").unwrap();
        for r1 in &table {
            write!(&mut output, "<tr>\n").unwrap();
            for r2 in r1 {
                write!(&mut output, "<td>{col}</td>", col = r2).unwrap();
            }
            write!(&mut output, "</tr>\n").unwrap();
        }
        write!(&mut output, "</table>").unwrap();
    });
}

pub fn teams(b: &mut criterion::Bencher<'_>, _: &usize) {
    let teams = Teams {
        year: 2015,
        teams: vec![
            Team {
                name: "Jiangsu".into(),
                score: 43,
            },
            Team {
                name: "Beijing".into(),
                score: 27,
            },
            Team {
                name: "Guangzhou".into(),
                score: 22,
            },
            Team {
                name: "Shandong".into(),
                score: 12,
            },
        ],
    };
    b.iter(|| {
        let mut output = Vec::new();
        write!(
            &mut output,
            "<html>
            <head>
                <title>{year}</title>
            </head>
            <body>
                <h1>CSL {year}</h1>
                <ul>",
            year = teams.year
        )
        .unwrap();
        for (i, team) in (&teams).teams.iter().enumerate() {
            let champion = if i != 0 { "" } else { "champion" };
            write!(
                &mut output,
                "<li class=\"{champion}\">
                <b>{name}</b>: {score}",
                champion = champion,
                name = team.name,
                score = team.score
            )
            .unwrap();
        }
        write!(
            &mut output,
            "   </ul>
            </body>
            </html>"
        )
        .unwrap();
    });
}

struct Teams {
    year: u16,
    teams: Vec<Team>,
}

struct Team {
    name: String,
    score: u8,
}
