#[macro_use] extern crate lazy_static;
extern crate regex;

pub mod regex_utils;
pub mod common;
pub mod ttm_io;

fn main() {
    ttm_io::experiment_parse_var("let X: u32 = 500;");
    ttm_io::experiment_parse_var("let X: u32 = 500;");
}
