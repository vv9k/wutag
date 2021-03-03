use std::fmt::Display;

use clap::Clap;
use colored::Colorize;

use rutag::opt::{RutagCmd, RutagOpts};
use rutag::{clear_tags, list_tags, remove_tag, search_files_with_tag, tag_file};

fn display_err<E: std::fmt::Display>(err: E) {
    eprintln!(
        "{}:\t{}",
        "ERROR".red().bold(),
        format!("{}", err).white().bold()
    )
}

fn display_arrow<D: Display>(from: D, to: D) {
    println!(
        "{} {}{} {}",
        from,
        "~~~".green().bold(),
        ">".red().bold(),
        to
    )
}

fn main() {
    let opts = RutagOpts::parse();

    match opts.cmd {
        RutagCmd::List { path, pretty: _ } => match list_tags(path.as_path()) {
            Ok(tags) => {
                print!("{}:\t", path.display().to_string().bold().blue());
                for tag in tags {
                    print!("{}\t", tag.bold().white());
                }
            }
            Err(e) => display_err(e),
        },
        RutagCmd::Set { path, tag } => {
            if let Err(e) = tag_file(path.as_path(), &tag) {
                display_err(e);
            } else {
                display_arrow(tag.bold().white(), path.display().to_string().bold().blue());
            }
        }
        RutagCmd::Rm { path, tag } => {
            if let Err(e) = remove_tag(path.as_path(), &tag) {
                display_err(e);
            }
        }

        RutagCmd::Clear { path } => {
            if let Err(e) = clear_tags(path.as_path()) {
                display_err(e);
            }
        }
        RutagCmd::Search { tag } => match search_files_with_tag(&tag) {
            Ok(files) => {
                println!("Files with tag {}:", tag.bold().white());
                for file in files {
                    println!("\t- {}", file.display().to_string().bold().blue());
                }
            }
            Err(e) => display_err(e),
        },
    }
}
