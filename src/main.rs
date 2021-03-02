use clap::Clap;

use rutag::opt::{RutagCmd, RutagOpts};
use rutag::{clear_tags, list_tags, remove_tag, search_files_with_tag, tag_file};

fn main() {
    let opts = RutagOpts::parse();

    match opts.cmd {
        RutagCmd::List { path, pretty: _ } => match list_tags(path.as_path()) {
            Ok(attrs) => {
                println!("{} : {:?}", path.display(), attrs);
            }
            Err(e) => eprintln!("{}", e),
        },
        RutagCmd::Set { path, tag } => {
            if let Err(e) = tag_file(path.as_path(), &tag) {
                eprintln!("{}", e);
            }
        }
        RutagCmd::Rm { path, tag } => {
            if let Err(e) = remove_tag(path.as_path(), &tag) {
                eprintln!("{}", e);
            }
        }

        RutagCmd::Clear { path } => {
            if let Err(e) = clear_tags(path.as_path()) {
                eprintln!("{}", e);
            }
        }
        RutagCmd::Search { tag } => match search_files_with_tag(&tag) {
            Ok(files) => {
                println!("Files with tag `{}`:", tag);
                for file in files {
                    println!("\t- {}", file.display());
                }
            }
            Err(e) => eprintln!("{}", e),
        },
    }
}
