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

- OS: Ubuntu 20.04 LTS
- CPU Model Name: Intel(R) Core(TM) i5-8265U CPU @ 1.60GHz
- BogoMIPS: 3600.00

## Results

- Big table

![Big table](./bigtable.png)

- Teams

![Teams](./teams.png)
