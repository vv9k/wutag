# 0.5.0
* **BREAKING** Global configuration will now be loaded from the users configuration directoy (for example `~/.config/wutag.yml`) and the file must not contain a `.` at the start of the filename.
* Add `update-registry` subcommand that scans for changes to the files tracked by wutag
* Add better handling of errors when a file has a maximum number of xattributes
* All output is now raw by default, to enable it use `-p` or `--pretty` global flag or set `pretty_output: true` in configuration
* Subcommands `set`, `rm` and `clear` now take a list of paths to process rather than a pattern by default. Previous behaviour was preserved with a flag `-g` or `--glob` to treat the first provided path as a glob pattern.
* Add `get` subcommand that lists tags of specified entries
* The `clear` subcommand is now split into multiple subcommands like `wutag clear tags <TAGS>...` or `wutag clear files <PATHS>...`
* A running `wutagd` daemon is now required for **wutag** to work. The daemon is responsible for tracking tagged files and filesystem changes made to them.
* **BREAKING** the format of tags has changed, all previously tagged files won't work with this version.
* Replace `wutag clean-cache` subcommand with `wutag clear cache`
* Add `--with-files`/`-f` option to `wutag list tags` to list entries tagged with each tag
* Add option to print output as JSON or YAML
* The subcommands `set` and `rm` now take tags as a named parameter `--tags`/`-t`

# 0.4.0

* Add `--raw` `-r` flag to list for easier passing to other programs [#26](https://github.com/wojciechkepka/wutag/pull/26)
* Add ability to modify base colors and other settings from a configuration file [#27](https://github.com/wojciechkepka/wutag/pull/27)
* Improved error messages
* Rewrite core functionality by using cached state instead of relying on the file system.
  WARNING! this version is completely different functionally from the older one and the tags have to be recreated
  after updating it. [#30](https://github.com/wojciechkepka/wutag/pull/30)
* Add `clean-cache` subcommand that cleans the cached tag registry from the filesystem.


# 0.3.0

* Internal representation of tags changed meaning all previous tags won't work with this version
* Change `--recursive` global parameter to `--max-depth` [#19](https://github.com/wojciechkepka/wutag/pull/19)
* Add shorthand `-r` for `--raw` flag [#22](https://github.com/wojciechkepka/wutag/pull/22)
* Unify output in subcommands [#24](https://github.com/wojciechkepka/wutag/pull/24)
* Add `--any` flag to `search` subcommand [#25](https://github.com/wojciechkepka/wutag/pull/25)


# 0.2.0

* Add `--details` flag to `list` subcommand [#18](https://github.com/wojciechkepka/wutag/pull/18)
* Add `edit` subcommand [#15](https://github.com/wojciechkepka/wutag/pull/15)
* Add `print-completions` subcommand [#17](https://github.com/wojciechkepka/wutag/pull/17)


# 0.1.1

* Fix `clear` output
