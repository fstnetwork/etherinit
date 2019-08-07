use structopt::{clap::Shell as ClapShell, StructOpt};

#[derive(Debug, StructOpt)]
pub enum Shell {
    #[structopt(name = "bash")]
    Bash,

    #[structopt(name = "zsh")]
    Zsh,

    #[structopt(name = "fish")]
    Fish,

    #[structopt(name = "powershell")]
    PowerShell,

    #[structopt(name = "elvish")]
    Elvish,
}

impl Into<ClapShell> for Shell {
    fn into(self) -> ClapShell {
        match self {
            Shell::Bash => ClapShell::Bash,
            Shell::Elvish => ClapShell::Elvish,
            Shell::Fish => ClapShell::Fish,
            Shell::PowerShell => ClapShell::PowerShell,
            Shell::Zsh => ClapShell::Zsh,
        }
    }
}
