mod opt;
mod runner;

use clap::Clap;

use opt::WutagOpts;
use runner::WutagRunner;

fn main() {
    match WutagRunner::new(WutagOpts::parse()) {
        Ok(wutag) => wutag.run(),
        Err(e) => eprintln!("{}", e.to_string()),
    }
}
