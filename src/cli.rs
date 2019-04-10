use clap::{App, SubCommand};

use super::commands::Command;

pub struct Cli(App<'static, 'static>);

impl Cli {
    pub fn build() -> Cli {
        Cli(App::new(crate_name!())
            .author(crate_authors!())
            .version(crate_version!())
            .about(crate_description!())
            .subcommand(SubCommand::with_name("help").about("Show usage of FST-EtherInit"))
            .subcommand(SubCommand::with_name("version").about("Show version of FST-EtherInit"))
            .subcommand(
                SubCommand::with_name("generate-chainspec").about("Generate Ethereum ChainSpec"),
            )
            .subcommand(SubCommand::with_name("run-bootnode-server").about("Run Bootnode Service"))
            .subcommand(SubCommand::with_name("run-network-keeper").about("Run Network Keeper"))
            .subcommand(SubCommand::with_name("run-ethereum").about("Run Ethereum Service"))
            .subcommand(
                SubCommand::with_name("completions")
                    .about("Generate shell completions")
                    .subcommand(SubCommand::with_name("bash").about("Generate Bash completions"))
                    .subcommand(SubCommand::with_name("fish").about("Generate Fish completions"))
                    .subcommand(SubCommand::with_name("zsh").about("Generate Zsh completions"))
                    .subcommand(
                        SubCommand::with_name("powershell")
                            .about("Generate PowerShell completions"),
                    ),
            ))
    }

    pub fn command(self) -> Command {
        let cli = Self::build();
        let mut app = cli.0;
        let matches = app.clone().get_matches();

        match matches.subcommand() {
            ("generate-chainspec", _) => Command::GenerateChainSpec,
            ("run-bootnode-server", _) => Command::RunBootnodeServer,
            ("run-network-keeper", _) => Command::RunNetworkKeeper,
            ("run-ethereum", _) => Command::RunEthereum,
            ("completions", Some(cmd)) => {
                let shell = match cmd.subcommand() {
                    ("bash", _) => clap::Shell::Bash,
                    ("fish", _) => clap::Shell::Fish,
                    ("zsh", _) => clap::Shell::Zsh,
                    ("powershell", _) => clap::Shell::PowerShell,
                    _ => {
                        app.print_help().unwrap();
                        return Command::ExitFailure;
                    }
                };
                app.gen_completions_to(crate_name!(), shell, &mut std::io::stdout());
                Command::ExitSuccess
            }
            ("help", Some(_)) => {
                app.print_help().unwrap();
                Command::ExitSuccess
            }
            ("version", Some(_)) => {
                println!("{} {}", crate_name!(), crate_version!());
                Command::ExitSuccess
            }
            (_, _) => {
                app.print_help().unwrap();
                Command::ExitFailure
            }
        }
    }
}
