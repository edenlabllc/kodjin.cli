# generate-completions

Kodjin-cli supports auto-completion for basic commands. Currently `bash`, `fish`, `powershell` and `zsh` shells are supported. To make Auto-completion works, you have to generate the completion file..

How to use auto-completions is written in [Command auto-completion](./command-auto-completion.md)

In order to generate the file required to make the completion work you have to [install Kodjin-cli](./installation.md) first.

The command `generate-competions` generates the completion file. Different command should be used for different shells. By default this command will print on the standard output (the shell window) the content of the completion file. To save to an actual file use the > redirect symbol.

**Usage**
```shell
kodjin-cli generate-completions <SHELL>
```

Remember to open a new shell to test the functionality.

## Options:

### --install

Option `--install` automatically installs completion files for the current/selected shell

**Usage**
```shell
kodjin-cli generate-completions --install
```

### --help

Option `--help` displays help information for `generate-completions` command

**Usage**
```shell
kodjin-cli generate-completions --help
```

**Example**

```shell
kodjin-cli generate-completions --help
Generate command autocompletions

Usage: kodjin-cli generate-completions [OPTIONS] [SHELL]

Arguments:
  [SHELL]  Manually specify what shell to install completions for [possible values: bash, elvish, fish, powershell, zsh]

Options:
  -i, --install  Automatically install completion files for the current/selected shell
  -h, --help     Print help
```
