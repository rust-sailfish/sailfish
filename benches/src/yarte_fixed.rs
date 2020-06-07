use yarte::TemplateFixed;

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
    b.iter(|| {
        let mut buf = String::with_capacity(109915);
        unsafe {
            buf.as_mut_vec().set_len(109915);
            let b = ctx.call(buf.as_bytes_mut()).unwrap();
            buf.as_mut_vec().set_len(b);
        }
        buf
    });
}

#[derive(TemplateFixed)]
#[template(path = "big-table.hbs")]
struct BigTable {
    table: Vec<Vec<usize>>,
}

pub fn teams(b: &mut criterion::Bencher<'_>) {
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
        let mut buf = String::with_capacity(239);
        unsafe {
            buf.as_mut_vec().set_len(239);
            let b = teams.call(buf.as_bytes_mut()).unwrap();
            buf.as_mut_vec().set_len(b);
        }
        buf
    });
}

#[derive(TemplateFixed)]
#[template(path = "teams.hbs")]
struct Teams {
    year: u16,
    teams: Vec<Team>,
}

struct Team {
    name: String,
    score: u8,
}
