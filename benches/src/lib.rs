#![feature(proc_macro_hygiene)]

pub mod askama_bench;
pub mod fomat;
pub mod handlebars;
pub mod horrorshow_bench;
pub mod liquid;
pub mod markup_bench;
pub mod maud_bench;
pub mod ramhorns;
pub mod ructe;
pub mod sailfish;
pub mod std_write;
pub mod tera;
pub mod yarte_bench;
pub mod yarte_fixed;

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
