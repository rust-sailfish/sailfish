# Rust template engine benchmarks

Performance comparison of template engines for Rust based on [criterion](https://github.com/bheisler/criterion.rs) crate

## crates

- [askama](https://github.com/djc/askama): Type-safe, compiled Jinja-like templates for Rust
- [fomat](https://github.com/krdln/fomat-macros): Alternative syntax for printing macros in Rust
- [horrorshow](https://github.com/Stebalien/horrorshow-rs): A macro-based html builder for rust
- [markup](https://github.com/utkarshkukreti/markup.rs): A blazing fast, type-safe template engine for Rust.
- [std::write!](https://doc.rust-lang.org/std/macro.write.html): the std library `write!` macro
- [yarte](https://github.com/botika/yarte): Yet Another Rust Template Engine, is the fastest template engine
- [sailfish](https://github.com/Kogia-sima/sailfish): Simple, small, and extremely fast template engine for Rust

## Running the benchmarks

```console
$ cargo bench
```

## Environment

Benchmark results were collected on the IBM Cloud Virtual Server Instance

- OS: Ubuntu 18.04-64 Minimal for VSI
- CPU Model Name: Intel(R) Xeon(R) CPU E5-2683 v4 @ 2.10GHz
- BogoMIPS: 4200.04

## Results

- Big table

![Big table](./bigtable.png)

- Teams

![Teams](./teams.png)

## License

This benchmark code is distributed under the special permission granted by [Dirkjan Ochtman](https://github.com/djc) (See [this issue](https://github.com/djc/template-benchmarks-rs/issues/26)).
**You cannot modify or redistribute the source code without an explicit permission**.
