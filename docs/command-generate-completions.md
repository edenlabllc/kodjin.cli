# generate-completions

Kodjin-cli supports auto-completion for basic commands. Currently `bash`, `elvish`, `fish`, `powershell` and `zsh` shells are supported. To make Auto-completion works, you have to generates the completion file.

How to use auto-completions is written in [Command auto-completion](./command-auto-completion.md)

In order to generate the file required to make the completion work you have to [install Kodjin-cli](./installation.md) first.

The command `generate-competions` generates the completion file. Different command should be used for different shells. By default this command will print on the standard output (the shell window) the content of the completion file. To save to an actual file use the > redirect symbol.

**Usage**
```shell
kodjin-cli generate-completions <SHELL>
```

All examples are a full comands with the > redirect symbol. So, to generate the file, you have to copy and paste the command bellow into your shell.

## Bash
Use the next command to generate the file in bash 
```shell
kodjin-cli generate-completions bash | sudo tee /etc/bash_completion.d/kodjin-cli.sh
```
Remember to open a new shell to test the functionality.

## Elvish
Use the next command to generate the file in elvish
```shell
```

Remember to open a new shell to test the functionality.

## Fish
Use the next command to generate the file in fish
```shell
mkdir -p ~/.config/fish/completions/ && kodjin-cli generate-completions fish > ~/.config/fish/completions/kodjin-cli.fish
```

Remember to open a new shell to test the functionality.

For more information on where to write a completions in Fish, please, refer to [official documentation](https://fishshell.com/docs/current/completions.html#where-to-put-completions).

## Powershell
Use the next command to generate the file in fish
```shell
```

Remember to open a new shell to test the functionality.

For more information on tab-completion on PowerShell, please, refer to [Autocomplete in PowerShell](https://techcommunity.microsoft.com/t5/itops-talk-blog/autocomplete-in-powershell/ba-p/2604524).

## Zsh
Use the next command to generate the file in zsh

```shell
kodjin-cli generate-completions zsh >> ~/.zshrc
```

Remember to open a new shell to test the functionality.

Options:

## --help

Option `--help` displays help information for `generate-completions` command

**Usage**
```shell
kodjin-cli generate-completions --help
```

**Example**

```shell
$ kodjin-cli generate-completions --help
Generate command autocompletions

Usage: kodjin-cli generate-completions <SHELL>

Arguments:
  <SHELL>  [possible values: bash, elvish, fish, powershell, zsh]

Options:
  -h, --help  Print help
```
