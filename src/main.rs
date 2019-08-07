#![recursion_limit = "128"]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate toml;

#[macro_use]
extern crate tower_web;

#[macro_use]
extern crate failure;

extern crate structopt;

mod bootnode;
mod commands;
mod ethereum_controller;
mod ethereum_launcher;
mod network_keeper;

pub mod primitives;
pub mod utils;

use structopt::StructOpt;

fn main() {
    commands::Command::from_args().run();
}
