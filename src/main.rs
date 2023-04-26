#![feature(thread_id_value)]
#![feature(concat_idents)]
#![allow(unused_assignments, unused_imports, dead_code, unused_variables, unused_must_use, non_snake_case)]

mod jdbc;
mod test;
mod h2_rust_common;
mod engine;
mod message;
mod api;
mod command;
mod util;
mod store;
mod mode;
mod result;
mod db;
mod mvstore;

fn main() {
    println!("Hello, world!");
}