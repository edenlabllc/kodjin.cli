# Command auto-completion

The Kodjin-cli includes a command auto-completion feature that enables you to use the **Tab** key to complete a partially entered command.

How to activate?
Use [generate-completions](./command-generate-completions.md) to activate auto-completion.

!!! note "Auto-completion could be already installed"

How it works?

When you enter a part of the command after a `kodjin-cli` you can press **Tab** and command auto-completion either automatically completes your command or displays a suggested list of commands.

For example, if you enter `kodjin-cli in` and press **Tab** then kodjin-cli will suggest you options to choose
```shell
$ kodjin-cli in[Tab]
info     -- Show information about a FHIR package
install  -- Install a FHIR package
```  

If you enter `kodjin-cli ins` and press **Tab** then kodjin-cli will autocomplete the command
```shell 
$ kodjin-cli install
```
