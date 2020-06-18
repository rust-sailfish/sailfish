use benches::{
    askama_bench, fomat, handlebars, horrorshow_bench, liquid, markup_bench, maud_bench,
    ramhorns, ructe, sailfish, std_write, tera, yarte_bench, yarte_bytes, yarte_fixed,
};
use criterion::{criterion_group, criterion_main, Criterion};

fn big_table(c: &mut Criterion) {
    let mut g = c.benchmark_group("Big table");
    g.bench_function("Askama", |b| askama_bench::big_table(b, &100));
    g.bench_function("fomat", |b| fomat::big_table(b, &100));
    g.bench_function("Handlebars", |b| handlebars::big_table(b, &100));
    g.bench_function("Horrorshow", |b| horrorshow_bench::big_table(b, &100));
    g.bench_function("Liquid", |b| liquid::big_table(b, &100));
    g.bench_function("Markup", |b| markup_bench::big_table(b, &100));
    g.bench_function("Maud", |b| maud_bench::big_table(b, &100));
    g.bench_function("Ramhorns", |b| ramhorns::big_table(b, &100));
    g.bench_function("Ructe", |b| ructe::big_table(b, &100));
    g.bench_function("Sailfish", |b| sailfish::big_table(b, &100));
    g.bench_function("Tera", |b| tera::big_table(b, &100));
    g.bench_function("Yarte", |b| yarte_bench::big_table(b, &100));
    g.bench_function("Yarte Send", |b| yarte_bytes::big_table(b, &100));
    g.bench_function("Yarte ?Send", |b| yarte_fixed::big_table(b, &100));
    g.bench_function("write", |b| std_write::big_table(b, &100));
    g.finish();
}

fn teams(c: &mut Criterion) {
    let mut g = c.benchmark_group("Teams");
    g.bench_function("Askama", |b| askama_bench::teams(b, &0));
    g.bench_function("fomat", |b| fomat::teams(b, &0));
    g.bench_function("Handlebars", |b| handlebars::teams(b, &0));
    g.bench_function("Horrorshow", |b| horrorshow_bench::teams(b, &0));
    g.bench_function("Liquid", |b| liquid::teams(b, &0));
    g.bench_function("Markup", |b| markup_bench::teams(b, &0));
    g.bench_function("Maud", |b| maud_bench::teams(b, &0));
    g.bench_function("Ramhorns", |b| ramhorns::teams(b));
    g.bench_function("Ructe", |b| ructe::teams(b, &0));
    g.bench_function("Sailfish", |b| sailfish::teams(b));
    g.bench_function("Tera", |b| tera::teams(b, &0));
    g.bench_function("Yarte", |b| yarte_bench::teams(b));
    g.bench_function("Yarte Send", |b| yarte_bytes::teams(b));
    g.bench_function("Yarte ?Send", |b| yarte_fixed::teams(b));
    g.bench_function("write", |b| std_write::teams(b, &0));
    g.finish();
}

criterion_group!(benches, big_table, teams);
criterion_main!(benches);
