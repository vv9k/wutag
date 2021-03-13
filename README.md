# wutag 🔱🏷️ 
[![master](https://github.com/wojciechkepka/wutag/actions/workflows/master.yml/badge.svg)](https://github.com/wojciechkepka/wutag/actions/workflows/master.yml)

CLI tool for tagging and organizing files by tags.

![Example usage](https://github.com/wojciechkepka/wutag/blob/master/static/usage.svg)

## Persistance

This tool uses extra attributes also known as *xattrs* to store metadata so it might not work on certain filesystems. A thing to keep in mind is that most transfer tools require opt-in flag to transfer xattrs like `rsync` with option `-X`, `--xattrs` or `cp` with `--preserve=xattr` while `mv` preserves xattrs by default.

Adding aliases in your `.bashrc` like `alias cp="cp --preserve=xattr"` might help avoiding accidental removal of tags.

GUI file managers seem to support them by default, only tested with `nautilus` and `Thunar` though so mileage may vary.

Support for Windows will be added through NTFS data streams which offer access to extended attributes. I haven't yet tested if they can be preserved while transfering to other filesystems.

MacOS and Linux should work out of the box.

## Install

If you use arch Linux and have AUR repositories set up you can use your favourite AUR manager to download `wutag`. For example with `paru`:
 - `paru -S wutag-git`

If you're on another Linux distribution or MacOS you can download one of the prebuilt binaries from [here](https://github.com/wojciechkepka/wutag/releases).

To build manually you'll need latest `rust` and `cargo`. Build with:
 - `cargo build --release`

## User interface
```
USAGE:
    wutag [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help         Prints help information
    -n, --no-color     If passed the output won't be colored
    -r, --recursive    Increase maximum recursion depth of filesystem traversal to 512. Default is
                       2. What this means is by deafult all subcommands that take a pattern as input
                       will match files only 2 levels deep starting from the base directory which is
                       current working directory if `dir` is not specified
    -V, --version      Prints version information

OPTIONS:
    -d, --dir <dir>    When this parameter is specified the program will look for files starting
                       from provided path, otherwise defaults to current directory

SUBCOMMANDS:
    clear     Clears all tags of the files that match the provided pattern.
    cp        Copies tags from the specified file to files that match a pattern
    help      Prints this message or the help of the given subcommand(s)
    list      Lists all tags of the files that match the provided pattern
    rm        Removes the specified tags of the files that match the provided pattern
    search    Searches for files that have all of the provided `tags`
    set       Tags the files that match the given pattern with specified tags
```

## License
[MIT](https://github.com/wojciechkepka/wutag/blob/master/LICENSE)
