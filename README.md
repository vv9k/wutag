# wutag 🔱🏷️ 

Multi platform tool for tagging and organizing files by tags.

![Example usage](https://github.com/wojciechkepka/wutag/blob/master/static/usage.svg)

## Disclaimer

This tool uses so called extra attributes also known as *xattrs* to store metadata and so it might not work on certain filesystems. A thing to keep in mind is that most transfer tools require opt-in flag to transfer xattrs like `rsync` with option `-X`, `--xattrs` or `cp` with `--preserve=xattr`

Support for Windows will be added through NTFS data streams which offer access to extended attributes. I haven't yet tested if they can be preserved while transfering to other filesystems.

MacOS and Linux should work out of the box.

## Build
The build requires `rust` and `cargo`. To build run:
 - `cargo build --release`


## User interface
```
wutag 0.1.0
Wojciech Kępka <wojciech@wkepka.dev>

USAGE:
    wutag <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    clear     Clears all tags of the files that match the provided pattern in the current
              working directory. By default only first level of the directory is processed
    cp        Copies tags from the specified file to files that match a pattern
    help      Prints this message or the help of the given subcommand(s)
    list      Lists all tags of the files that match the provided pattern in the current working
              directory. By default only first level of the directory is scanned
    rm        Removes the specified tags of the files that match the provided pattern in the
              current working directory. By default only first level of the directory is
              processed
    search    Searches for files that have all of the provided `tags` in the current directory
    set       Tags the files located at the given `path` with the set of `tags`. By default only
              first level of the directory is processed
```

## License
[MIT](https://github.com/wojciechkepka/wutag/blob/master/LICENSE)
