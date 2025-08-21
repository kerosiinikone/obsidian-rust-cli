# obsidian-rust-cli (in prog.)

**Quick capture from terminal**: create a new "idea" note / append to daily notes from a "fleeting thought". Open, create (daily) notes from terminal, pretty printing. Incorporate vault statistics.

### Workflow (for self)

1. Setup the repo / vault (_config file_)
   - Read config file into memory before exec (cached?)
   - Commands to add / edit config from terminal
   - Give cfg path directly as a **flag**
2. Commands to create, append, display (_pretty_) and open (_daily_) notes
   - Define structures for `Notes`, etc
3. Vault statistics (async)
4. Visual TUI / visual formatting for notes, prompts
5. Handle signals and stdout, stderr
6. Scripts, config

### Commands (Windows PS)

```powershell
$Env:VAULT_PATH="path_to_vault"; .\target\debug\obsidian-rust-cli.exe new Hello
.\target\debug\obsidian-rust-cli.exe --vault "path_to_vault" append -n "note" Hello
```

### TODOs

- Licence
- Verbose
- Package
- Fix slashes
