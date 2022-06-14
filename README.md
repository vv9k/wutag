# wutag 🔱🏷️ 
[![master](https://github.com/vv9k/wutag/actions/workflows/master.yml/badge.svg)](https://github.com/vv9k/wutag/actions/workflows/master.yml)

CLI tool for tagging and organizing files by tags.

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

`wutag` lets you configure base colors used when creating tags or modify other settings globally. To do this create a file `wutag.yml` in your config directory (on unix XDG_CONFIG_DIR) lihe `~/.config/wutag.yml`.

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


## License
[MIT](https://github.com/vv9k/wutag/blob/master/LICENSE)
