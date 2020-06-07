use criterion::{criterion_group, criterion_main, Criterion};
use benches::{
    askama_bench, fomat, horrorshow_bench, markup_bench, sailfish, std_write, yarte_bench, yarte_fixed
};

fn big_table(c: &mut Criterion) {
    let mut g = c.benchmark_group("Big table");
    g.bench_function("Askama", |b| askama_bench::big_table(b, &100));
    g.bench_function("fomat", |b| fomat::big_table(b, &100));
    g.bench_function("Horrorshow", |b| horrorshow_bench::big_table(b, &100));
    g.bench_function("Markup", |b| markup_bench::big_table(b, &100));
    g.bench_function("Yarte", |b| yarte_bench::big_table(b, &100));
    g.bench_function("Yarte Fixed", |b| yarte_fixed::big_table(b, &100));
    g.bench_function("write", |b| std_write::big_table(b, &100));
    g.bench_function("sailfish", |b| sailfish::big_table(b, &100));
    g.finish();
}

fn teams(c: &mut Criterion) {
    let mut g = c.benchmark_group("Teams");
    g.bench_function("Askama", |b| askama_bench::teams(b, &0));
    g.bench_function("fomat", |b| fomat::teams(b, &0));
    g.bench_function("Horrorshow", |b| horrorshow_bench::teams(b, &0));
    g.bench_function("Markup", |b| markup_bench::teams(b, &0));
    g.bench_function("Yarte", |b| yarte_bench::teams(b));
    g.bench_function("Yarte Fixed", |b| yarte_fixed::teams(b));
    g.bench_function("write", |b| std_write::teams(b, &0));
    g.bench_function("sailfish", |b| sailfish::teams(b));
    g.finish();
}

criterion_group!(benches, big_table, teams);
criterion_main!(benches);
