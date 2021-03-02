use clap::Clap;
use serde_json::Value;
use std::path::PathBuf;

use rutag::{get_xattr, list_xattrs, set_xattr};

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Wojciech Kępka <wojciech@wkepka.dev>")]
struct RutagOpts {
    #[clap(subcommand)]
    pub cmd: RutagCmd,
}

#[derive(Clap)]
enum RutagCmd {
    List {
        path: PathBuf,
        #[clap(short, long)]
        pretty: bool,
    },
    Set {
        path: PathBuf,
        key: String,
        value: String,
    },
    Get {
        path: PathBuf,
        key: String,
    },
}

fn main() {
    let opts = RutagOpts::parse();

    match opts.cmd {
        RutagCmd::Get { path, key } => match get_xattr(path.as_path(), &key) {
            Ok(tag) => println!("{}={}", key, tag),
            Err(e) => eprintln!("{}", e),
        },
        RutagCmd::List { path, pretty } => match list_xattrs(path.as_path()) {
            Ok(attrs) => {
                let attrs = attrs
                    .into_iter()
                    .map(|(k, v)| (k, Value::from(v.as_str())))
                    .collect::<serde_json::Map<String, Value>>();

                let display = if pretty {
                    serde_json::to_string_pretty(&attrs)
                } else {
                    serde_json::to_string(&attrs)
                };

                match display {
                    Ok(display) => {
                        println!("{}", display);
                    }
                    Err(e) => eprintln!("failed to serialize attributes - {}", e),
                }
            }
            Err(e) => eprintln!("{}", e),
        },
        RutagCmd::Set { path, key, value } => {
            if let Err(e) = set_xattr(path.as_path(), &key, &value) {
                eprintln!("{}", e);
            }
        }
    }
}
