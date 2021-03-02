use clap::Clap;
use std::path::PathBuf;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Wojciech Kępka <wojciech@wkepka.dev>")]
pub struct RutagOpts {
    #[clap(subcommand)]
    pub cmd: RutagCmd,
}

#[derive(Clap)]
pub enum RutagCmd {
    List {
        path: PathBuf,
        #[clap(short, long)]
        pretty: bool,
    },
    Set {
        path: PathBuf,
        tag: String,
    },
    Rm {
        path: PathBuf,
        tag: String,
    },
}
