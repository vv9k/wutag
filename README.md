# wutag 🔱🏷️ 
[![master](https://github.com/vv9k/wutag/actions/workflows/master.yml/badge.svg)](https://github.com/vv9k/wutag/actions/workflows/master.yml)

CLI tool for tagging and organizing files by tags.

![Example usage](https://github.com/vv9k/wutag/blob/master/static/usage.svg)

## Install

If you use arch Linux and have AUR repositories set up you can use your favourite AUR manager to download `wutag`. For example with `paru`:
 - `paru -S wutag`
 - or latest master branch with `paru -S wutag-git`

If you're on another Linux distribution or MacOS you can download one of the prebuilt binaries from [here](https://github.com/vv9k/wutag/releases).

To build manually you'll need latest `rust` and `cargo`. Build with:
 - `cargo build --release`

## Usage

By default each tag will be assigned with a random color from 8 base colors (either bright or normal so 16 colors in total). You can later edit each tag by using `edit` subcommand like this:
 - `wutag edit school --color 0x1f1f1f`
 - or `wutag edit code --color '#ff00aa'`
 - or `wutag edit work --color FF0000`
 - The colors are case insensitive

Each command that takes a pattern starts a filesystem traversal from current working directory. To override this
behaviour specify a global parameter `--dir` or `-d` like this:
 - `wutag -d ~ set '**' code`

Default recursion depth is set to *2*. To increase it use `--max-depth` or `-m` global parameter.

After tagging your files with `set` like:
 - `wutag set '*.jpg' photos`
 - `wutag set 'DCIM_12*' doge`  
you can easily get the list of files with specified tags by doing `wutag search photos doge`. 

To utilize the list by other programs pass the `--raw` or `-r` flag to `search` subcommand like:
 - `wutag search -r --any cat doge | xargs rm -rf  # please don't do this :(`. 

When `--any` flag is provided as in the example `wutag` will match files containing any of the provided tags rather than all of them.

If you are into emojis then surely you can use emojis to tag files 🙂 ```wutag set '*.doc' 📋```

## Configuration

`wutag` lets you configure base colors used when creating tags or modify other settings globally. To do this create a file `.wutag.yml` in your home directory like `~/.wutag.yml`.

Example configuration:
```yaml
---
max_depth: 100
colors:
- '0xabba0f'
- '#121212'
- '0x111111'
```

## Tab completion

To get tab completion use `wutag print-completions <shell> > /path/to/completions/dir/...` to enable it in your favourite shell.  

Available shells are:
 - `bash`
 - `elvish`
 - `fish`
 - `powershell`
 - `zsh`

 To enable completions on the fly use:
 - `. <(wutag print-completions bash)`


## User interface
```
USAGE:
    wutag [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help        Prints help information
    -n, --no-color    If passed the output won't be colored
    -V, --version     Prints version information

OPTIONS:
    -d, --dir <dir>                When this parameter is specified the program will look for files
                                   starting from provided path, otherwise defaults to current
                                   directory
    -m, --max-depth <max-depth>    If provided increase maximum recursion depth of filesystem
                                   traversal to specified value, otherwise default depth is 2

SUBCOMMANDS:
    clear                Clears all tags of the files that match the provided pattern
    cp                   Copies tags from the specified file to files that match a pattern
    edit                 Edits the tag of files that match the provided pattern
    help                 Prints this message or the help of the given subcommand(s)
    list                 Lists all tags of the files that match the provided pattern
    print-completions    Prints completions for the specified shell to stdout
    rm                   Removes the specified tags of the files that match the provided pattern
    search               Searches for files that have all of the provided `tags`
    set                  Tags the files that match the given pattern with specified tags
```

## License
[MIT](https://github.com/vv9k/wutag/blob/master/LICENSE)
