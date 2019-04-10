#![recursion_limit = "128"]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate toml;

#[macro_use]
extern crate tower_web;

use hdwallet;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

mod bootnode;
mod cli;
mod commands;
mod ethereum_controller;
mod ethereum_launcher;
mod network_keeper;
pub mod primitives;
pub mod utils;

fn main() {
    cli::Cli::build().command().run();
}
