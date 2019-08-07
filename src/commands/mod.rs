mod bootnode_server;
mod chainspec;
mod ethereum;
mod keeper;
mod liveness;
mod readiness;
mod shell;

use structopt::StructOpt;

use self::shell::Shell;

#[derive(Debug, StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
pub enum Command {
    #[structopt(name = "completion")]
    RunCompletion {
        #[structopt(subcommand)]
        shell: Shell,
    },

    #[structopt(name = "generate-chainspec")]
    GenerateChainSpec,

    #[structopt(name = "run-bootnode-server")]
    RunBootnodeServer,

    #[structopt(name = "run-network-keeper")]
    RunNetworkKeeper,

    #[structopt(name = "run-ethereum")]
    RunEthereum {
        #[structopt(subcommand)]
        runlevel: RunEthereumRunlevel,
    },

    #[structopt(name = "liveness-probe")]
    RunLivenessProbe,

    #[structopt(name = "readiness-probe")]
    RunReadinessProbe,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "run-ethereum-runlevel")]
pub enum RunEthereumRunlevel {
    #[structopt(name = "init")]
    Initial,

    #[structopt(name = "exec")]
    Exec,

    #[structopt(name = "full")]
    Full,
}

impl Command {
    pub fn run(self) {
        let exit_code = match self {
            Command::RunCompletion { shell } => {
                Command::clap().gen_completions_to(
                    "etherinit",
                    shell.into(),
                    &mut std::io::stdout(),
                );
                0
            }

            Command::GenerateChainSpec => chainspec::generate_chainspec(),

            Command::RunBootnodeServer => bootnode_server::execute(),
            Command::RunNetworkKeeper => keeper::execute(),

            Command::RunEthereum { runlevel } => match runlevel {
                RunEthereumRunlevel::Initial => ethereum::run_init(),
                RunEthereumRunlevel::Exec => ethereum::run_exec(),
                RunEthereumRunlevel::Full => ethereum::run_full(),
            },

            Command::RunLivenessProbe => liveness::execute(),
            Command::RunReadinessProbe => readiness::execute(),
        };

        ::std::process::exit(exit_code);
    }
}
