use criterion;
use markup::define;

// TODO: Switch to `markup::define!` when upgrading to Rust 2018.
define! {
    BigTable<'a>(table: &'a [Vec<usize>]) {
        table {
            @for r1 in table.iter() {
                tr {
                    @for r2 in r1.iter() {
                        td { {*r2} }
                    }
                }
            }
        }
    }
}

pub fn big_table(b: &mut criterion::Bencher<'_>, size: &usize) {
    let mut table = Vec::with_capacity(*size);
    for _ in 0..*size {
        let mut inner = Vec::with_capacity(*size);
        for i in 0..*size {
            inner.push(i);
        }
        table.push(inner);
    }
    b.iter(|| BigTable { table: &table }.to_string());
}

pub struct Team {
    name: String,
    score: u8,
}

define! {
    Teams<'a>(year: u32, teams: &'a [Team]) {
        html {
            head {
                title { {year} }
            }
            body {
                h1 { "CSL " {year} }
                ul {
                    @for (index, team) in teams.iter().enumerate() {
                        li.{if index == 0 { Some("champion") } else { None }} {
                            b { {team.name} } ": " {team.score}
                        }
                    }
                }
            }
        }
    }
}

pub fn teams(b: &mut criterion::Bencher<'_>, _: &usize) {
    let year = 2015;
    let teams = vec![
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
    ];
    b.iter(|| {
        Teams {
            year,
            teams: &teams,
        }
        .to_string()
    });
}
