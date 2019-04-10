#![recursion_limit = "128"]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;

extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate serde_derive;

#[macro_use]
extern crate toml;

extern crate futures;
extern crate tokio;
extern crate tokio_process;
extern crate tokio_signal;
extern crate tokio_timer;

extern crate ethereum_types;
extern crate hyper;
extern crate parking_lot;
extern crate url;

#[macro_use]
extern crate tower_web;

extern crate ethsign;
extern crate hdwallet;
extern crate web3;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
#[cfg(test)]
extern crate hex;

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
