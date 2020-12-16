use crate::templates;
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
        let mut buf = Vec::new();
        templates::big_table(&mut buf, &table).unwrap();
    });
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
        let mut buf = Vec::new();
        templates::teams(&mut buf, year, &teams).unwrap();
    });
}

pub struct Team {
    pub name: String,
    pub score: u8,
}
