use super::bootnode;
use super::ethereum_controller;
use super::ethereum_launcher;
use super::hdwallet;
use super::network_keeper;

pub use super::primitives;
pub use super::utils;

mod bootnode_server;
mod chainspec;
mod ethereum;
mod keeper;

pub enum Command {
    ExitSuccess,
    ExitFailure,

    GenerateChainSpec,

    RunBootnodeServer,
    RunNetworkKeeper,

    RunEthereum,
}

impl Command {
    pub fn run(self) {
        let exit_code = match self {
            Command::ExitSuccess => 0,
            Command::ExitFailure => -1,

            Command::GenerateChainSpec => chainspec::generate_chainspec(),

            Command::RunBootnodeServer => bootnode_server::execute(),
            Command::RunNetworkKeeper => keeper::execute(),

            Command::RunEthereum => ethereum::execute(),
        };

        ::std::process::exit(exit_code);
    }
}
