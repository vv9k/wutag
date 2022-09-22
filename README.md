# wutag 🔱🏷️ 
[![master](https://github.com/vv9k/wutag/actions/workflows/master.yml/badge.svg)](https://github.com/vv9k/wutag/actions/workflows/master.yml)

CLI tool for tagging and organizing files by tags.

## Install

If you use arch Linux and have AUR repositories set up you can use your favourite AUR manager to download `wutag`. For example with `paru`:
 - `paru -S wutag`
 - or latest master branch with `paru -S wutag-git`

If you're on another Linux distribution or MacOS you can download one of the prebuilt binaries from [here](https://github.com/vv9k/wutag/releases).

To build manually you'll need latest `rust` and `cargo`. Build with:
 - `cargo build --release --bins`

## Daemon

Wutag works in a client-server architecture with a daemon running in a background. If you install **wutag** from package like RPM or DEB it should automatically install the systemd service for you.

### Linux

If **wutag** was built manually, install the [example service](https://github.com/vv9k/wutag/blob/master/example/wutagd.service) to `/etc/systemd/user/`

To enable the daemon run:
```shell
    $ systemctl --user daemon-reload
    $ systemctl --user enable --now wutagd.service
```

Or start the daemon manually with:
```shell
    $ /usr/bin/wutagd
```

### Mac OS

To install wutag on MacOS download the static build of latest version from [releases page](https://github.com/vv9k/wutag/releases) and copy the binaries to `/usr/local/bin`. Last step is to copy the service definition `com.wutag.wutagd.plist` to `~/Library/LaunchAgents` and load it:
```shell
    $ launchctl load ~/Library/LaunchAgents/com.wutag.wutagd.plist
    $ launchctl start com.wutag.wutagd
```

## Usage

By default each tag will be assigned with a random color from 8 base colors (either bright or normal so 16 colors in total). You can later edit each tag by using `edit` subcommand like this:
 - `wutag edit school --color 0x1f1f1f`
 - or `wutag edit code --color '#ff00aa'`
 - or `wutag edit work --color FF0000`
 - The colors are case insensitive

To set a tag on multiple files use the `set` subcommand:
 - `wutag set src/lib.rs src/main.rs --tags code`  
The `set` subcommand can also be used with a pattern like this:
 - `wutag set -g '**' -t rust code`

To get the tags of some entries use:
 - `wutag get src/lib.rs src/main.rs`

The `rm` subcommand removes specified tags from the list of files. It behaves similarly to the `set` subcommand:
 - `wutag rm src/lib.rs src/main.rs -t code`
 - `wutag rm -g '**' -t rust code`

There is also `clear` that clears all tags from the specified files like so:
 - `wutag clear files Cargo.toml`
 - `wutag clear files -g '**'`  
Or clears the tags entirelly untagging all entries:
 - `wutag clear tags rust code`

When using glob processing, default recursion depth is set to *2*. To increase it use `--max-depth` or `-m` global parameter. For example:
 - `wutag -m 5 set -g '**' -t trash`

After tagging your files with `set` like:
 - `wutag set -g '**/*.jpg' -t photos`
 - `wutag set -g '**/DCIM_12*' -t doge`  
you can easily get the list of files with specified tags by doing `wutag search photos doge`. 

The output of the `search` subcommand can easily be piped to other programs:
 - `wutag search --any cat doge | xargs rm -rf  # please don't do this :(`. 

When `--any` flag is provided as in the example `wutag` will match files containing any of the provided tags rather than all of them.

If you are into emojis then surely you can use emojis to tag files 🙂 `wutag set -g '*.doc' -t 📋`

## Configuration

`wutag` lets you configure base colors used when creating tags or modify other settings globally. To do this create a file `wutag.yml` in your config directory (on unix **XDG_CONFIG_DIR**) like `~/.config/wutag.yml`.

Example configuration:
```yaml
---
max_depth: 100
pretty_output: true
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
