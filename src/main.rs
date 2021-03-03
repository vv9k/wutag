use clap::Clap;
use colored::Colorize;
use std::fmt::Display;

use rutag::opt::{RutagCmd, RutagOpts};
use rutag::{clear_tags, list_tags, remove_tag, search_files_with_tag, tag_file};

fn fmt_err<E: Display>(err: E) -> String {
    format!(
        "{}:\t{}",
        "ERROR".red().bold(),
        format!("{}", err).white().bold()
    )
}

fn fmt_arrow<D: Display>(from: D, to: D) -> String {
    format!(
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
                    print!("{}\t", tag.bold().yellow());
                }
            }
            Err(e) => eprintln!("{}", fmt_err(e)),
        },
        RutagCmd::Set { paths, tags } => paths.into_iter().for_each(|path| {
            println!("`{}`:", path.display().to_string().bold().blue());
            tags.iter().for_each(|tag| {
                if let Err(e) = tag_file(path.as_path(), &tag) {
                    eprintln!("  {}", fmt_err(e));
                } else {
                    println!("  {} {}", "+".bold().green(), tag.bold().yellow());
                }
            });
        }),
        RutagCmd::Rm { paths, tags } => paths.into_iter().for_each(|path| {
            println!("`{}`:", path.display().to_string().bold().blue());
            tags.iter().for_each(|tag| {
                if let Err(e) = remove_tag(path.as_path(), &tag) {
                    eprintln!("  {}", fmt_err(e));
                } else {
                    println!("  {} {}", "X".bold().red(), tag.bold().yellow());
                }
            })
        }),
        RutagCmd::Clear { path } => {
            if let Err(e) = clear_tags(path.as_path()) {
                eprintln!("{}", fmt_err(e));
            }
        }
        RutagCmd::Search { tag } => match search_files_with_tag(&tag) {
            Ok(files) => {
                if files.is_empty() {
                    println!("No files with tag {} were found.", tag.bold().yellow(),);
                } else {
                    println!("Files with tag {}:", tag.bold().yellow());
                    for file in files {
                        println!("\t{}", file.display().to_string().bold().blue());
                    }
                }
            }
            Err(e) => eprintln!("{}", fmt_err(e)),
        },
    }
}
